//! Tauri command layer over the vault: session state, data directory and the
//! four commands the frontend calls. Key derivation runs on a blocking thread
//! so the window never freezes; errors are readable Polish messages that never
//! contain the password.

use std::path::PathBuf;
use std::sync::Mutex;

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

use super::{KdfParams, StartupState, UnlockOutcome, UnlockedVault, VaultError};

/// The unlocked vault, held in memory until the app closes.
#[derive(Default)]
pub struct VaultSession(Mutex<Option<UnlockedVault>>);

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum VaultStatus {
    NeedsSetup,
    Locked,
    MissingWithBackup { backup_saved_at: u64 },
    Unlocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum SetupResult {
    Ok,
    PasswordTooShort,
    AlreadyExists,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum UnlockResult {
    Unlocked,
    WrongPassword,
    DamagedBackupAvailable { backup_saved_at: u64 },
    DamagedNoBackup,
    TooNew,
}

/// Startup state of the vault; `unlocked` when this session already holds it
/// (e.g. after a webview reload).
#[tauri::command]
pub async fn vault_status(
    app: AppHandle,
    session: State<'_, VaultSession>,
) -> Result<VaultStatus, String> {
    if lock_session(&session)?.is_some() {
        return Ok(VaultStatus::Unlocked);
    }
    let dir = data_dir(&app)?;
    let state = super::probe(&dir).map_err(error_message)?;
    Ok(match state {
        StartupState::NeedsSetup => VaultStatus::NeedsSetup,
        StartupState::Locked => VaultStatus::Locked,
        StartupState::MissingWithBackup { backup_saved_at_ms } => VaultStatus::MissingWithBackup {
            backup_saved_at: backup_saved_at_ms,
        },
    })
}

/// Creates the vault with the given password and unlocks the session.
#[tauri::command]
pub async fn vault_setup(
    app: AppHandle,
    session: State<'_, VaultSession>,
    password: String,
) -> Result<SetupResult, String> {
    let password = Zeroizing::new(password);
    let dir = data_dir(&app)?;
    let created = tauri::async_runtime::spawn_blocking(move || {
        super::create(&dir, &password, KdfParams::default())
    })
    .await
    .map_err(|_| INTERNAL_ERROR.to_string())?;

    match created {
        Ok(vault) => {
            *lock_session(&session)? = Some(vault);
            Ok(SetupResult::Ok)
        }
        Err(VaultError::PasswordTooShort) => Ok(SetupResult::PasswordTooShort),
        Err(VaultError::AlreadyExists) => Ok(SetupResult::AlreadyExists),
        Err(e) => Err(error_message(e)),
    }
}

/// Opens the vault; read-only on every outcome.
#[tauri::command]
pub async fn vault_unlock(
    app: AppHandle,
    session: State<'_, VaultSession>,
    password: String,
) -> Result<UnlockResult, String> {
    run_unlock(&app, &session, password, super::unlock).await
}

/// Restores the main file from the backup (after user confirmation) and
/// unlocks the session. Used both after `damagedBackupAvailable` and for
/// `missingWithBackup`.
#[tauri::command]
pub async fn vault_restore_backup(
    app: AppHandle,
    session: State<'_, VaultSession>,
    password: String,
) -> Result<UnlockResult, String> {
    run_unlock(&app, &session, password, super::restore_backup).await
}

const INTERNAL_ERROR: &str = "wewnętrzny błąd podczas operacji na danych";

async fn run_unlock(
    app: &AppHandle,
    session: &VaultSession,
    password: String,
    op: fn(&std::path::Path, &str) -> Result<UnlockOutcome, VaultError>,
) -> Result<UnlockResult, String> {
    let password = Zeroizing::new(password);
    let dir = data_dir(app)?;
    let outcome = tauri::async_runtime::spawn_blocking(move || op(&dir, &password))
        .await
        .map_err(|_| INTERNAL_ERROR.to_string())?
        .map_err(error_message)?;

    Ok(match outcome {
        UnlockOutcome::Unlocked(vault) => {
            *lock_session(session)? = Some(vault);
            UnlockResult::Unlocked
        }
        UnlockOutcome::WrongPassword => UnlockResult::WrongPassword,
        UnlockOutcome::DamagedBackupAvailable { backup_saved_at_ms } => {
            UnlockResult::DamagedBackupAvailable {
                backup_saved_at: backup_saved_at_ms,
            }
        }
        UnlockOutcome::DamagedNoBackup => UnlockResult::DamagedNoBackup,
        UnlockOutcome::TooNew => UnlockResult::TooNew,
    })
}

/// App data directory (`%APPDATA%\io.github.m-swietek.kmplus` on Windows),
/// created when missing.
fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("nie udało się ustalić katalogu danych: {e}"))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("nie udało się utworzyć katalogu danych: {e}"))?;
    Ok(dir)
}

fn lock_session(
    session: &VaultSession,
) -> Result<std::sync::MutexGuard<'_, Option<UnlockedVault>>, String> {
    session
        .0
        .lock()
        .map_err(|_| "stan sesji jest niespójny; uruchom aplikację ponownie".to_string())
}

fn error_message(e: VaultError) -> String {
    e.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, to_value};

    #[test]
    fn vault_status_serializes_to_frontend_shape() {
        assert_eq!(to_value(VaultStatus::NeedsSetup).unwrap(), json!({"kind": "needsSetup"}));
        assert_eq!(to_value(VaultStatus::Locked).unwrap(), json!({"kind": "locked"}));
        assert_eq!(
            to_value(VaultStatus::MissingWithBackup { backup_saved_at: 1_700_000_000_123 }).unwrap(),
            json!({"kind": "missingWithBackup", "backupSavedAt": 1_700_000_000_123u64})
        );
        assert_eq!(to_value(VaultStatus::Unlocked).unwrap(), json!({"kind": "unlocked"}));
    }

    #[test]
    fn setup_result_serializes_to_frontend_shape() {
        assert_eq!(to_value(SetupResult::Ok).unwrap(), json!({"kind": "ok"}));
        assert_eq!(
            to_value(SetupResult::PasswordTooShort).unwrap(),
            json!({"kind": "passwordTooShort"})
        );
        assert_eq!(
            to_value(SetupResult::AlreadyExists).unwrap(),
            json!({"kind": "alreadyExists"})
        );
    }

    #[test]
    fn unlock_result_serializes_to_frontend_shape() {
        assert_eq!(to_value(UnlockResult::Unlocked).unwrap(), json!({"kind": "unlocked"}));
        assert_eq!(
            to_value(UnlockResult::WrongPassword).unwrap(),
            json!({"kind": "wrongPassword"})
        );
        assert_eq!(
            to_value(UnlockResult::DamagedBackupAvailable { backup_saved_at: 1_700_000_000_123 })
                .unwrap(),
            json!({"kind": "damagedBackupAvailable", "backupSavedAt": 1_700_000_000_123u64})
        );
        assert_eq!(
            to_value(UnlockResult::DamagedNoBackup).unwrap(),
            json!({"kind": "damagedNoBackup"})
        );
        assert_eq!(to_value(UnlockResult::TooNew).unwrap(), json!({"kind": "tooNew"}));
    }
}
