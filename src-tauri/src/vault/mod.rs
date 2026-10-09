//! Password-unlocked vault: one encrypted file in the app data directory.
//!
//! Pure Rust, no Tauri types — every operation takes the data directory as a
//! parameter so it can be tested on a temporary directory.
//!
//! Files in the data directory:
//! - `kmplus.vault` — main vault
//! - `kmplus.vault.bak` — previous good version (refreshed on every save)
//! - `kmplus.vault.tmp`, `kmplus.vault.bak.tmp` — scratch files of atomic writes
//! - `kmplus.vault.damaged-<unix s>` — damaged main file set aside on restore
//!   (never deleted by the app)

pub mod commands;
mod data;
mod format;
#[cfg(test)]
mod tests;

use std::fmt;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use chacha20poly1305::aead::rand_core::RngCore;
use chacha20poly1305::aead::OsRng;
use zeroize::Zeroizing;

pub use data::VaultData;
pub use format::KdfParams;

use format::{Header, OpenError, KEY_LEN, SALT_LEN};

pub const VAULT_FILE: &str = "kmplus.vault";
pub const BACKUP_FILE: &str = "kmplus.vault.bak";
pub const TMP_FILE: &str = "kmplus.vault.tmp";
pub const BACKUP_TMP_FILE: &str = "kmplus.vault.bak.tmp";
pub const DAMAGED_PREFIX: &str = "kmplus.vault.damaged-";

/// Minimum password length in Unicode scalar values; no trimming is applied.
pub const MIN_PASSWORD_CHARS: usize = 8;

#[derive(Debug, thiserror::Error)]
pub enum VaultError {
    #[error("operacja na pliku danych nie powiodła się: {0}")]
    Io(#[from] io::Error),
    #[error("hasło musi mieć co najmniej {MIN_PASSWORD_CHARS} znaków")]
    PasswordTooShort,
    #[error("plik danych lub jego kopia już istnieje")]
    AlreadyExists,
    #[error("nieprawidłowe parametry wyprowadzania klucza")]
    InvalidKdfParams,
    #[error("nie udało się zserializować danych: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl From<format::KdfError> for VaultError {
    fn from(_: format::KdfError) -> Self {
        VaultError::InvalidKdfParams
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupState {
    /// Neither the main file nor a backup exists.
    NeedsSetup,
    /// The main file exists.
    Locked,
    /// The main file is missing but a backup exists (mtime of the backup).
    MissingWithBackup { backup_saved_at_ms: u64 },
}

#[derive(Debug)]
pub enum UnlockOutcome {
    Unlocked(UnlockedVault),
    WrongPassword,
    DamagedBackupAvailable { backup_saved_at_ms: u64 },
    DamagedNoBackup,
    TooNew,
}

/// An open vault held in memory for the session.
pub struct UnlockedVault {
    dir: PathBuf,
    key: Zeroizing<[u8; KEY_LEN]>,
    salt: [u8; SALT_LEN],
    params: KdfParams,
    pub data: VaultData,
}

impl fmt::Debug for UnlockedVault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("UnlockedVault")
            .field("dir", &self.dir)
            .field("key", &"<redacted>")
            .field("params", &self.params)
            .field("data", &self.data)
            .finish()
    }
}

impl UnlockedVault {
    /// Encrypts the current data under a fresh nonce (same salt and KDF
    /// params) and writes it atomically, rotating the previous main file into
    /// `kmplus.vault.bak` first.
    pub fn save(&mut self) -> Result<(), VaultError> {
        let plaintext = data::encode(&self.data)?;
        let header = Header {
            params: self.params,
            salt: self.salt,
        };
        let sealed = format::seal(&self.key, &header, &plaintext);

        let main = self.dir.join(VAULT_FILE);
        // (1) previous version -> .bak.tmp -> .bak
        if let Some(previous) = read_optional(&main)? {
            let bak_tmp = self.dir.join(BACKUP_TMP_FILE);
            write_synced(&bak_tmp, &previous)?;
            fs::rename(&bak_tmp, self.dir.join(BACKUP_FILE))?;
        }
        // (2) new version -> .tmp -> main
        let tmp = self.dir.join(TMP_FILE);
        write_synced(&tmp, &sealed)?;
        fs::rename(&tmp, &main)?;
        Ok(())
    }
}

/// Startup state of the data directory; reads metadata only.
pub fn probe(dir: &Path) -> Result<StartupState, VaultError> {
    if exists(&dir.join(VAULT_FILE))? {
        return Ok(StartupState::Locked);
    }
    match backup_saved_at_ms(dir)? {
        Some(backup_saved_at_ms) => Ok(StartupState::MissingWithBackup { backup_saved_at_ms }),
        None => Ok(StartupState::NeedsSetup),
    }
}

/// Creates a new vault. Refuses when a main file or a backup already exists so
/// existing history can never be replaced by an empty vault.
pub fn create(dir: &Path, password: &str, params: KdfParams) -> Result<UnlockedVault, VaultError> {
    if password.chars().count() < MIN_PASSWORD_CHARS {
        return Err(VaultError::PasswordTooShort);
    }
    if !params.is_valid() {
        return Err(VaultError::InvalidKdfParams);
    }
    if exists(&dir.join(VAULT_FILE))? || exists(&dir.join(BACKUP_FILE))? {
        return Err(VaultError::AlreadyExists);
    }
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    let key = format::derive_key(password, &salt, &params)?;
    let mut vault = UnlockedVault {
        dir: dir.to_path_buf(),
        key,
        salt,
        params,
        data: VaultData::new(now_unix_secs()),
    };
    vault.save()?;
    Ok(vault)
}

/// Opens the vault. Read-only: never writes anything, whatever the outcome.
pub fn unlock(dir: &Path, password: &str) -> Result<UnlockOutcome, VaultError> {
    let main = read_optional(&dir.join(VAULT_FILE))?;
    let main_attempt = main.as_deref().map(|bytes| try_open(dir, bytes, password));

    match main_attempt {
        Some(Attempt::Opened(vault)) => Ok(UnlockOutcome::Unlocked(vault)),
        Some(Attempt::TooNew) => Ok(UnlockOutcome::TooNew),
        // Header fine, tag mismatch: wrong password or damage. Only a backup
        // that opens with the same password proves damage.
        Some(Attempt::AuthFailed) => Ok(match check_backup(dir, password)? {
            Some((Attempt::Opened(_), backup_saved_at_ms)) => {
                UnlockOutcome::DamagedBackupAvailable { backup_saved_at_ms }
            }
            _ => UnlockOutcome::WrongPassword,
        }),
        // Main file certainly damaged (or missing).
        Some(Attempt::Malformed) | None => Ok(match check_backup(dir, password)? {
            None => UnlockOutcome::DamagedNoBackup,
            Some((Attempt::Opened(_), backup_saved_at_ms)) => {
                UnlockOutcome::DamagedBackupAvailable { backup_saved_at_ms }
            }
            Some((Attempt::AuthFailed, _)) => UnlockOutcome::WrongPassword,
            Some((Attempt::TooNew, _)) => UnlockOutcome::TooNew,
            Some((Attempt::Malformed, _)) => UnlockOutcome::DamagedNoBackup,
        }),
    }
}

/// Restores the main file from `kmplus.vault.bak` after the user confirmed.
/// The backup is decrypted first; on failure nothing on disk changes. On
/// success an existing main file is set aside as `kmplus.vault.damaged-<s>`
/// (never deleted), the backup is copied into place and stays as it is.
pub fn restore_backup(dir: &Path, password: &str) -> Result<UnlockOutcome, VaultError> {
    let Some(backup) = read_optional(&dir.join(BACKUP_FILE))? else {
        return Ok(UnlockOutcome::DamagedNoBackup);
    };
    let vault = match try_open(dir, &backup, password) {
        Attempt::Opened(vault) => vault,
        Attempt::AuthFailed => return Ok(UnlockOutcome::WrongPassword),
        Attempt::TooNew => return Ok(UnlockOutcome::TooNew),
        Attempt::Malformed => return Ok(UnlockOutcome::DamagedNoBackup),
    };

    let main = dir.join(VAULT_FILE);
    if exists(&main)? {
        fs::rename(&main, damaged_path(dir)?)?;
    }
    let tmp = dir.join(TMP_FILE);
    write_synced(&tmp, &backup)?;
    fs::rename(&tmp, &main)?;
    Ok(UnlockOutcome::Unlocked(vault))
}

enum Attempt {
    Opened(UnlockedVault),
    Malformed,
    AuthFailed,
    TooNew,
}

fn try_open(dir: &Path, bytes: &[u8], password: &str) -> Attempt {
    let header = match format::parse_header(bytes) {
        Ok((header, _)) => header,
        Err(OpenError::UnsupportedEnvelope) => return Attempt::TooNew,
        Err(_) => return Attempt::Malformed,
    };
    let Ok(key) = format::derive_key(password, &header.salt, &header.params) else {
        return Attempt::Malformed;
    };
    let plaintext = match format::open(&key, bytes) {
        Ok(plaintext) => plaintext,
        Err(OpenError::AuthFailed) => return Attempt::AuthFailed,
        Err(OpenError::UnsupportedEnvelope) => return Attempt::TooNew,
        Err(OpenError::Malformed) => return Attempt::Malformed,
    };
    match data::decode(&plaintext) {
        Ok(data) => Attempt::Opened(UnlockedVault {
            dir: dir.to_path_buf(),
            key,
            salt: header.salt,
            params: header.params,
            data,
        }),
        Err(data::DecodeError::TooNew) => Attempt::TooNew,
        Err(data::DecodeError::Malformed) => Attempt::Malformed,
    }
}

/// Tries the backup with the given password; `None` when there is no backup.
fn check_backup(dir: &Path, password: &str) -> Result<Option<(Attempt, u64)>, VaultError> {
    let Some(bytes) = read_optional(&dir.join(BACKUP_FILE))? else {
        return Ok(None);
    };
    let saved_at = mtime_ms(&dir.join(BACKUP_FILE))?;
    Ok(Some((try_open(dir, &bytes, password), saved_at)))
}

fn backup_saved_at_ms(dir: &Path) -> Result<Option<u64>, VaultError> {
    let path = dir.join(BACKUP_FILE);
    if exists(&path)? {
        Ok(Some(mtime_ms(&path)?))
    } else {
        Ok(None)
    }
}

fn damaged_path(dir: &Path) -> Result<PathBuf, VaultError> {
    let base = format!("{DAMAGED_PREFIX}{}", now_unix_secs());
    let mut candidate = dir.join(&base);
    let mut n = 1u32;
    while exists(&candidate)? {
        candidate = dir.join(format!("{base}-{n}"));
        n += 1;
    }
    Ok(candidate)
}

fn read_optional(path: &Path) -> io::Result<Option<Vec<u8>>> {
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e),
    }
}

fn exists(path: &Path) -> io::Result<bool> {
    path.try_exists()
}

fn write_synced(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = File::create(path)?;
    file.write_all(bytes)?;
    file.sync_all()
}

fn mtime_ms(path: &Path) -> io::Result<u64> {
    let modified = fs::metadata(path)?.modified()?;
    Ok(modified
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0))
}

fn now_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
