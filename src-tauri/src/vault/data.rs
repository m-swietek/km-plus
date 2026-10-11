//! Versioned plaintext stored inside the vault. Later slices extend `VaultData`;
//! a newer `format_version` than this build understands is refused, an older
//! one is migrated in memory step by step (`migrate_v1_to_v2`, ...). A migrated
//! payload reaches the disk only on the next save, preceded by a pre-migration
//! copy of the old file (see `super::UnlockedVault::save`).

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::fuel::{FillUp, StartPoint};

pub const CURRENT_FORMAT_VERSION: u32 = 2;

/// Payload v2. No `#[serde(default)]`: a v2 payload missing a field is
/// damage, not an empty value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultData {
    pub format_version: u32,
    /// Vault creation time, unix seconds.
    pub created_at: u64,
    /// Set once by the user; `null` until then. The key itself is required
    /// (serde would otherwise treat a missing `Option` as `None`).
    #[serde(deserialize_with = "Option::deserialize")]
    pub start_point: Option<StartPoint>,
    pub fill_ups: Vec<FillUp>,
    /// Id for the next fill-up; ids are never reused.
    pub next_fill_up_id: u64,
}

impl VaultData {
    pub fn new(created_at: u64) -> Self {
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            created_at,
            start_point: None,
            fill_ups: Vec::new(),
            next_fill_up_id: 1,
        }
    }
}

/// Payload v1 (S-01, app 0.1.3 and earlier).
#[derive(Debug, Deserialize)]
struct VaultDataV1 {
    #[allow(dead_code)]
    format_version: u32,
    created_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decoded {
    pub data: VaultData,
    /// Format version of the payload when it had to be migrated.
    pub migrated_from: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    /// Written by a newer version of the app.
    TooNew,
    /// Authenticated, but not a payload this build can read.
    Malformed,
}

/// Only the version is read first, so a newer payload is refused before its
/// (possibly incompatible) fields are interpreted.
#[derive(Deserialize)]
struct VersionProbe {
    format_version: u32,
}

pub fn decode(plaintext: &[u8]) -> Result<Decoded, DecodeError> {
    let probe: VersionProbe =
        serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)?;
    match probe.format_version {
        v if v > CURRENT_FORMAT_VERSION => Err(DecodeError::TooNew),
        1 => {
            let v1: VaultDataV1 =
                serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)?;
            Ok(Decoded {
                data: migrate_v1_to_v2(v1),
                migrated_from: Some(1),
            })
        }
        CURRENT_FORMAT_VERSION => Ok(Decoded {
            data: serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)?,
            migrated_from: None,
        }),
        _ => Err(DecodeError::Malformed),
    }
}

/// v1 had no domain data: the vault keeps its creation time and starts empty.
fn migrate_v1_to_v2(v1: VaultDataV1) -> VaultData {
    VaultData {
        format_version: 2,
        created_at: v1.created_at,
        start_point: None,
        fill_ups: Vec::new(),
        next_fill_up_id: 1,
    }
}

pub fn encode(data: &VaultData) -> Result<Zeroizing<Vec<u8>>, serde_json::Error> {
    serde_json::to_vec(data).map(Zeroizing::new)
}
