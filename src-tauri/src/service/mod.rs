//! Planned services: types stored in the vault, validation of a new or edited
//! entry and the order of the service register.
//!
//! Pure Rust, no Tauri and no IO. A service keeps absolute thresholds (an
//! odometer reading in km and/or a `YYYY-MM-DD` date) and, when recurring, an
//! interval for each threshold that is set. Thresholds are accepted as
//! entered: they are not compared with the start point, the fill-ups or
//! today's date.

pub mod commands;
#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};

use crate::fuel::{is_valid_date, MAX_ODOMETER_KM};

/// Longest name, in Unicode scalar values after trimming.
pub const MAX_NAME_CHARS: usize = 100;
/// Longest note, in Unicode scalar values after trimming.
pub const MAX_NOTE_CHARS: usize = 500;
/// Longest recurrence interval, in months.
pub const MAX_INTERVAL_MONTHS: u32 = 120;

/// Every key is required, `Option` values may be `null` (serde would
/// otherwise treat a missing `Option` key as `None`, hiding damage).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedService {
    pub id: u64,
    pub name: String,
    #[serde(deserialize_with = "Option::deserialize")]
    pub note: Option<String>,
    /// Odometer threshold, in km.
    #[serde(deserialize_with = "Option::deserialize")]
    pub due_odometer_km: Option<u32>,
    /// Date threshold, `YYYY-MM-DD`.
    #[serde(deserialize_with = "Option::deserialize")]
    pub due_date: Option<String>,
    /// `None` for a one-off service.
    #[serde(deserialize_with = "Option::deserialize")]
    pub recurrence: Option<Recurrence>,
}

/// Interval of a recurring service, one per threshold that is set.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Recurrence {
    #[serde(deserialize_with = "Option::deserialize")]
    pub every_km: Option<u32>,
    #[serde(deserialize_with = "Option::deserialize")]
    pub every_months: Option<u32>,
}

/// A service as entered, before it gets an id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewService {
    pub name: String,
    pub note: Option<String>,
    pub due_odometer_km: Option<u32>,
    pub due_date: Option<String>,
    pub recurrence: Option<Recurrence>,
}

impl NewService {
    /// Trims the name and the note; a note left empty becomes `None`.
    pub fn normalize(self) -> Self {
        let note = self
            .note
            .map(|n| n.trim().to_owned())
            .filter(|n| !n.is_empty());
        Self {
            name: self.name.trim().to_owned(),
            note,
            ..self
        }
    }

    pub fn with_id(self, id: u64) -> PlannedService {
        PlannedService {
            id,
            name: self.name,
            note: self.note,
            due_odometer_km: self.due_odometer_km,
            due_date: self.due_date,
            recurrence: self.recurrence,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ServiceRejection {
    NameEmpty,
    NameTooLong,
    NoteTooLong,
    /// Neither an odometer threshold nor a date.
    NoThreshold,
    OdometerOutOfRange,
    InvalidDate,
    /// A recurring service needs an interval for each threshold that is set,
    /// and no interval for a threshold that is not.
    IntervalMismatch,
    IntervalKmOutOfRange,
    IntervalMonthsOutOfRange,
}

/// Checks an entry that has already been through [`NewService::normalize`].
/// The first failing rule is reported, in the order of `ServiceRejection`.
pub fn validate_service(new: &NewService) -> Result<(), ServiceRejection> {
    if new.name.is_empty() {
        return Err(ServiceRejection::NameEmpty);
    }
    if new.name.chars().count() > MAX_NAME_CHARS {
        return Err(ServiceRejection::NameTooLong);
    }
    if new
        .note
        .as_ref()
        .is_some_and(|n| n.chars().count() > MAX_NOTE_CHARS)
    {
        return Err(ServiceRejection::NoteTooLong);
    }
    if new.due_odometer_km.is_none() && new.due_date.is_none() {
        return Err(ServiceRejection::NoThreshold);
    }
    if new
        .due_odometer_km
        .is_some_and(|km| !(1..=MAX_ODOMETER_KM).contains(&km))
    {
        return Err(ServiceRejection::OdometerOutOfRange);
    }
    if new.due_date.as_deref().is_some_and(|d| !is_valid_date(d)) {
        return Err(ServiceRejection::InvalidDate);
    }
    if let Some(r) = &new.recurrence {
        if r.every_km.is_some() != new.due_odometer_km.is_some()
            || r.every_months.is_some() != new.due_date.is_some()
        {
            return Err(ServiceRejection::IntervalMismatch);
        }
        if r.every_km
            .is_some_and(|km| !(1..=MAX_ODOMETER_KM).contains(&km))
        {
            return Err(ServiceRejection::IntervalKmOutOfRange);
        }
        if r.every_months
            .is_some_and(|m| !(1..=MAX_INTERVAL_MONTHS).contains(&m))
        {
            return Err(ServiceRejection::IntervalMonthsOutOfRange);
        }
    }
    Ok(())
}

/// Sort key of the register. Variant order matters: every dated entry comes
/// before every entry without a date.
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum RegisterKey<'a> {
    /// By date, then id (an entry with both thresholds sorts by its date).
    Dated(&'a str, u64),
    /// By odometer threshold, then id.
    Undated(Option<u32>, u64),
}

fn register_key(s: &PlannedService) -> RegisterKey<'_> {
    match &s.due_date {
        Some(date) => RegisterKey::Dated(date, s.id),
        None => RegisterKey::Undated(s.due_odometer_km, s.id),
    }
}

/// The register in display order: entries with a date first, ascending by
/// date, then entries with only an odometer threshold, ascending by km; ties
/// are broken by id.
pub fn register_order(services: &[PlannedService]) -> Vec<&PlannedService> {
    let mut sorted: Vec<&PlannedService> = services.iter().collect();
    sorted.sort_by_key(|s| register_key(*s));
    sorted
}
