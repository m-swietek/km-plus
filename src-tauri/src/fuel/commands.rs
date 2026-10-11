//! Tauri command layer over the fuel domain: reading the fuel overview,
//! setting the start point and adding a fill-up. Every command works only on
//! an unlocked vault session; writes go through `UnlockedVault::commit`, so a
//! failed save leaves the session data as it was.
//!
//! The domain types in `super` stay serde-free for results and rejections;
//! the frontend JSON shape is owned here (DTOs below, `kind`-tagged unions,
//! camelCase fields).

use serde::{Deserialize, Serialize};
use tauri::State;

use super::{
    compute, latest_consumption, validate_fill_up, validate_start_point, Fuel, FillUpRejection,
    FillUpResult, NewFillUp, StartPoint, StartPointRejection,
};
use crate::vault::commands::{lock_session, VaultSession};
use crate::vault::{UnlockedVault, VaultData, VaultError};

const LOCKED_ERROR: &str = "dane są zablokowane; odblokuj aplikację";

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FuelOverview {
    pub start_point: Option<StartPointView>,
    pub lpg: Option<LatestConsumption>,
    pub petrol: Option<LatestConsumption>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StartPointView {
    pub date: String,
    pub odometer_km: u32,
}

/// The latest full-tank fill-up of one fuel that closed a consumption period.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LatestConsumption {
    pub date: String,
    pub odometer_km: u32,
    #[serde(rename = "litresPer100Km")]
    pub litres_per_100km: f64,
    pub distance_km: u32,
    pub period_cost_grosze: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SetStartPointResult {
    Ok,
    AlreadySet,
    Invalid { reason: StartPointRejectionView },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StartPointRejectionView {
    InvalidDate,
    OdometerOutOfRange,
}

/// A fill-up as sent by the frontend (`add_fill_up` argument `input`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FillUpInput {
    pub fuel: Fuel,
    pub date: String,
    pub odometer_km: u32,
    pub litres_cl: u32,
    pub amount_grosze: u64,
    pub full_tank: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum AddFillUpResult {
    Saved { result: FillUpResultView },
    NoStartPoint,
    Rejected { reason: FillUpRejectionView },
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FillUpResultView {
    Consumption {
        #[serde(rename = "litresPer100Km")]
        litres_per_100km: f64,
        distance_km: u32,
        period_litres_cl: u64,
        period_cost_grosze: u64,
        price_per_litre_grosze: f64,
    },
    TopUp {
        price_per_litre_grosze: f64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FillUpRejectionView {
    InvalidDate,
    OdometerOutOfRange,
    LitresOutOfRange,
    AmountOutOfRange,
    DateBeforeStart,
    OdometerNotAfterStart,
    SameFuelSameOdometer { date: String },
    OrderConflict { date: String, odometer_km: u32 },
}

impl From<StartPointRejection> for StartPointRejectionView {
    fn from(r: StartPointRejection) -> Self {
        match r {
            StartPointRejection::InvalidDate => Self::InvalidDate,
            StartPointRejection::OdometerOutOfRange => Self::OdometerOutOfRange,
        }
    }
}

impl From<FillUpRejection> for FillUpRejectionView {
    fn from(r: FillUpRejection) -> Self {
        match r {
            FillUpRejection::InvalidDate => Self::InvalidDate,
            FillUpRejection::OdometerOutOfRange => Self::OdometerOutOfRange,
            FillUpRejection::LitresOutOfRange => Self::LitresOutOfRange,
            FillUpRejection::AmountOutOfRange => Self::AmountOutOfRange,
            FillUpRejection::DateBeforeStart => Self::DateBeforeStart,
            FillUpRejection::OdometerNotAfterStart => Self::OdometerNotAfterStart,
            FillUpRejection::SameFuelSameOdometer { date } => Self::SameFuelSameOdometer { date },
            FillUpRejection::OrderConflict { date, odometer_km } => {
                Self::OrderConflict { date, odometer_km }
            }
        }
    }
}

impl From<FillUpResult> for FillUpResultView {
    fn from(r: FillUpResult) -> Self {
        match r {
            FillUpResult::Consumption {
                litres_per_100km,
                distance_km,
                period_litres_cl,
                period_cost_grosze,
                price_per_litre_grosze,
            } => Self::Consumption {
                litres_per_100km,
                distance_km,
                period_litres_cl,
                period_cost_grosze,
                price_per_litre_grosze,
            },
            FillUpResult::TopUp {
                price_per_litre_grosze,
            } => Self::TopUp {
                price_per_litre_grosze,
            },
        }
    }
}

impl From<FillUpInput> for NewFillUp {
    fn from(i: FillUpInput) -> Self {
        NewFillUp {
            fuel: i.fuel,
            date: i.date,
            odometer_km: i.odometer_km,
            litres_cl: i.litres_cl,
            amount_grosze: i.amount_grosze,
            full_tank: i.full_tank,
        }
    }
}

/// Start point and the latest consumption figure of each fuel.
#[tauri::command]
pub async fn fuel_overview(session: State<'_, VaultSession>) -> Result<FuelOverview, String> {
    with_vault(&session, |vault| Ok(overview(&vault.data)))
}

/// Sets the one-time start point; refused once it is set.
#[tauri::command]
pub async fn set_start_point(
    session: State<'_, VaultSession>,
    date: String,
    odometer_km: u32,
) -> Result<SetStartPointResult, String> {
    with_vault(&session, |vault| {
        apply_start_point(vault, StartPoint { date, odometer_km })
    })
}

/// Validates and saves a new fill-up, returning its computed result.
#[tauri::command]
pub async fn add_fill_up(
    session: State<'_, VaultSession>,
    input: FillUpInput,
) -> Result<AddFillUpResult, String> {
    with_vault(&session, |vault| apply_fill_up(vault, input.into()))
}

/// Runs `f` on the unlocked vault; `Err` when the session is locked.
fn with_vault<T>(
    session: &VaultSession,
    f: impl FnOnce(&mut UnlockedVault) -> Result<T, VaultError>,
) -> Result<T, String> {
    let mut guard = lock_session(session)?;
    let vault = guard.as_mut().ok_or_else(|| LOCKED_ERROR.to_string())?;
    f(vault).map_err(|e| e.to_string())
}

fn overview(data: &VaultData) -> FuelOverview {
    let Some(start) = &data.start_point else {
        return FuelOverview {
            start_point: None,
            lpg: None,
            petrol: None,
        };
    };
    let latest = |fuel| {
        let (fill_up, result) = latest_consumption(start, &data.fill_ups, fuel)?;
        // Only a closed period gives a figure; anything else shows nothing.
        match result {
            FillUpResult::Consumption {
                litres_per_100km,
                distance_km,
                period_cost_grosze,
                ..
            } => Some(LatestConsumption {
                date: fill_up.date,
                odometer_km: fill_up.odometer_km,
                litres_per_100km,
                distance_km,
                period_cost_grosze,
            }),
            FillUpResult::TopUp { .. } => None,
        }
    };
    FuelOverview {
        start_point: Some(StartPointView {
            date: start.date.clone(),
            odometer_km: start.odometer_km,
        }),
        lpg: latest(Fuel::Lpg),
        petrol: latest(Fuel::Petrol),
    }
}

fn apply_start_point(
    vault: &mut UnlockedVault,
    start: StartPoint,
) -> Result<SetStartPointResult, VaultError> {
    if vault.data.start_point.is_some() {
        return Ok(SetStartPointResult::AlreadySet);
    }
    if let Err(reason) = validate_start_point(&start) {
        return Ok(SetStartPointResult::Invalid {
            reason: reason.into(),
        });
    }
    let mut next = vault.data.clone();
    next.start_point = Some(start);
    vault.commit(next)?;
    Ok(SetStartPointResult::Ok)
}

fn apply_fill_up(
    vault: &mut UnlockedVault,
    new: NewFillUp,
) -> Result<AddFillUpResult, VaultError> {
    let Some(start) = vault.data.start_point.clone() else {
        return Ok(AddFillUpResult::NoStartPoint);
    };
    if let Err(reason) = validate_fill_up(&start, &vault.data.fill_ups, &new) {
        return Ok(AddFillUpResult::Rejected {
            reason: reason.into(),
        });
    }
    let mut next = vault.data.clone();
    let id = next.next_fill_up_id;
    next.next_fill_up_id += 1;
    next.fill_ups.push(new.with_id(id));
    let result = compute(&start, &next.fill_ups)
        .into_iter()
        .find(|(entry_id, _)| *entry_id == id)
        .map(|(_, r)| r)
        .expect("compute returns a result for every entry");
    vault.commit(next)?;
    Ok(AddFillUpResult::Saved {
        result: result.into(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vault::KdfParams;
    use serde_json::{json, to_value};
    use tempfile::TempDir;

    fn new_vault(dir: &std::path::Path) -> UnlockedVault {
        let cheap = KdfParams {
            m_cost: 8,
            t_cost: 1,
            p_cost: 1,
        };
        crate::vault::create(dir, "correct horse", cheap).expect("create vault")
    }

    fn start() -> StartPoint {
        StartPoint {
            date: "2026-01-01".into(),
            odometer_km: 100_000,
        }
    }

    fn lpg(date: &str, odometer_km: u32, full_tank: bool) -> NewFillUp {
        NewFillUp {
            fuel: Fuel::Lpg,
            date: date.into(),
            odometer_km,
            litres_cl: 4_000,
            amount_grosze: 12_000,
            full_tank,
        }
    }

    #[test]
    fn fuel_overview_serializes_to_frontend_shape() {
        let empty = FuelOverview {
            start_point: None,
            lpg: None,
            petrol: None,
        };
        assert_eq!(
            to_value(empty).unwrap(),
            json!({"startPoint": null, "lpg": null, "petrol": null})
        );
        let full = FuelOverview {
            start_point: Some(StartPointView {
                date: "2026-01-01".into(),
                odometer_km: 100_000,
            }),
            lpg: Some(LatestConsumption {
                date: "2026-01-05".into(),
                odometer_km: 100_400,
                litres_per_100km: 10.0,
                distance_km: 400,
                period_cost_grosze: 12_000,
            }),
            petrol: None,
        };
        assert_eq!(
            to_value(full).unwrap(),
            json!({
                "startPoint": {"date": "2026-01-01", "odometerKm": 100_000},
                "lpg": {
                    "date": "2026-01-05",
                    "odometerKm": 100_400,
                    "litresPer100Km": 10.0,
                    "distanceKm": 400,
                    "periodCostGrosze": 12_000
                },
                "petrol": null
            })
        );
    }

    #[test]
    fn set_start_point_result_serializes_to_frontend_shape() {
        assert_eq!(to_value(SetStartPointResult::Ok).unwrap(), json!({"kind": "ok"}));
        assert_eq!(
            to_value(SetStartPointResult::AlreadySet).unwrap(),
            json!({"kind": "alreadySet"})
        );
        assert_eq!(
            to_value(SetStartPointResult::Invalid {
                reason: StartPointRejectionView::InvalidDate
            })
            .unwrap(),
            json!({"kind": "invalid", "reason": "invalidDate"})
        );
        assert_eq!(
            to_value(SetStartPointResult::Invalid {
                reason: StartPointRejectionView::OdometerOutOfRange
            })
            .unwrap(),
            json!({"kind": "invalid", "reason": "odometerOutOfRange"})
        );
    }

    #[test]
    fn add_fill_up_result_serializes_to_frontend_shape() {
        assert_eq!(
            to_value(AddFillUpResult::Saved {
                result: FillUpResultView::Consumption {
                    litres_per_100km: 10.0,
                    distance_km: 400,
                    period_litres_cl: 4_000,
                    period_cost_grosze: 12_000,
                    price_per_litre_grosze: 300.0,
                }
            })
            .unwrap(),
            json!({
                "kind": "saved",
                "result": {
                    "kind": "consumption",
                    "litresPer100Km": 10.0,
                    "distanceKm": 400,
                    "periodLitresCl": 4_000,
                    "periodCostGrosze": 12_000,
                    "pricePerLitreGrosze": 300.0
                }
            })
        );
        assert_eq!(
            to_value(AddFillUpResult::Saved {
                result: FillUpResultView::TopUp {
                    price_per_litre_grosze: 650.0
                }
            })
            .unwrap(),
            json!({"kind": "saved", "result": {"kind": "topUp", "pricePerLitreGrosze": 650.0}})
        );
        assert_eq!(
            to_value(AddFillUpResult::NoStartPoint).unwrap(),
            json!({"kind": "noStartPoint"})
        );
        assert_eq!(
            to_value(AddFillUpResult::Rejected {
                reason: FillUpRejectionView::InvalidDate
            })
            .unwrap(),
            json!({"kind": "rejected", "reason": {"kind": "invalidDate"}})
        );
    }

    #[test]
    fn fill_up_rejection_serializes_to_frontend_shape() {
        let unit = [
            (FillUpRejection::InvalidDate, "invalidDate"),
            (FillUpRejection::OdometerOutOfRange, "odometerOutOfRange"),
            (FillUpRejection::LitresOutOfRange, "litresOutOfRange"),
            (FillUpRejection::AmountOutOfRange, "amountOutOfRange"),
            (FillUpRejection::DateBeforeStart, "dateBeforeStart"),
            (FillUpRejection::OdometerNotAfterStart, "odometerNotAfterStart"),
        ];
        for (rejection, kind) in unit {
            assert_eq!(
                to_value(FillUpRejectionView::from(rejection)).unwrap(),
                json!({"kind": kind})
            );
        }
        assert_eq!(
            to_value(FillUpRejectionView::from(FillUpRejection::SameFuelSameOdometer {
                date: "2026-01-05".into()
            }))
            .unwrap(),
            json!({"kind": "sameFuelSameOdometer", "date": "2026-01-05"})
        );
        assert_eq!(
            to_value(FillUpRejectionView::from(FillUpRejection::OrderConflict {
                date: "2026-01-05".into(),
                odometer_km: 100_400
            }))
            .unwrap(),
            json!({"kind": "orderConflict", "date": "2026-01-05", "odometerKm": 100_400})
        );
    }

    #[test]
    fn fill_up_input_deserializes_from_frontend_shape() {
        let input: FillUpInput = serde_json::from_value(json!({
            "fuel": "petrol",
            "date": "2026-01-08",
            "odometerKm": 100_600,
            "litresCl": 1_000,
            "amountGrosze": 6_500,
            "fullTank": false
        }))
        .unwrap();
        assert_eq!(
            input,
            FillUpInput {
                fuel: Fuel::Petrol,
                date: "2026-01-08".into(),
                odometer_km: 100_600,
                litres_cl: 1_000,
                amount_grosze: 6_500,
                full_tank: false,
            }
        );
    }

    #[test]
    fn start_point_is_set_once_and_saved() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        assert_eq!(
            apply_start_point(&mut vault, StartPoint { date: "2026-02-30".into(), odometer_km: 0 })
                .unwrap(),
            SetStartPointResult::Invalid {
                reason: StartPointRejectionView::InvalidDate
            }
        );
        assert_eq!(vault.data.start_point, None);
        assert_eq!(
            apply_start_point(&mut vault, start()).unwrap(),
            SetStartPointResult::Ok
        );
        assert_eq!(
            apply_start_point(
                &mut vault,
                StartPoint { date: "2026-03-01".into(), odometer_km: 5 }
            )
            .unwrap(),
            SetStartPointResult::AlreadySet
        );
        assert_eq!(vault.data.start_point, Some(start()));

        let reopened = match crate::vault::unlock(dir.path(), "correct horse").unwrap() {
            crate::vault::UnlockOutcome::Unlocked(v) => v,
            other => panic!("unexpected outcome: {other:?}"),
        };
        assert_eq!(reopened.data.start_point, Some(start()));
    }

    #[test]
    fn fill_up_needs_start_point_and_valid_input() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        assert_eq!(
            apply_fill_up(&mut vault, lpg("2026-01-05", 100_400, true)).unwrap(),
            AddFillUpResult::NoStartPoint
        );
        apply_start_point(&mut vault, start()).unwrap();
        assert_eq!(
            apply_fill_up(&mut vault, lpg("2025-12-31", 100_400, true)).unwrap(),
            AddFillUpResult::Rejected {
                reason: FillUpRejectionView::DateBeforeStart
            }
        );
        assert!(vault.data.fill_ups.is_empty());
        assert_eq!(vault.data.next_fill_up_id, 1);
    }

    #[test]
    fn saved_fill_ups_get_increasing_ids_and_feed_the_overview() {
        let dir = TempDir::new().unwrap();
        let mut vault = new_vault(dir.path());
        apply_start_point(&mut vault, start()).unwrap();

        assert_eq!(
            apply_fill_up(&mut vault, lpg("2026-01-03", 100_200, false)).unwrap(),
            AddFillUpResult::Saved {
                result: FillUpResultView::TopUp {
                    price_per_litre_grosze: 300.0
                }
            }
        );
        assert_eq!(overview(&vault.data).lpg, None);

        assert_eq!(
            apply_fill_up(&mut vault, lpg("2026-01-05", 100_400, true)).unwrap(),
            AddFillUpResult::Saved {
                result: FillUpResultView::Consumption {
                    litres_per_100km: 20.0,
                    distance_km: 400,
                    period_litres_cl: 8_000,
                    period_cost_grosze: 24_000,
                    price_per_litre_grosze: 300.0,
                }
            }
        );
        let ids: Vec<u64> = vault.data.fill_ups.iter().map(|f| f.id).collect();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(vault.data.next_fill_up_id, 3);

        assert_eq!(
            overview(&vault.data),
            FuelOverview {
                start_point: Some(StartPointView {
                    date: "2026-01-01".into(),
                    odometer_km: 100_000
                }),
                lpg: Some(LatestConsumption {
                    date: "2026-01-05".into(),
                    odometer_km: 100_400,
                    litres_per_100km: 20.0,
                    distance_km: 400,
                    period_cost_grosze: 24_000,
                }),
                petrol: None,
            }
        );
    }
}
