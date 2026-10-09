//! Versioned plaintext stored inside the vault. Later slices extend `VaultData`;
//! a newer `format_version` than this build understands is refused.

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

pub const CURRENT_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultData {
    pub format_version: u32,
    /// Vault creation time, unix seconds.
    pub created_at: u64,
}

impl VaultData {
    pub fn new(created_at: u64) -> Self {
        Self {
            format_version: CURRENT_FORMAT_VERSION,
            created_at,
        }
    }
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

pub fn decode(plaintext: &[u8]) -> Result<VaultData, DecodeError> {
    let probe: VersionProbe =
        serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)?;
    if probe.format_version > CURRENT_FORMAT_VERSION {
        return Err(DecodeError::TooNew);
    }
    serde_json::from_slice(plaintext).map_err(|_| DecodeError::Malformed)
}

pub fn encode(data: &VaultData) -> Result<Zeroizing<Vec<u8>>, serde_json::Error> {
    serde_json::to_vec(data).map(Zeroizing::new)
}
