use super::*;

/// A one-off service with only an odometer threshold.
fn by_km(km: u32) -> NewService {
    NewService {
        name: "Wymiana oleju".into(),
        note: None,
        due_odometer_km: Some(km),
        due_date: None,
        recurrence: None,
    }
}

/// A one-off service with only a date threshold.
fn by_date(date: &str) -> NewService {
    NewService {
        name: "Przegląd techniczny".into(),
        note: None,
        due_odometer_km: None,
        due_date: Some(date.into()),
        recurrence: None,
    }
}

fn every(every_km: Option<u32>, every_months: Option<u32>) -> Option<Recurrence> {
    Some(Recurrence {
        every_km,
        every_months,
    })
}

#[track_caller]
fn rejected(new: NewService) -> ServiceRejection {
    validate_service(&new.normalize()).expect_err("expected a rejection")
}

#[track_caller]
fn accepted(new: NewService) {
    assert_eq!(validate_service(&new.normalize()), Ok(()));
}

// --- Normalization ---

#[test]
fn normalize_trims_name_and_note() {
    let new = NewService {
        name: "  Rozrząd \t".into(),
        note: Some("\n 5W-30  ".into()),
        ..by_km(115_000)
    }
    .normalize();
    assert_eq!(new.name, "Rozrząd");
    assert_eq!(new.note.as_deref(), Some("5W-30"));
    // Inner whitespace stays.
    let new = NewService {
        name: " Płyn  hamulcowy ".into(),
        ..by_km(1)
    }
    .normalize();
    assert_eq!(new.name, "Płyn  hamulcowy");
}

#[test]
fn whitespace_only_note_becomes_none() {
    for note in ["", "   ", " \t\n "] {
        let new = NewService {
            note: Some(note.into()),
            ..by_km(115_000)
        }
        .normalize();
        assert_eq!(new.note, None, "note {note:?}");
    }
    assert_eq!(by_km(115_000).normalize().note, None);
}

#[test]
fn with_id_keeps_every_field() {
    let new = NewService {
        name: "Rozrząd".into(),
        note: Some("pasek + pompa".into()),
        due_odometer_km: Some(180_000),
        due_date: Some("2028-06-01".into()),
        recurrence: every(Some(90_000), Some(48)),
    };
    assert_eq!(
        new.clone().with_id(7),
        PlannedService {
            id: 7,
            name: new.name,
            note: new.note,
            due_odometer_km: new.due_odometer_km,
            due_date: new.due_date,
            recurrence: new.recurrence,
        }
    );
}

// --- Name and note ---

#[test]
fn empty_name_is_rejected() {
    for name in ["", "    ", "\t"] {
        assert_eq!(
            rejected(NewService {
                name: name.into(),
                ..by_km(115_000)
            }),
            ServiceRejection::NameEmpty,
            "name {name:?}"
        );
    }
}

#[test]
fn name_length_limit_counts_unicode_chars() {
    // Multi-byte Polish letters: 100 chars is well over 100 bytes.
    let at_limit = "ąęółżźćń".repeat(12) + "ąęść";
    assert_eq!(at_limit.chars().count(), 100);
    assert!(at_limit.len() > 100);
    accepted(NewService {
        name: at_limit.clone(),
        ..by_km(115_000)
    });
    // Surrounding spaces are trimmed before counting.
    accepted(NewService {
        name: format!("  {at_limit}  "),
        ..by_km(115_000)
    });

    assert_eq!(
        rejected(NewService {
            name: at_limit + "ż",
            ..by_km(115_000)
        }),
        ServiceRejection::NameTooLong
    );
}

#[test]
fn note_length_limit_counts_unicode_chars() {
    let at_limit = "żółć".repeat(125);
    assert_eq!(at_limit.chars().count(), 500);
    accepted(NewService {
        note: Some(at_limit.clone()),
        ..by_km(115_000)
    });
    accepted(NewService {
        note: Some(format!(" {at_limit} ")),
        ..by_km(115_000)
    });

    assert_eq!(
        rejected(NewService {
            note: Some(at_limit + "ą"),
            ..by_km(115_000)
        }),
        ServiceRejection::NoteTooLong
    );
}

// --- Thresholds ---

#[test]
fn no_threshold_is_rejected() {
    assert_eq!(
        rejected(NewService {
            due_odometer_km: None,
            ..by_km(115_000)
        }),
        ServiceRejection::NoThreshold
    );
}

#[test]
fn km_only_date_only_and_both_are_accepted() {
    accepted(by_km(115_000));
    accepted(by_date("2027-03-01"));
    accepted(NewService {
        due_date: Some("2028-06-01".into()),
        ..by_km(180_000)
    });
}

#[test]
fn thresholds_in_the_past_are_accepted() {
    // Not compared with today, the start point or the fill-ups.
    accepted(by_date("2020-01-01"));
    accepted(by_km(1));
    accepted(NewService {
        due_date: Some("2020-01-01".into()),
        ..by_km(1)
    });
}

#[test]
fn odometer_threshold_out_of_range_is_rejected() {
    accepted(by_km(1));
    accepted(by_km(MAX_ODOMETER_KM));
    for km in [0, 10_000_000, u32::MAX] {
        assert_eq!(
            rejected(by_km(km)),
            ServiceRejection::OdometerOutOfRange,
            "km {km}"
        );
    }
}

#[test]
fn invalid_date_is_rejected() {
    accepted(by_date("2028-02-29"));
    for date in ["", "2027-02-29", "2027-13-01", "01.03.2027", "2027-3-1"] {
        assert_eq!(
            rejected(by_date(date)),
            ServiceRejection::InvalidDate,
            "date {date:?}"
        );
    }
}

// --- Recurrence ---

#[test]
fn recurring_with_matching_intervals_is_accepted() {
    accepted(NewService {
        recurrence: every(Some(15_000), None),
        ..by_km(115_000)
    });
    accepted(NewService {
        recurrence: every(None, Some(12)),
        ..by_date("2027-03-01")
    });
    accepted(NewService {
        due_date: Some("2028-06-01".into()),
        recurrence: every(Some(90_000), Some(48)),
        ..by_km(180_000)
    });
}

#[test]
fn interval_mismatch_is_rejected() {
    // Odometer threshold without a km interval.
    assert_eq!(
        rejected(NewService {
            recurrence: every(None, None),
            ..by_km(115_000)
        }),
        ServiceRejection::IntervalMismatch
    );
    // Months interval without a date threshold.
    assert_eq!(
        rejected(NewService {
            recurrence: every(Some(15_000), Some(12)),
            ..by_km(115_000)
        }),
        ServiceRejection::IntervalMismatch
    );
    // No interval at all for a date threshold.
    assert_eq!(
        rejected(NewService {
            recurrence: every(None, None),
            ..by_date("2027-03-01")
        }),
        ServiceRejection::IntervalMismatch
    );
    // Both thresholds, only one interval.
    assert_eq!(
        rejected(NewService {
            due_date: Some("2028-06-01".into()),
            recurrence: every(Some(90_000), None),
            ..by_km(180_000)
        }),
        ServiceRejection::IntervalMismatch
    );
    // Km interval without an odometer threshold.
    assert_eq!(
        rejected(NewService {
            recurrence: every(Some(15_000), Some(12)),
            ..by_date("2027-03-01")
        }),
        ServiceRejection::IntervalMismatch
    );
}

#[test]
fn km_interval_out_of_range_is_rejected() {
    accepted(NewService {
        recurrence: every(Some(1), None),
        ..by_km(115_000)
    });
    accepted(NewService {
        recurrence: every(Some(MAX_ODOMETER_KM), None),
        ..by_km(115_000)
    });
    for km in [0, 10_000_000] {
        assert_eq!(
            rejected(NewService {
                recurrence: every(Some(km), None),
                ..by_km(115_000)
            }),
            ServiceRejection::IntervalKmOutOfRange,
            "every_km {km}"
        );
    }
}

#[test]
fn months_interval_out_of_range_is_rejected() {
    for months in [1, 120] {
        accepted(NewService {
            recurrence: every(None, Some(months)),
            ..by_date("2027-03-01")
        });
    }
    for months in [0, 121] {
        assert_eq!(
            rejected(NewService {
                recurrence: every(None, Some(months)),
                ..by_date("2027-03-01")
            }),
            ServiceRejection::IntervalMonthsOutOfRange,
            "every_months {months}"
        );
    }
}

#[test]
fn first_failing_rule_is_reported() {
    // Empty name wins over everything else.
    assert_eq!(
        rejected(NewService {
            name: " ".into(),
            note: Some("x".repeat(501)),
            due_odometer_km: None,
            due_date: None,
            recurrence: every(Some(0), Some(0)),
        }),
        ServiceRejection::NameEmpty
    );
    // Note before thresholds.
    assert_eq!(
        rejected(NewService {
            note: Some("x".repeat(501)),
            due_odometer_km: None,
            ..by_km(1)
        }),
        ServiceRejection::NoteTooLong
    );
    // Odometer range before date.
    assert_eq!(
        rejected(NewService {
            due_date: Some("2027-02-30".into()),
            ..by_km(0)
        }),
        ServiceRejection::OdometerOutOfRange
    );
    // Date before recurrence.
    assert_eq!(
        rejected(NewService {
            recurrence: every(None, None),
            ..by_date("2027-02-30")
        }),
        ServiceRejection::InvalidDate
    );
    // Mismatch before interval ranges.
    assert_eq!(
        rejected(NewService {
            recurrence: every(Some(0), Some(121)),
            ..by_km(115_000)
        }),
        ServiceRejection::IntervalMismatch
    );
    // Km interval before months interval.
    assert_eq!(
        rejected(NewService {
            due_date: Some("2028-06-01".into()),
            recurrence: every(Some(0), Some(121)),
            ..by_km(180_000)
        }),
        ServiceRejection::IntervalKmOutOfRange
    );
}

// --- Register order ---

fn planned(id: u64, name: &str, km: Option<u32>, date: Option<&str>) -> PlannedService {
    PlannedService {
        id,
        name: name.into(),
        note: None,
        due_odometer_km: km,
        due_date: date.map(Into::into),
        recurrence: None,
    }
}

fn names<'a>(ordered: &[&'a PlannedService]) -> Vec<&'a str> {
    ordered.iter().map(|s| s.name.as_str()).collect()
}

#[test]
fn register_order_dates_first_then_km_ties_by_id() {
    let services = [
        planned(1, "A", None, Some("2027-03-01")),
        planned(2, "B", Some(115_000), None),
        planned(3, "C", Some(180_000), Some("2028-06-01")),
        planned(4, "D", None, Some("2026-10-01")),
        planned(5, "E", Some(115_000), None),
    ];
    assert_eq!(names(&register_order(&services)), ["D", "A", "C", "B", "E"]);

    // Independent of the stored order.
    let mut reversed = services.clone();
    reversed.reverse();
    assert_eq!(names(&register_order(&reversed)), ["D", "A", "C", "B", "E"]);
}

#[test]
fn register_order_same_date_ties_by_id_not_km() {
    let services = [
        planned(9, "late id", Some(1), Some("2027-03-01")),
        planned(2, "early id", Some(500_000), Some("2027-03-01")),
        planned(3, "km only", Some(1), None),
    ];
    assert_eq!(
        names(&register_order(&services)),
        ["early id", "late id", "km only"]
    );
}

#[test]
fn register_order_of_empty_register_is_empty() {
    assert!(register_order(&[]).is_empty());
}

// --- Stored shape ---

#[test]
fn planned_service_round_trips_with_snake_case_keys() {
    let service = PlannedService {
        id: 3,
        name: "Rozrząd".into(),
        note: None,
        due_odometer_km: Some(180_000),
        due_date: Some("2028-06-01".into()),
        recurrence: every(Some(90_000), None),
    };
    let json = serde_json::to_value(&service).unwrap();
    assert_eq!(
        json,
        serde_json::json!({
            "id": 3,
            "name": "Rozrząd",
            "note": null,
            "due_odometer_km": 180000,
            "due_date": "2028-06-01",
            "recurrence": { "every_km": 90000, "every_months": null }
        })
    );
    assert_eq!(
        serde_json::from_value::<PlannedService>(json).unwrap(),
        service
    );
}

#[test]
fn missing_key_is_an_error_even_for_nullable_fields() {
    let full = serde_json::json!({
        "id": 3,
        "name": "Rozrząd",
        "note": null,
        "due_odometer_km": 180000,
        "due_date": null,
        "recurrence": null
    });
    assert!(serde_json::from_value::<PlannedService>(full.clone()).is_ok());
    for key in [
        "id",
        "name",
        "note",
        "due_odometer_km",
        "due_date",
        "recurrence",
    ] {
        let mut value = full.clone();
        value.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<PlannedService>(value).is_err(),
            "missing {key}"
        );
    }

    let recurrence = serde_json::json!({ "every_km": 15000, "every_months": null });
    assert!(serde_json::from_value::<Recurrence>(recurrence.clone()).is_ok());
    for key in ["every_km", "every_months"] {
        let mut value = recurrence.clone();
        value.as_object_mut().unwrap().remove(key);
        assert!(
            serde_json::from_value::<Recurrence>(value).is_err(),
            "missing {key}"
        );
    }
}
