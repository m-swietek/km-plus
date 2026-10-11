use super::*;

const EPS: f64 = 1e-9;

fn start() -> StartPoint {
    StartPoint {
        date: "2026-01-01".into(),
        odometer_km: 100_000,
    }
}

fn entry(
    id: u64,
    fuel: Fuel,
    date: &str,
    odometer_km: u32,
    litres_cl: u32,
    amount_grosze: u64,
    full_tank: bool,
) -> FillUp {
    FillUp {
        id,
        fuel,
        date: date.into(),
        odometer_km,
        litres_cl,
        amount_grosze,
        full_tank,
    }
}

fn new_entry(fuel: Fuel, date: &str, odometer_km: u32, full_tank: bool) -> NewFillUp {
    NewFillUp {
        fuel,
        date: date.into(),
        odometer_km,
        litres_cl: 3_000,
        amount_grosze: 9_000,
        full_tank,
    }
}

fn result_of(results: &[(u64, FillUpResult)], id: u64) -> &FillUpResult {
    &results
        .iter()
        .find(|(i, _)| *i == id)
        .unwrap_or_else(|| panic!("no result for entry {id}"))
        .1
}

fn assert_close(actual: f64, expected: f64, what: &str) {
    assert!(
        (actual - expected).abs() < EPS,
        "{what}: expected {expected}, got {actual}"
    );
}

#[track_caller]
fn assert_consumption(
    result: &FillUpResult,
    litres_per_100km: f64,
    distance_km: u32,
    period_litres_cl: u64,
    period_cost_grosze: u64,
    price_per_litre_grosze: f64,
) {
    match result {
        FillUpResult::Consumption {
            litres_per_100km: l,
            distance_km: d,
            period_litres_cl: pl,
            period_cost_grosze: pc,
            price_per_litre_grosze: p,
        } => {
            assert_close(*l, litres_per_100km, "l/100 km");
            assert_eq!(*d, distance_km, "distance");
            assert_eq!(*pl, period_litres_cl, "period litres");
            assert_eq!(*pc, period_cost_grosze, "period cost");
            assert_close(*p, price_per_litre_grosze, "price per litre");
        }
        other => panic!("expected Consumption, got {other:?}"),
    }
}

#[track_caller]
fn assert_top_up(result: &FillUpResult, price_per_litre_grosze: f64) {
    match result {
        FillUpResult::TopUp {
            price_per_litre_grosze: p,
        } => assert_close(*p, price_per_litre_grosze, "price per litre"),
        other => panic!("expected TopUp, got {other:?}"),
    }
}

/// The "Próbka wyroczni" table from the plan; ids equal the row numbers.
fn oracle_sample() -> Vec<FillUp> {
    use Fuel::{Lpg, Petrol};
    vec![
        entry(1, Lpg, "2026-01-05", 100_400, 4_000, 12_000, true),
        entry(2, Petrol, "2026-01-08", 100_600, 1_000, 6_500, false),
        entry(3, Lpg, "2026-01-10", 100_750, 2_000, 6_200, false),
        entry(4, Lpg, "2026-01-12", 100_900, 2_500, 7_500, true),
        entry(5, Petrol, "2026-01-12", 100_900, 3_000, 19_500, true),
        entry(6, Lpg, "2026-01-20", 101_350, 4_635, 14_137, true),
        entry(7, Petrol, "2026-01-25", 101_500, 500, 3_300, false),
        entry(8, Lpg, "2026-01-28", 101_800, 1_200, 3_660, false),
        entry(9, Lpg, "2026-02-02", 102_010, 3_000, 9_150, true),
        entry(10, Petrol, "2026-02-05", 102_300, 777, 5_128, true),
    ]
}

#[test]
fn oracle_sample_matches_hand_computed_results() {
    let sample = oracle_sample();
    // Shuffled insertion order; every entry must also pass validation against
    // the ones inserted before it.
    let order = [10, 1, 7, 4, 2, 9, 5, 3, 8, 6];
    let mut fill_ups: Vec<FillUp> = Vec::new();
    for id in order {
        let f = sample.iter().find(|f| f.id == id).unwrap().clone();
        let new = NewFillUp {
            fuel: f.fuel,
            date: f.date.clone(),
            odometer_km: f.odometer_km,
            litres_cl: f.litres_cl,
            amount_grosze: f.amount_grosze,
            full_tank: f.full_tank,
        };
        assert_eq!(validate_fill_up(&start(), &fill_ups, &new), Ok(()), "entry {id}");
        fill_ups.push(new.with_id(id));
    }

    let results = compute(&start(), &fill_ups);
    assert_eq!(results.len(), 10);
    // Results come back in the order of the input list.
    let ids: Vec<u64> = results.iter().map(|(id, _)| *id).collect();
    assert_eq!(ids, order);

    let r = |id| result_of(&results, id);
    assert_consumption(r(1), 10.0, 400, 4_000, 12_000, 300.0);
    assert_top_up(r(2), 650.0);
    assert_top_up(r(3), 310.0);
    assert_consumption(r(4), 9.0, 500, 4_500, 13_700, 300.0);
    assert_consumption(r(5), 4_000.0 / 900.0, 900, 4_000, 26_000, 650.0);
    assert_consumption(r(6), 10.3, 450, 4_635, 14_137, 14_137.0 * 100.0 / 4_635.0);
    assert_top_up(r(7), 660.0);
    assert_top_up(r(8), 305.0);
    assert_consumption(r(9), 4_200.0 / 660.0, 660, 4_200, 12_810, 305.0);
    assert_consumption(r(10), 1_277.0 / 1_400.0, 1_400, 1_277, 8_428, 5_128.0 * 100.0 / 777.0);

    // Spot-check the rounded values shown in the table.
    assert_eq!(format!("{:.2}", 4_000.0_f64 / 900.0), "4.44");
    assert_eq!(format!("{:.2}", 14_137.0_f64 / 4_635.0), "3.05");
    assert_eq!(format!("{:.2}", 4_200.0_f64 / 660.0), "6.36");
    assert_eq!(format!("{:.2}", 1_277.0_f64 / 1_400.0), "0.91");
    assert_eq!(format!("{:.2}", 5_128.0_f64 / 777.0), "6.60");

    let (lpg, lpg_result) = latest_consumption(&start(), &fill_ups, Fuel::Lpg).unwrap();
    assert_eq!(lpg.id, 9);
    assert_eq!(&lpg_result, r(9));
    let (petrol, petrol_result) = latest_consumption(&start(), &fill_ups, Fuel::Petrol).unwrap();
    assert_eq!(petrol.id, 10);
    assert_eq!(&petrol_result, r(10));
}

#[test]
fn first_full_fill_up_after_start_gives_result() {
    for fuel in [Fuel::Lpg, Fuel::Petrol] {
        let fill_ups = [entry(1, fuel, "2026-01-03", 100_500, 3_500, 21_000, true)];
        let results = compute(&start(), &fill_ups);
        assert_consumption(result_of(&results, 1), 7.0, 500, 3_500, 21_000, 600.0);
    }
}

#[test]
fn top_up_never_gives_consumption() {
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-03", 100_300, 2_000, 6_000, false),
        entry(2, Fuel::Lpg, "2026-01-04", 100_600, 2_000, 6_000, false),
        entry(3, Fuel::Petrol, "2026-01-05", 100_700, 1_000, 6_500, false),
    ];
    let results = compute(&start(), &fill_ups);
    assert_top_up(result_of(&results, 1), 300.0);
    assert_top_up(result_of(&results, 2), 300.0);
    assert_top_up(result_of(&results, 3), 650.0);
    assert_eq!(latest_consumption(&start(), &fill_ups, Fuel::Lpg), None);
}

#[test]
fn top_up_litres_count_toward_next_full_period() {
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-03", 100_200, 1_000, 3_000, false),
        entry(2, Fuel::Lpg, "2026-01-04", 100_400, 1_500, 4_800, false),
        entry(3, Fuel::Lpg, "2026-01-05", 100_500, 2_500, 7_500, true),
        entry(4, Fuel::Lpg, "2026-01-06", 100_900, 3_200, 9_600, true),
    ];
    let results = compute(&start(), &fill_ups);
    // 10 + 15 + 25 l over 500 km; cost of all three.
    assert_consumption(result_of(&results, 3), 10.0, 500, 5_000, 15_300, 300.0);
    // The full tank reset the sums: only entry 4 counts in the next period.
    assert_consumption(result_of(&results, 4), 8.0, 400, 3_200, 9_600, 300.0);
}

#[test]
fn fuels_are_paired_only_with_same_fuel() {
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-02", 100_200, 2_000, 6_000, true),
        entry(2, Fuel::Petrol, "2026-01-03", 100_300, 900, 5_850, true),
        entry(3, Fuel::Lpg, "2026-01-04", 100_500, 3_000, 9_000, true),
        entry(4, Fuel::Petrol, "2026-01-05", 100_900, 600, 3_900, true),
    ];
    let results = compute(&start(), &fill_ups);
    assert_consumption(result_of(&results, 1), 10.0, 200, 2_000, 6_000, 300.0);
    // Petrol is measured from the start point, not from the LPG entry 1.
    assert_consumption(result_of(&results, 2), 3.0, 300, 900, 5_850, 650.0);
    // LPG from LPG entry 1 (100 200), ignoring petrol entry 2 in between.
    assert_consumption(result_of(&results, 3), 10.0, 300, 3_000, 9_000, 300.0);
    // Petrol from petrol entry 2 (100 300).
    assert_consumption(result_of(&results, 4), 1.0, 600, 600, 3_900, 650.0);
}

#[test]
fn backdated_entry_between_existing_recomputes_periods() {
    let mut fill_ups = vec![entry(1, Fuel::Lpg, "2026-01-10", 101_000, 8_000, 24_000, true)];
    let before = compute(&start(), &fill_ups);
    assert_consumption(result_of(&before, 1), 8.0, 1_000, 8_000, 24_000, 300.0);

    // A forgotten receipt from between the start and entry 1.
    let backdated = NewFillUp {
        fuel: Fuel::Lpg,
        date: "2026-01-05".into(),
        odometer_km: 100_400,
        litres_cl: 3_600,
        amount_grosze: 10_800,
        full_tank: true,
    };
    assert_eq!(validate_fill_up(&start(), &fill_ups, &backdated), Ok(()));
    fill_ups.push(backdated.with_id(2));

    let after = compute(&start(), &fill_ups);
    assert_consumption(result_of(&after, 2), 9.0, 400, 3_600, 10_800, 300.0);
    // The later period now starts at 100 400 and only holds entry 1's litres.
    assert_consumption(result_of(&after, 1), 8_000.0 / 600.0, 600, 8_000, 24_000, 300.0);
}

#[test]
fn zero_distance_never_divides() {
    // Bypasses validation: a full tank at the start odometer, then a second
    // full tank at the same odometer as the first.
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-01", 100_000, 1_000, 3_000, true),
        entry(2, Fuel::Petrol, "2026-01-02", 100_100, 1_000, 6_500, true),
        entry(3, Fuel::Petrol, "2026-01-02", 100_100, 500, 3_250, true),
    ];
    let results = compute(&start(), &fill_ups);
    assert_top_up(result_of(&results, 1), 300.0);
    assert_consumption(result_of(&results, 2), 10.0, 100, 1_000, 6_500, 650.0);
    assert_top_up(result_of(&results, 3), 650.0);
    for (_, r) in &results {
        if let FillUpResult::Consumption { litres_per_100km, .. } = r {
            assert!(litres_per_100km.is_finite());
        }
    }
}

#[test]
fn latest_consumption_is_highest_odometer_full_fill_up() {
    // Inserted out of order; a later top-up must not hide the full tank.
    let fill_ups = [
        entry(3, Fuel::Lpg, "2026-01-08", 100_900, 2_000, 6_000, true),
        entry(1, Fuel::Lpg, "2026-01-03", 100_400, 4_000, 12_000, true),
        entry(4, Fuel::Lpg, "2026-01-09", 101_000, 500, 1_500, false),
        entry(2, Fuel::Petrol, "2026-01-10", 101_200, 1_000, 6_500, true),
    ];
    let (latest, result) = latest_consumption(&start(), &fill_ups, Fuel::Lpg).unwrap();
    assert_eq!(latest.id, 3);
    assert_consumption(&result, 4.0, 500, 2_000, 6_000, 300.0);

    let (latest, _) = latest_consumption(&start(), &fill_ups, Fuel::Petrol).unwrap();
    assert_eq!(latest.id, 2);
}

#[test]
fn latest_consumption_none_with_only_top_ups() {
    assert_eq!(latest_consumption(&start(), &[], Fuel::Lpg), None);
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-03", 100_300, 2_000, 6_000, false),
        entry(2, Fuel::Petrol, "2026-01-04", 100_400, 1_000, 6_500, true),
    ];
    assert_eq!(latest_consumption(&start(), &fill_ups, Fuel::Lpg), None);
    assert!(latest_consumption(&start(), &fill_ups, Fuel::Petrol).is_some());
}

#[test]
fn rejects_odometer_not_after_start() {
    for km in [99_999, 100_000] {
        assert_eq!(
            validate_fill_up(&start(), &[], &new_entry(Fuel::Lpg, "2026-01-05", km, true)),
            Err(FillUpRejection::OdometerNotAfterStart)
        );
    }
    assert_eq!(
        validate_fill_up(&start(), &[], &new_entry(Fuel::Lpg, "2026-01-05", 100_001, true)),
        Ok(())
    );
}

#[test]
fn rejects_date_before_start() {
    assert_eq!(
        validate_fill_up(&start(), &[], &new_entry(Fuel::Lpg, "2025-12-31", 100_500, true)),
        Err(FillUpRejection::DateBeforeStart)
    );
    // The start date itself is allowed.
    assert_eq!(
        validate_fill_up(&start(), &[], &new_entry(Fuel::Lpg, "2026-01-01", 100_500, true)),
        Ok(())
    );
}

#[test]
fn rejects_same_fuel_same_odometer() {
    let existing = [entry(1, Fuel::Lpg, "2026-01-05", 100_400, 4_000, 12_000, true)];
    for date in ["2026-01-05", "2026-01-06"] {
        assert_eq!(
            validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, date, 100_400, false)),
            Err(FillUpRejection::SameFuelSameOdometer {
                date: "2026-01-05".into()
            })
        );
    }
}

#[test]
fn allows_other_fuel_same_odometer_same_date() {
    let existing = [entry(1, Fuel::Lpg, "2026-01-05", 100_400, 4_000, 12_000, true)];
    assert_eq!(
        validate_fill_up(
            &start(),
            &existing,
            &new_entry(Fuel::Petrol, "2026-01-05", 100_400, true)
        ),
        Ok(())
    );
}

#[test]
fn rejects_other_fuel_same_odometer_other_date() {
    let existing = [entry(1, Fuel::Lpg, "2026-01-05", 100_400, 4_000, 12_000, true)];
    for date in ["2026-01-04", "2026-01-06"] {
        assert_eq!(
            validate_fill_up(&start(), &existing, &new_entry(Fuel::Petrol, date, 100_400, true)),
            Err(FillUpRejection::OrderConflict {
                date: "2026-01-05".into(),
                odometer_km: 100_400
            })
        );
    }
}

#[test]
fn rejects_order_conflict_both_directions() {
    let existing = [
        entry(1, Fuel::Lpg, "2026-01-05", 100_400, 4_000, 12_000, true),
        entry(2, Fuel::Petrol, "2026-01-20", 101_000, 1_000, 6_500, true),
    ];
    // Higher odometer, earlier date than an entry below it.
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-04", 100_500, true)),
        Err(FillUpRejection::OrderConflict {
            date: "2026-01-05".into(),
            odometer_km: 100_400
        })
    );
    // Lower odometer, later date than an entry above it.
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-21", 100_900, true)),
        Err(FillUpRejection::OrderConflict {
            date: "2026-01-20".into(),
            odometer_km: 101_000
        })
    );
    // Below both entries with an earlier date: fine.
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-02", 100_100, true)),
        Ok(())
    );
    // Conflicting with both: the first by odometer is reported.
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-21", 100_300, true)),
        Err(FillUpRejection::OrderConflict {
            date: "2026-01-05".into(),
            odometer_km: 100_400
        })
    );
    // Equal dates on both sides are fine (backdated entry fits in between).
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-05", 100_700, true)),
        Ok(())
    );
    assert_eq!(
        validate_fill_up(&start(), &existing, &new_entry(Fuel::Lpg, "2026-01-20", 100_700, true)),
        Ok(())
    );
}

#[test]
fn rejects_out_of_range_values() {
    // Start at 0 km so the odometer bounds are not masked by the start check.
    let start = StartPoint {
        date: "2026-01-01".into(),
        odometer_km: 0,
    };
    let base = new_entry(Fuel::Lpg, "2026-01-05", 100, true);
    let check = |f: &dyn Fn(&mut NewFillUp)| {
        let mut n = base.clone();
        f(&mut n);
        validate_fill_up(&start, &[], &n)
    };

    assert_eq!(check(&|n| n.litres_cl = 0), Err(FillUpRejection::LitresOutOfRange));
    assert_eq!(check(&|n| n.litres_cl = 1), Ok(()));
    assert_eq!(check(&|n| n.litres_cl = 100_000), Ok(()));
    assert_eq!(check(&|n| n.litres_cl = 100_001), Err(FillUpRejection::LitresOutOfRange));

    assert_eq!(check(&|n| n.amount_grosze = 0), Err(FillUpRejection::AmountOutOfRange));
    assert_eq!(check(&|n| n.amount_grosze = 1), Ok(()));
    assert_eq!(check(&|n| n.amount_grosze = 10_000_000), Ok(()));
    assert_eq!(
        check(&|n| n.amount_grosze = 10_000_001),
        Err(FillUpRejection::AmountOutOfRange)
    );

    assert_eq!(check(&|n| n.odometer_km = 0), Err(FillUpRejection::OdometerOutOfRange));
    assert_eq!(check(&|n| n.odometer_km = 1), Ok(()));
    assert_eq!(check(&|n| n.odometer_km = 9_999_999), Ok(()));
    assert_eq!(
        check(&|n| n.odometer_km = 10_000_000),
        Err(FillUpRejection::OdometerOutOfRange)
    );
}

#[test]
fn rejects_invalid_calendar_dates() {
    let check = |date: &str| {
        validate_fill_up(&start(), &[], &new_entry(Fuel::Lpg, date, 100_500, true))
    };
    for bad in [
        "2026-02-29",
        "2026-13-01",
        "2026-1-5",
        "2026-00-10",
        "2026-04-31",
        "2026-01-00",
        "2100-02-29",
        "0000-01-01",
        "2026/01/05",
        "2026-01-05 ",
        "2026-0a-05",
        "+026-01-05",
        "",
    ] {
        assert_eq!(check(bad), Err(FillUpRejection::InvalidDate), "{bad:?}");
    }
    for good in ["2028-02-29", "2026-12-31", "2026-01-31"] {
        assert_eq!(check(good), Ok(()), "{good:?}");
    }
    // Century leap rule (before the start date, so checked directly).
    assert!(is_valid_date("2000-02-29"));
    assert!(!is_valid_date("1900-02-29"));
}

#[test]
fn validate_start_point_bounds() {
    let sp = |date: &str, odometer_km| StartPoint {
        date: date.into(),
        odometer_km,
    };
    assert_eq!(validate_start_point(&sp("2026-01-01", 0)), Ok(()));
    assert_eq!(validate_start_point(&sp("2026-01-01", 9_999_999)), Ok(()));
    assert_eq!(
        validate_start_point(&sp("2026-01-01", 10_000_000)),
        Err(StartPointRejection::OdometerOutOfRange)
    );
    assert_eq!(
        validate_start_point(&sp("2026-02-29", 1_000)),
        Err(StartPointRejection::InvalidDate)
    );
    assert_eq!(validate_start_point(&sp("2028-02-29", 1_000)), Ok(()));
}
