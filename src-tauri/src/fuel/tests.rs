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

fn report_of(reports: &[(Fuel, FuelReport); 2], fuel: Fuel) -> &FuelReport {
    &reports
        .iter()
        .find(|(f, _)| *f == fuel)
        .unwrap_or_else(|| panic!("no report for {fuel:?}"))
        .1
}

fn history_ids(report: &FuelReport) -> Vec<u64> {
    report.history.iter().map(|(f, _)| f.id).collect()
}

/// The latest closed period and the average of a `Consumption` summary.
#[track_caller]
fn consumption_summary(report: &FuelReport) -> (&FillUp, &FillUpResult, &AverageConsumption) {
    match &report.summary {
        FuelSummary::Consumption {
            latest: (fill_up, result),
            average,
        } => (fill_up, result, average),
        other => panic!("expected Consumption, got {other:?}"),
    }
}

#[track_caller]
fn assert_average(
    average: &AverageConsumption,
    litres_per_100km: f64,
    distance_km: u32,
    litres_cl: u64,
    periods: u32,
) {
    assert_close(average.litres_per_100km, litres_per_100km, "average l/100 km");
    assert_eq!(average.distance_km, distance_km, "average distance");
    assert_eq!(average.litres_cl, litres_cl, "average litres");
    assert_eq!(average.periods, periods, "periods");
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

    let reports = fuel_reports(&start(), &fill_ups);
    assert_eq!(reports[0].0, Fuel::Lpg);
    assert_eq!(reports[1].0, Fuel::Petrol);

    // LPG: periods closed by 1, 4, 6, 9 (400 + 500 + 450 + 660 km).
    let lpg = report_of(&reports, Fuel::Lpg);
    let (latest, latest_result, average) = consumption_summary(lpg);
    assert_eq!(latest.id, 9);
    assert_eq!(latest_result, r(9));
    assert_average(average, 17_335.0 / 2_010.0, 2_010, 17_335, 4);
    assert_eq!(history_ids(lpg), [9, 8, 6, 4, 3, 1]);

    // Petrol: periods closed by 5 and 10 (900 + 1 400 km).
    let petrol = report_of(&reports, Fuel::Petrol);
    let (latest, latest_result, average) = consumption_summary(petrol);
    assert_eq!(latest.id, 10);
    assert_eq!(latest_result, r(10));
    assert_average(average, 5_277.0 / 2_300.0, 2_300, 5_277, 2);
    assert_eq!(history_ids(petrol), [10, 7, 5, 2]);

    assert_eq!(format!("{:.4}", 17_335.0_f64 / 2_010.0), "8.6244");
    assert_eq!(format!("{:.4}", 5_277.0_f64 / 2_300.0), "2.2943");

    // Every history row is the stored entry with its own `compute` result.
    for (_, report) in &reports {
        for (f, result) in &report.history {
            assert_eq!(Some(f), sample.iter().find(|s| s.id == f.id));
            assert_eq!(result, r(f.id), "entry {}", f.id);
        }
    }
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
    let reports = fuel_reports(&start(), &fill_ups);
    assert_eq!(report_of(&reports, Fuel::Lpg).summary, FuelSummary::NoClosedPeriod);
    assert_eq!(report_of(&reports, Fuel::Petrol).summary, FuelSummary::NoClosedPeriod);
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
fn report_latest_is_highest_odometer_closed_period() {
    // Inserted out of order; a later top-up must not hide the full tank.
    let mut fill_ups = vec![
        entry(3, Fuel::Lpg, "2026-01-08", 100_900, 2_000, 6_000, true),
        entry(1, Fuel::Lpg, "2026-01-03", 100_400, 4_000, 12_000, true),
        entry(2, Fuel::Petrol, "2026-01-10", 101_200, 1_000, 6_500, true),
    ];
    let before = fuel_reports(&start(), &fill_ups);
    fill_ups.push(entry(4, Fuel::Lpg, "2026-01-09", 101_000, 500, 1_500, false));
    let reports = fuel_reports(&start(), &fill_ups);

    let lpg = report_of(&reports, Fuel::Lpg);
    let (latest, result, average) = consumption_summary(lpg);
    assert_eq!(latest.id, 3);
    assert_consumption(result, 4.0, 500, 2_000, 6_000, 300.0);
    // 40 + 20 l over 400 + 500 km; the 5 l top-up after entry 3 is left out.
    assert_average(average, 6_000.0 / 900.0, 900, 6_000, 2);
    // The top-up after the last full tank changes neither latest nor average.
    assert_eq!(lpg.summary, report_of(&before, Fuel::Lpg).summary);
    // ...but it is in the history, newest first.
    assert_eq!(history_ids(lpg), [4, 3, 1]);

    let (latest, _, _) = consumption_summary(report_of(&reports, Fuel::Petrol));
    assert_eq!(latest.id, 2);
}

#[test]
fn report_no_closed_period_with_only_top_ups() {
    let fill_ups = [
        entry(1, Fuel::Lpg, "2026-01-03", 100_300, 2_000, 6_000, false),
        entry(2, Fuel::Petrol, "2026-01-04", 100_400, 1_000, 6_500, true),
    ];
    let reports = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&reports, Fuel::Lpg);
    assert_eq!(lpg.summary, FuelSummary::NoClosedPeriod);
    assert_eq!(history_ids(lpg), [1]);
    let (latest, _, _) = consumption_summary(report_of(&reports, Fuel::Petrol));
    assert_eq!(latest.id, 2);
}

#[test]
fn report_no_entries() {
    for (_, report) in &fuel_reports(&start(), &[]) {
        assert_eq!(report.summary, FuelSummary::NoEntries);
        assert!(report.history.is_empty());
    }

    // Only the other fuel has entries.
    let fill_ups = [entry(1, Fuel::Lpg, "2026-01-03", 100_400, 4_000, 12_000, true)];
    let reports = fuel_reports(&start(), &fill_ups);
    let petrol = report_of(&reports, Fuel::Petrol);
    assert_eq!(petrol.summary, FuelSummary::NoEntries);
    assert!(petrol.history.is_empty());
    consumption_summary(report_of(&reports, Fuel::Lpg));
}

#[test]
fn report_single_closed_period_average_equals_latest() {
    let fill_ups = [
        entry(1, Fuel::Petrol, "2026-01-03", 100_200, 1_000, 6_500, false),
        entry(2, Fuel::Petrol, "2026-01-05", 100_500, 2_500, 16_250, true),
    ];
    let reports = fuel_reports(&start(), &fill_ups);
    let (latest, result, average) = consumption_summary(report_of(&reports, Fuel::Petrol));
    assert_eq!(latest.id, 2);
    assert_consumption(result, 7.0, 500, 3_500, 22_750, 650.0);
    assert_average(average, 7.0, 500, 3_500, 1);
}

#[test]
fn backdated_entry_changes_average_and_history_order() {
    let mut fill_ups = vec![
        entry(1, Fuel::Lpg, "2026-01-05", 100_400, 4_000, 12_000, true),
        entry(2, Fuel::Lpg, "2026-01-20", 101_000, 4_800, 14_400, true),
    ];
    let before = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&before, Fuel::Lpg);
    let (latest, result, average) = consumption_summary(lpg);
    assert_eq!(latest.id, 2);
    assert_consumption(result, 8.0, 600, 4_800, 14_400, 300.0);
    assert_average(average, 8.8, 1_000, 8_800, 2);
    assert_eq!(history_ids(lpg), [2, 1]);

    // A forgotten receipt from between entries 1 and 2.
    let backdated = NewFillUp {
        fuel: Fuel::Lpg,
        date: "2026-01-10".into(),
        odometer_km: 100_700,
        litres_cl: 2_400,
        amount_grosze: 7_200,
        full_tank: true,
    };
    assert_eq!(validate_fill_up(&start(), &fill_ups, &backdated), Ok(()));
    fill_ups.push(backdated.with_id(3));

    let after = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&after, Fuel::Lpg);
    let (latest, result, average) = consumption_summary(lpg);
    // Entry 2's period now starts at 100 700 and only covers 300 km.
    assert_eq!(latest.id, 2);
    assert_consumption(result, 16.0, 300, 4_800, 14_400, 300.0);
    assert_average(average, 11.2, 1_000, 11_200, 3);
    assert_eq!(history_ids(lpg), [2, 3, 1]);
    assert_consumption(&lpg.history[1].1, 8.0, 300, 2_400, 7_200, 300.0);
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

/// The entry as it would be entered again.
fn as_new(f: &FillUp) -> NewFillUp {
    NewFillUp {
        fuel: f.fuel,
        date: f.date.clone(),
        odometer_km: f.odometer_km,
        litres_cl: f.litres_cl,
        amount_grosze: f.amount_grosze,
        full_tank: f.full_tank,
    }
}

/// `fill_ups` entered again from an empty list, by `(odometer_km, id)`; every
/// entry must pass validation against the ones entered before it.
#[track_caller]
fn entered_from_scratch(fill_ups: &[FillUp]) -> Vec<FillUp> {
    let mut order: Vec<&FillUp> = fill_ups.iter().collect();
    order.sort_by_key(|f| (f.odometer_km, f.id));
    let mut rebuilt: Vec<FillUp> = Vec::new();
    for f in order {
        let new = as_new(f);
        assert_eq!(validate_fill_up(&start(), &rebuilt, &new), Ok(()), "entry {}", f.id);
        rebuilt.push(new.with_id(f.id));
    }
    rebuilt
}

/// Same reports, and the same `compute` result for every id, as the list
/// entered from scratch.
#[track_caller]
fn assert_equals_entered_from_scratch(fill_ups: &[FillUp], what: &str) {
    let rebuilt = entered_from_scratch(fill_ups);
    assert_eq!(
        fuel_reports(&start(), fill_ups),
        fuel_reports(&start(), &rebuilt),
        "{what}: reports"
    );
    let results = compute(&start(), fill_ups);
    let rebuilt_results = compute(&start(), &rebuilt);
    assert_eq!(results.len(), rebuilt_results.len(), "{what}: result count");
    for f in fill_ups {
        assert_eq!(
            result_of(&results, f.id),
            result_of(&rebuilt_results, f.id),
            "{what}: entry {}",
            f.id
        );
    }
}

#[test]
fn edit_and_delete_equal_entering_from_scratch() {
    let sample = oracle_sample();
    for f in &sample {
        let id = f.id;

        let removed = remove_fill_up(&sample, id).expect("entry exists");
        assert_eq!(removed.len(), sample.len() - 1);
        assert!(removed.iter().all(|e| e.id != id));
        assert_equals_entered_from_scratch(&removed, &format!("delete {id}"));

        let toggled = NewFillUp {
            full_tank: !f.full_tank,
            ..as_new(f)
        };
        let edited = replace_fill_up(&start(), &sample, id, toggled).expect("accepted");
        assert_equals_entered_from_scratch(&edited, &format!("toggle full tank of {id}"));

        let more_litres = NewFillUp {
            litres_cl: f.litres_cl + 100,
            ..as_new(f)
        };
        let edited = replace_fill_up(&start(), &sample, id, more_litres).expect("accepted");
        assert_equals_entered_from_scratch(&edited, &format!("+100 cl on {id}"));
    }
}

#[test]
fn delete_full_tank_merges_top_ups_into_next_period() {
    let fill_ups = remove_fill_up(&oracle_sample(), 4).expect("entry 4 exists");
    let results = compute(&start(), &fill_ups);
    // Top-up 3 now belongs to the period closed by 6: 100 400 -> 101 350.
    assert_consumption(
        result_of(&results, 6),
        6_635.0 / 950.0,
        950,
        6_635,
        20_337,
        14_137.0 * 100.0 / 4_635.0,
    );
    assert_eq!(format!("{:.4}", 6_635.0_f64 / 950.0), "6.9842");

    let reports = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&reports, Fuel::Lpg);
    let (latest, _, average) = consumption_summary(lpg);
    assert_eq!(latest.id, 9);
    assert_average(average, 14_835.0 / 2_010.0, 2_010, 14_835, 3);
    assert_eq!(history_ids(lpg), [9, 8, 6, 3, 1]);

    // Petrol is untouched.
    let petrol = report_of(&reports, Fuel::Petrol);
    let (latest, _, average) = consumption_summary(petrol);
    assert_eq!(latest.id, 10);
    assert_average(average, 5_277.0 / 2_300.0, 2_300, 5_277, 2);
    assert_eq!(petrol, report_of(&fuel_reports(&start(), &oracle_sample()), Fuel::Petrol));
}

#[test]
fn delete_only_full_tank_gives_no_closed_period() {
    let fill_ups = vec![
        entry(1, Fuel::Lpg, "2026-01-03", 100_300, 2_000, 6_000, false),
        entry(2, Fuel::Lpg, "2026-01-05", 100_500, 3_000, 9_000, true),
        entry(3, Fuel::Petrol, "2026-01-06", 100_600, 1_000, 6_500, true),
    ];
    consumption_summary(report_of(&fuel_reports(&start(), &fill_ups), Fuel::Lpg));

    let fill_ups = remove_fill_up(&fill_ups, 2).expect("entry 2 exists");
    let reports = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&reports, Fuel::Lpg);
    assert_eq!(lpg.summary, FuelSummary::NoClosedPeriod);
    assert_eq!(history_ids(lpg), [1]);

    let fill_ups = remove_fill_up(&fill_ups, 1).expect("entry 1 exists");
    let reports = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&reports, Fuel::Lpg);
    assert_eq!(lpg.summary, FuelSummary::NoEntries);
    assert!(lpg.history.is_empty());
    // The other fuel keeps its figure.
    let (latest, _, _) = consumption_summary(report_of(&reports, Fuel::Petrol));
    assert_eq!(latest.id, 3);
}

#[test]
fn edit_unchanged_entry_is_accepted() {
    let sample = oracle_sample();
    let before = fuel_reports(&start(), &sample);
    for f in &sample {
        let edited = replace_fill_up(&start(), &sample, f.id, as_new(f))
            .unwrap_or_else(|e| panic!("entry {}: {e:?}", f.id));
        assert_eq!(edited, sample, "entry {}", f.id);
        assert_eq!(fuel_reports(&start(), &edited), before, "entry {}", f.id);
    }
}

#[test]
fn edit_fuel_moves_entry_and_recomputes_both() {
    let sample = oracle_sample();
    let ten = sample.iter().find(|f| f.id == 10).unwrap();
    let as_lpg = NewFillUp {
        fuel: Fuel::Lpg,
        ..as_new(ten)
    };
    let fill_ups = replace_fill_up(&start(), &sample, 10, as_lpg).expect("accepted");
    // Same place in the list, same id, now LPG.
    assert_eq!(fill_ups.len(), sample.len());
    assert_eq!(fill_ups[9].id, 10);
    assert_eq!(fill_ups[9].fuel, Fuel::Lpg);

    let results = compute(&start(), &fill_ups);
    // Measured from LPG entry 9 (102 010), not from a petrol entry.
    assert_consumption(
        result_of(&results, 10),
        777.0 / 290.0,
        290,
        777,
        5_128,
        5_128.0 * 100.0 / 777.0,
    );
    assert_eq!(format!("{:.4}", 777.0_f64 / 290.0), "2.6793");

    let reports = fuel_reports(&start(), &fill_ups);
    let lpg = report_of(&reports, Fuel::Lpg);
    let (latest, latest_result, average) = consumption_summary(lpg);
    assert_eq!(latest.id, 10);
    assert_eq!(latest_result, result_of(&results, 10));
    assert_average(average, 18_112.0 / 2_300.0, 2_300, 18_112, 5);
    assert_eq!(history_ids(lpg), [10, 9, 8, 6, 4, 3, 1]);

    let petrol = report_of(&reports, Fuel::Petrol);
    let (latest, _, average) = consumption_summary(petrol);
    assert_eq!(latest.id, 5);
    assert_average(average, 4_000.0 / 900.0, 900, 4_000, 1);
    assert_eq!(history_ids(petrol), [7, 5, 2]);
}

#[test]
fn edit_rejected_reports_reason() {
    let sample = oracle_sample();
    let six = sample.iter().find(|f| f.id == 6).unwrap();
    let onto_four = NewFillUp {
        odometer_km: 100_900,
        ..as_new(six)
    };
    assert_eq!(
        replace_fill_up(&start(), &sample, 6, onto_four),
        Err(EditRejection::Invalid(FillUpRejection::SameFuelSameOdometer {
            date: "2026-01-12".into()
        }))
    );
}

#[test]
fn edit_or_delete_unknown_id_is_not_found() {
    let sample = oracle_sample();
    let new = new_entry(Fuel::Lpg, "2026-03-01", 103_000, true);
    assert_eq!(
        replace_fill_up(&start(), &sample, 99, new),
        Err(EditRejection::NotFound)
    );
    assert_eq!(remove_fill_up(&sample, 99), None);
}
