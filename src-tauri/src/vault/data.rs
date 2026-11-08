//! Versioned plaintext stored inside the vault (current format: v3). Later
//! slices extend `VaultData`; a newer `format_version` than this build
//! understands is refused, an older one is migrated in memory step by step
//! (`migrate_v1_to_v2`, then `migrate_v2_to_v3`). A migrated payload reaches
//! the disk only on the next save, preceded by a pre-migration copy of the old
//! file (see `super::UnlockedVault::save`).

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::fuel::{FillUp, StartPoint};
use crate::service::PlannedService;

pub const CURRENT_FORMAT_VERSION: u32 = 3;

/// Payload v3. No `#[serde(default)]`: a v3 payload missing a field is
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
    /// The service register, in no particular order (see
    /// `crate::service::register_order`).
    pub services: Vec<PlannedService>,
    /// Id for the next service; ids are never reused.
    pub next_service_id: u64,
}

impl VaultData {
    pub fn new(created_at: u64) -> Self {
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            created_at,
            start_point: None,
            fill_ups: Vec::new(),
            next_fill_up_id: 1,
            services: Vec::new(),
            next_service_id: 1,
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

/// Payload v2 (app 0.1.4 up to the release before S-05). Frozen: never changes
/// again. Every field is required, as it was when v2 was current.
#[derive(Debug, Deserialize)]
struct VaultDataV2 {
    #[allow(dead_code)]
    format_version: u32,
    created_at: u64,
    #[serde(deserialize_with = "Option::deserialize")]
    start_point: Option<StartPoint>,
    fill_ups: Vec<FillUp>,
    next_fill_up_id: u64,
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
                data: migrate_v2_to_v3(migrate_v1_to_v2(v1)),
                migrated_from: Some(1),
            })
        }
        2 => {
            let v2: VaultDataV2 =
                serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)?;
            Ok(Decoded {
                data: migrate_v2_to_v3(v2),
                migrated_from: Some(2),
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
fn migrate_v1_to_v2(v1: VaultDataV1) -> VaultDataV2 {
    VaultDataV2 {
        format_version: 2,
        created_at: v1.created_at,
        start_point: None,
        fill_ups: Vec::new(),
        next_fill_up_id: 1,
    }
}

/// v3 adds the service register: every v2 field is kept as is, the register
/// starts empty.
fn migrate_v2_to_v3(v2: VaultDataV2) -> VaultData {
    VaultData {
        format_version: 3,
        created_at: v2.created_at,
        start_point: v2.start_point,
        fill_ups: v2.fill_ups,
        next_fill_up_id: v2.next_fill_up_id,
        services: Vec::new(),
        next_service_id: 1,
    }
}

pub fn encode(data: &VaultData) -> Result<Zeroizing<Vec<u8>>, serde_json::Error> {
    serde_json::to_vec(data).map(Zeroizing::new)
}
