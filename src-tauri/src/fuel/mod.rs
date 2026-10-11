//! Fuel-ups and consumption: types stored in the vault, validation of the
//! start point and of a new fill-up, and the full-to-full consumption rule.
//!
//! Pure Rust, no Tauri and no IO. Results are never stored — they are
//! recomputed from the start point and the whole list every time, so editing
//! or deleting an entry later recomputes everything for free.
//!
//! Units: odometer in whole km, litres in centilitres, amounts in grosze; the
//! stored data is integer-only, `f64` appears only in computed results.
//! Dates are `YYYY-MM-DD` strings and are compared lexicographically.

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

/// Highest odometer reading accepted, in km.
pub const MAX_ODOMETER_KM: u32 = 9_999_999;
/// Highest litres per fill-up, in centilitres (1000,00 l).
pub const MAX_LITRES_CL: u32 = 100_000;
/// Highest amount per fill-up, in grosze (100 000,00 zł).
pub const MAX_AMOUNT_GROSZE: u64 = 10_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fuel {
    Lpg,
    Petrol,
}

/// One-time starting point: the moment both tanks were full. Acts as a
/// virtual full-tank fill-up of both fuels.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartPoint {
    /// `YYYY-MM-DD`.
    pub date: String,
    pub odometer_km: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FillUp {
    pub id: u64,
    pub fuel: Fuel,
    /// `YYYY-MM-DD`.
    pub date: String,
    pub odometer_km: u32,
    pub litres_cl: u32,
    pub amount_grosze: u64,
    pub full_tank: bool,
}

/// A fill-up as entered, before it gets an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewFillUp {
    pub fuel: Fuel,
    pub date: String,
    pub odometer_km: u32,
    pub litres_cl: u32,
    pub amount_grosze: u64,
    pub full_tank: bool,
}

impl NewFillUp {
    pub fn with_id(self, id: u64) -> FillUp {
        FillUp {
            id,
            fuel: self.fuel,
            date: self.date,
            odometer_km: self.odometer_km,
            litres_cl: self.litres_cl,
            amount_grosze: self.amount_grosze,
            full_tank: self.full_tank,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StartPointRejection {
    InvalidDate,
    OdometerOutOfRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FillUpRejection {
    InvalidDate,
    OdometerOutOfRange,
    LitresOutOfRange,
    AmountOutOfRange,
    DateBeforeStart,
    OdometerNotAfterStart,
    /// A fill-up of the same fuel already exists at this odometer reading.
    SameFuelSameOdometer { date: String },
    /// Odometer and date would not grow together with this existing entry.
    OrderConflict { date: String, odometer_km: u32 },
}

#[derive(Debug, Clone, PartialEq)]
pub enum FillUpResult {
    /// A full-tank fill-up that closed a consumption period.
    Consumption {
        litres_per_100km: f64,
        distance_km: u32,
        period_litres_cl: u64,
        period_cost_grosze: u64,
        price_per_litre_grosze: f64,
    },
    /// No consumption figure: a partial top-up (or, defensively, a full tank
    /// with zero distance since the previous one).
    TopUp { price_per_litre_grosze: f64 },
}

/// The start odometer may be 0 (a brand-new car); the date must be a real
/// calendar date.
pub fn validate_start_point(start: &StartPoint) -> Result<(), StartPointRejection> {
    if !is_valid_date(&start.date) {
        return Err(StartPointRejection::InvalidDate);
    }
    if start.odometer_km > MAX_ODOMETER_KM {
        return Err(StartPointRejection::OdometerOutOfRange);
    }
    Ok(())
}

/// Accepts a new entry only when it keeps odometer and date growing together
/// with every existing entry, so every consumption period has a distance > 0.
/// On a conflict, the first colliding entry (by odometer) is reported.
pub fn validate_fill_up(
    start: &StartPoint,
    existing: &[FillUp],
    new: &NewFillUp,
) -> Result<(), FillUpRejection> {
    if !is_valid_date(&new.date) {
        return Err(FillUpRejection::InvalidDate);
    }
    if !(1..=MAX_ODOMETER_KM).contains(&new.odometer_km) {
        return Err(FillUpRejection::OdometerOutOfRange);
    }
    if !(1..=MAX_LITRES_CL).contains(&new.litres_cl) {
        return Err(FillUpRejection::LitresOutOfRange);
    }
    if !(1..=MAX_AMOUNT_GROSZE).contains(&new.amount_grosze) {
        return Err(FillUpRejection::AmountOutOfRange);
    }
    if new.date.as_str() < start.date.as_str() {
        return Err(FillUpRejection::DateBeforeStart);
    }
    if new.odometer_km <= start.odometer_km {
        return Err(FillUpRejection::OdometerNotAfterStart);
    }

    let mut sorted: Vec<&FillUp> = existing.iter().collect();
    sorted.sort_by(|a, b| {
        (a.odometer_km, &a.date, a.id).cmp(&(b.odometer_km, &b.date, b.id))
    });
    for e in sorted {
        let conflict = || FillUpRejection::OrderConflict {
            date: e.date.clone(),
            odometer_km: e.odometer_km,
        };
        if e.odometer_km < new.odometer_km {
            if e.date > new.date {
                return Err(conflict());
            }
        } else if e.odometer_km > new.odometer_km {
            if e.date < new.date {
                return Err(conflict());
            }
        } else if e.fuel == new.fuel {
            return Err(FillUpRejection::SameFuelSameOdometer {
                date: e.date.clone(),
            });
        } else if e.date != new.date {
            return Err(conflict());
        }
    }
    Ok(())
}

/// Result for every entry, in the order of `fill_ups`. Each fuel is handled
/// on its own: entries sorted by odometer, the start point as the first
/// anchor, litres and amounts summed until a full tank closes the period.
pub fn compute(start: &StartPoint, fill_ups: &[FillUp]) -> Vec<(u64, FillUpResult)> {
    let mut results: Vec<Option<FillUpResult>> = vec![None; fill_ups.len()];
    for fuel in [Fuel::Lpg, Fuel::Petrol] {
        let mut indices: Vec<usize> = (0..fill_ups.len())
            .filter(|&i| fill_ups[i].fuel == fuel)
            .collect();
        indices.sort_by_key(|&i| (fill_ups[i].odometer_km, fill_ups[i].id));

        let mut anchor_km = start.odometer_km;
        let mut sum_litres_cl: u64 = 0;
        let mut sum_grosze: u64 = 0;
        for i in indices {
            let f = &fill_ups[i];
            sum_litres_cl += u64::from(f.litres_cl);
            sum_grosze += f.amount_grosze;
            let price_per_litre_grosze = price_per_litre_grosze(f);
            let distance_km = f.odometer_km.saturating_sub(anchor_km);
            results[i] = Some(if f.full_tank && distance_km > 0 {
                let result = FillUpResult::Consumption {
                    // litres / km * 100 == centilitres / km
                    litres_per_100km: sum_litres_cl as f64 / f64::from(distance_km),
                    distance_km,
                    period_litres_cl: sum_litres_cl,
                    period_cost_grosze: sum_grosze,
                    price_per_litre_grosze,
                };
                anchor_km = f.odometer_km;
                sum_litres_cl = 0;
                sum_grosze = 0;
                result
            } else {
                FillUpResult::TopUp {
                    price_per_litre_grosze,
                }
            });
        }
    }
    fill_ups
        .iter()
        .zip(results)
        .map(|(f, r)| (f.id, r.expect("every entry belongs to one fuel")))
        .collect()
}

/// The full-tank fill-up of `fuel` with the highest odometer, with its result.
pub fn latest_consumption(
    start: &StartPoint,
    fill_ups: &[FillUp],
    fuel: Fuel,
) -> Option<(FillUp, FillUpResult)> {
    let latest = fill_ups
        .iter()
        .filter(|f| f.fuel == fuel && f.full_tank)
        .max_by_key(|f| (f.odometer_km, f.id))?;
    let result = compute(start, fill_ups)
        .into_iter()
        .find(|(id, _)| *id == latest.id)
        .map(|(_, r)| r)?;
    Some((latest.clone(), result))
}

/// Price per litre in grosze: amount / (centilitres / 100).
fn price_per_litre_grosze(f: &FillUp) -> f64 {
    f.amount_grosze as f64 * 100.0 / f64::from(f.litres_cl)
}

/// Strict `YYYY-MM-DD` with a real calendar day (leap years included).
pub fn is_valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let digits = |r: std::ops::Range<usize>| -> Option<u32> {
        b[r].iter().try_fold(0u32, |acc, &c| {
            c.is_ascii_digit().then(|| acc * 10 + u32::from(c - b'0'))
        })
    };
    let (Some(year), Some(month), Some(day)) = (digits(0..4), digits(5..7), digits(8..10)) else {
        return false;
    };
    if year == 0 || !(1..=12).contains(&month) || day == 0 {
        return false;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_in_month = match month {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    day <= days_in_month
}
