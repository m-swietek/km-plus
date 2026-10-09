mod vault;

use vault::commands::{
    vault_restore_backup, vault_setup, vault_status, vault_unlock, VaultSession,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .manage(VaultSession::default())
        .invoke_handler(tauri::generate_handler![
            vault_status,
            vault_setup,
            vault_unlock,
            vault_restore_backup
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
