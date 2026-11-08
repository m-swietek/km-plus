mod fuel;
// Used only by the vault payload and tests until the service Tauri commands
// land (Phase 2 of plan-service); drop the allow then.
#[allow(dead_code)]
mod service;
mod vault;

use fuel::commands::{
    add_fill_up, update_fill_up, delete_fill_up, fuel_overview, set_start_point,
};
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
            vault_restore_backup,
            fuel_overview,
            set_start_point,
            add_fill_up,
            update_fill_up,
            delete_fill_up
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
