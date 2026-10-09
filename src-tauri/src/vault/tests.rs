use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

use tempfile::TempDir;

use super::format::{self, Header, OpenError, HEADER_LEN, NONCE_LEN, SALT_LEN};
use super::*;

const PASSWORD: &str = "correct horse";
const WRONG: &str = "wrong horse!";

/// Cheap Argon2id parameters so tests stay fast.
fn cheap() -> KdfParams {
    KdfParams {
        m_cost: 8,
        t_cost: 1,
        p_cost: 1,
    }
}

fn new_vault(dir: &Path) -> UnlockedVault {
    create(dir, PASSWORD, cheap()).expect("create vault")
}

/// Vault with a main file and a `.bak` holding the previous version.
fn vault_with_backup(dir: &Path) -> UnlockedVault {
    let mut vault = new_vault(dir);
    vault.data.created_at = 1;
    vault.save().expect("second save");
    vault
}

type Snapshot = BTreeMap<String, (Vec<u8>, SystemTime)>;

fn snapshot(dir: &Path) -> Snapshot {
    fs::read_dir(dir)
        .unwrap()
        .map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            (
                entry.file_name().to_string_lossy().into_owned(),
                (
                    fs::read(&path).unwrap(),
                    fs::metadata(&path).unwrap().modified().unwrap(),
                ),
            )
        })
        .collect()
}

fn main_path(dir: &Path) -> std::path::PathBuf {
    dir.join(VAULT_FILE)
}

fn flip_byte(path: &Path, index_from_end: usize) {
    let mut bytes = fs::read(path).unwrap();
    let i = bytes.len() - 1 - index_from_end;
    bytes[i] ^= 0x01;
    fs::write(path, bytes).unwrap();
}

fn expect_unlocked(outcome: UnlockOutcome) -> UnlockedVault {
    match outcome {
        UnlockOutcome::Unlocked(vault) => vault,
        other => panic!("expected Unlocked, got {other:?}"),
    }
}

/// Decrypts arbitrary vault file bytes with the password (bypassing the file
/// layout), for checking what a `.bak` contains.
fn decrypt_file(dir: &Path, bytes: &[u8], password: &str) -> VaultData {
    match try_open(dir, bytes, password) {
        Attempt::Opened(vault) => vault.data,
        _ => panic!("file did not open"),
    }
}

#[test]
fn create_then_unlock_roundtrip() {
    let dir = TempDir::new().unwrap();
    let created = new_vault(dir.path());
    assert_eq!(created.data.format_version, data::CURRENT_FORMAT_VERSION);
    assert!(created.data.created_at > 0);

    let unlocked = expect_unlocked(unlock(dir.path(), PASSWORD).unwrap());
    assert_eq!(unlocked.data, created.data);
    assert_eq!(unlocked.params, cheap());
    assert_eq!(unlocked.salt, created.salt);
}

#[test]
fn wrong_password_returns_wrong_password_and_leaves_files_untouched() {
    let dir = TempDir::new().unwrap();
    vault_with_backup(dir.path());
    let before = snapshot(dir.path());

    let outcome = unlock(dir.path(), WRONG).unwrap();
    assert!(matches!(outcome, UnlockOutcome::WrongPassword), "{outcome:?}");
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn password_shorter_than_8_chars_rejected() {
    let dir = TempDir::new().unwrap();

    for short in ["", "1234567", "ąęółżźć"] {
        assert!(matches!(
            create(dir.path(), short, cheap()),
            Err(VaultError::PasswordTooShort)
        ));
    }
    // Nothing was written for a rejected password.
    assert!(snapshot(dir.path()).is_empty());

    // 8 multi-byte characters pass.
    let multibyte = "ąęółżźćń";
    create(dir.path(), multibyte, cheap()).expect("8 chars accepted");
    expect_unlocked(unlock(dir.path(), multibyte).unwrap());

    // Spaces count and are not trimmed: 7 visible chars + a trailing space is
    // 8 chars, and the trimmed variant is a different (wrong) password.
    let dir2 = TempDir::new().unwrap();
    let spaced = "1234567 ";
    create(dir2.path(), spaced, cheap()).expect("space counts as a char");
    expect_unlocked(unlock(dir2.path(), spaced).unwrap());
    assert!(matches!(
        unlock(dir2.path(), spaced.trim()).unwrap(),
        UnlockOutcome::WrongPassword
    ));
}

#[test]
fn create_refuses_when_vault_or_backup_exists() {
    // Main file exists.
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    let before = snapshot(dir.path());
    assert!(matches!(
        create(dir.path(), PASSWORD, cheap()),
        Err(VaultError::AlreadyExists)
    ));
    assert_eq!(snapshot(dir.path()), before);

    // Only a backup exists.
    let dir = TempDir::new().unwrap();
    fs::write(dir.path().join(BACKUP_FILE), b"some backup").unwrap();
    let before = snapshot(dir.path());
    assert!(matches!(
        create(dir.path(), PASSWORD, cheap()),
        Err(VaultError::AlreadyExists)
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn file_contains_no_plaintext() {
    fn contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack.windows(needle.len()).any(|w| w == needle)
    }

    let dir = TempDir::new().unwrap();
    let vault = new_vault(dir.path());
    let bytes = fs::read(main_path(dir.path())).unwrap();
    assert!(!contains(&bytes, b"format_version"));
    assert!(!contains(&bytes, b"created_at"));
    assert!(!contains(&bytes, PASSWORD.as_bytes()));

    let marker = b"KMPLUS-PLAINTEXT-MARKER-1234567890";
    let header = Header {
        params: cheap(),
        salt: vault.salt,
    };
    let sealed = format::seal(&vault.key, &header, marker);
    assert!(!contains(&sealed, marker));
    assert_eq!(&format::open(&vault.key, &sealed).unwrap()[..], &marker[..]);
}

#[test]
fn tampered_ciphertext_byte_fails() {
    let dir = TempDir::new().unwrap();
    let vault = new_vault(dir.path());
    flip_byte(&main_path(dir.path()), 0);

    let bytes = fs::read(main_path(dir.path())).unwrap();
    assert_eq!(format::open(&vault.key, &bytes), Err(OpenError::AuthFailed));
    // No backup: indistinguishable from a wrong password.
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::WrongPassword
    ));
}

#[test]
fn tampered_header_byte_fails() {
    let dir = TempDir::new().unwrap();
    let vault = new_vault(dir.path());
    let mut bytes = fs::read(main_path(dir.path())).unwrap();

    // t_cost 1 -> 2 stays within bounds; decrypting with the original key
    // must still fail because the header is authenticated (AAD).
    assert_eq!(bytes[11], 1);
    bytes[11] = 2;
    assert!(format::parse_header(&bytes).is_ok());
    assert_eq!(format::open(&vault.key, &bytes), Err(OpenError::AuthFailed));

    // A changed salt byte also fails through the whole unlock path.
    let mut bytes = fs::read(main_path(dir.path())).unwrap();
    bytes[19] ^= 0xFF;
    fs::write(main_path(dir.path()), &bytes).unwrap();
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::WrongPassword
    ));
}

#[test]
fn truncated_file_is_malformed() {
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    let bytes = fs::read(main_path(dir.path())).unwrap();

    for len in [0, 3, 30, HEADER_LEN, HEADER_LEN + 15] {
        assert_eq!(
            format::parse_header(&bytes[..len]).map(|_| ()),
            Err(OpenError::Malformed),
            "len {len}"
        );
    }

    fs::write(main_path(dir.path()), &bytes[..30]).unwrap();
    let before = snapshot(dir.path());
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedNoBackup
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn bad_magic_is_malformed() {
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    let mut bytes = fs::read(main_path(dir.path())).unwrap();
    bytes[0] = b'X';
    assert_eq!(
        format::parse_header(&bytes).map(|_| ()),
        Err(OpenError::Malformed)
    );

    fs::write(main_path(dir.path()), &bytes).unwrap();
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedNoBackup
    ));
}

#[test]
fn absurd_kdf_params_are_malformed() {
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    let original = fs::read(main_path(dir.path())).unwrap();

    // (offset, value): m_cost huge, m_cost too small, t_cost 0 and huge,
    // p_cost 0 and huge, unknown kdf id.
    let cases: [(usize, u32); 6] = [
        (7, u32::MAX),
        (7, 1),
        (11, 0),
        (11, 1_000),
        (15, 0),
        (15, 64),
    ];
    for (offset, value) in cases {
        let mut bytes = original.clone();
        bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        assert_eq!(
            format::parse_header(&bytes).map(|_| ()),
            Err(OpenError::Malformed),
            "offset {offset} value {value}"
        );
    }
    let mut bytes = original.clone();
    bytes[6] = 2;
    assert_eq!(
        format::parse_header(&bytes).map(|_| ()),
        Err(OpenError::Malformed)
    );

    let mut bytes = original.clone();
    bytes[7..11].copy_from_slice(&u32::MAX.to_le_bytes());
    fs::write(main_path(dir.path()), &bytes).unwrap();
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedNoBackup
    ));
}

#[test]
fn save_rotates_previous_version_into_bak() {
    let dir = TempDir::new().unwrap();
    let mut vault = new_vault(dir.path());
    assert!(!dir.path().join(BACKUP_FILE).exists());

    vault.data.created_at = 111;
    vault.save().unwrap();
    vault.data.created_at = 222;
    vault.save().unwrap();

    let bak = fs::read(dir.path().join(BACKUP_FILE)).unwrap();
    assert_eq!(decrypt_file(dir.path(), &bak, PASSWORD).created_at, 111);
    let main = expect_unlocked(unlock(dir.path(), PASSWORD).unwrap());
    assert_eq!(main.data.created_at, 222);

    // Scratch files are renamed away.
    assert!(!dir.path().join(TMP_FILE).exists());
    assert!(!dir.path().join(BACKUP_TMP_FILE).exists());
}

#[test]
fn each_save_uses_fresh_nonce() {
    let dir = TempDir::new().unwrap();
    let mut vault = new_vault(dir.path());
    let first = fs::read(main_path(dir.path())).unwrap();
    vault.save().unwrap();
    let second = fs::read(main_path(dir.path())).unwrap();

    let nonce = HEADER_LEN - NONCE_LEN..HEADER_LEN;
    let salt = 19..19 + SALT_LEN;
    assert_ne!(first[nonce.clone()], second[nonce]);
    assert_eq!(first[salt.clone()], second[salt]);
    assert_ne!(first[HEADER_LEN..], second[HEADER_LEN..]);
}

#[test]
fn damaged_main_with_good_backup_reports_backup_available() {
    let dir = TempDir::new().unwrap();
    vault_with_backup(dir.path());
    flip_byte(&main_path(dir.path()), 0);
    let expected_ms = mtime_ms(&dir.path().join(BACKUP_FILE)).unwrap();
    let before = snapshot(dir.path());

    match unlock(dir.path(), PASSWORD).unwrap() {
        UnlockOutcome::DamagedBackupAvailable { backup_saved_at_ms } => {
            assert_eq!(backup_saved_at_ms, expected_ms);
            assert!(backup_saved_at_ms > 0);
        }
        other => panic!("expected DamagedBackupAvailable, got {other:?}"),
    }
    assert_eq!(snapshot(dir.path()), before);

    // A malformed (not just tampered) main behaves the same with a good backup.
    fs::write(main_path(dir.path()), b"garbage").unwrap();
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedBackupAvailable { .. }
    ));
}

#[test]
fn wrong_password_with_backup_present_reports_wrong_password() {
    let dir = TempDir::new().unwrap();
    vault_with_backup(dir.path());
    flip_byte(&main_path(dir.path()), 0);
    let before = snapshot(dir.path());

    assert!(matches!(
        unlock(dir.path(), WRONG).unwrap(),
        UnlockOutcome::WrongPassword
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn malformed_main_without_backup_reports_damaged_no_backup() {
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    fs::write(main_path(dir.path()), b"definitely not a vault file").unwrap();
    let before = snapshot(dir.path());

    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedNoBackup
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn restore_backup_sets_aside_damaged_main_and_unlocks() {
    let dir = TempDir::new().unwrap();
    vault_with_backup(dir.path());
    flip_byte(&main_path(dir.path()), 0);
    let damaged = fs::read(main_path(dir.path())).unwrap();
    let backup = fs::read(dir.path().join(BACKUP_FILE)).unwrap();

    let restored = expect_unlocked(restore_backup(dir.path(), PASSWORD).unwrap());
    assert_eq!(restored.data, decrypt_file(dir.path(), &backup, PASSWORD));

    // Damaged main set aside, not deleted.
    let set_aside: Vec<_> = fs::read_dir(dir.path())
        .unwrap()
        .map(|e| e.unwrap())
        .filter(|e| e.file_name().to_string_lossy().starts_with(DAMAGED_PREFIX))
        .collect();
    assert_eq!(set_aside.len(), 1);
    assert_eq!(fs::read(set_aside[0].path()).unwrap(), damaged);

    // Backup stays, main is a copy of it, and the vault unlocks normally.
    assert_eq!(fs::read(dir.path().join(BACKUP_FILE)).unwrap(), backup);
    assert_eq!(fs::read(main_path(dir.path())).unwrap(), backup);
    assert!(!dir.path().join(TMP_FILE).exists());
    expect_unlocked(unlock(dir.path(), PASSWORD).unwrap());

    // Missing main with a backup: restore just puts the backup in place.
    fs::remove_file(main_path(dir.path())).unwrap();
    expect_unlocked(restore_backup(dir.path(), PASSWORD).unwrap());
    assert_eq!(fs::read(main_path(dir.path())).unwrap(), backup);
}

#[test]
fn restore_backup_with_wrong_password_changes_nothing() {
    let dir = TempDir::new().unwrap();
    vault_with_backup(dir.path());
    flip_byte(&main_path(dir.path()), 0);
    let before = snapshot(dir.path());

    assert!(matches!(
        restore_backup(dir.path(), WRONG).unwrap(),
        UnlockOutcome::WrongPassword
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn probe_states() {
    let dir = TempDir::new().unwrap();
    assert_eq!(probe(dir.path()).unwrap(), StartupState::NeedsSetup);

    vault_with_backup(dir.path());
    assert_eq!(probe(dir.path()).unwrap(), StartupState::Locked);

    let expected_ms = mtime_ms(&dir.path().join(BACKUP_FILE)).unwrap();
    fs::remove_file(main_path(dir.path())).unwrap();
    assert_eq!(
        probe(dir.path()).unwrap(),
        StartupState::MissingWithBackup {
            backup_saved_at_ms: expected_ms
        }
    );

    // Missing main + backup: unlock offers the backup instead of setup.
    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::DamagedBackupAvailable { .. }
    ));
}

#[test]
fn newer_envelope_version_is_too_new() {
    let dir = TempDir::new().unwrap();
    new_vault(dir.path());
    let mut bytes = fs::read(main_path(dir.path())).unwrap();
    bytes[4..6].copy_from_slice(&2u16.to_le_bytes());
    assert_eq!(
        format::parse_header(&bytes).map(|_| ()),
        Err(OpenError::UnsupportedEnvelope)
    );
    fs::write(main_path(dir.path()), &bytes).unwrap();
    let before = snapshot(dir.path());

    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::TooNew
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn newer_format_version_is_too_new() {
    let payload = br#"{"format_version":2,"created_at":"not a number","new_field":[1,2]}"#;
    assert_eq!(data::decode(payload), Err(data::DecodeError::TooNew));

    let dir = TempDir::new().unwrap();
    let vault = new_vault(dir.path());
    let header = Header {
        params: vault.params,
        salt: vault.salt,
    };
    fs::write(
        main_path(dir.path()),
        format::seal(&vault.key, &header, payload),
    )
    .unwrap();
    let before = snapshot(dir.path());

    assert!(matches!(
        unlock(dir.path(), PASSWORD).unwrap(),
        UnlockOutcome::TooNew
    ));
    assert_eq!(snapshot(dir.path()), before);
}

#[test]
fn default_kdf_params_written_to_header() {
    let dir = TempDir::new().unwrap();
    create(dir.path(), PASSWORD, KdfParams::default()).unwrap();
    let bytes = fs::read(main_path(dir.path())).unwrap();

    assert_eq!(&bytes[0..4], b"KMPV");
    assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 1);
    assert_eq!(bytes[6], 1);
    let u32_at = |o: usize| u32::from_le_bytes(bytes[o..o + 4].try_into().unwrap());
    assert_eq!(u32_at(7), 65536);
    assert_eq!(u32_at(11), 3);
    assert_eq!(u32_at(15), 1);

    let (header, _) = format::parse_header(&bytes).unwrap();
    assert_eq!(header.params, KdfParams::default());
    expect_unlocked(unlock(dir.path(), PASSWORD).unwrap());
}
