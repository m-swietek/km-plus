// Typed wrappers over the vault commands (src-tauri/src/vault/commands.rs).
// Shapes are registered in docs/reference/contract-surfaces.md; times are epoch milliseconds.
import { invoke } from "@tauri-apps/api/core";

export type VaultStatus =
  | { kind: "needsSetup" }
  | { kind: "locked" }
  | { kind: "missingWithBackup"; backupSavedAt: number }
  | { kind: "unlocked" };

export type SetupResult =
  | { kind: "ok" }
  | { kind: "passwordTooShort" }
  | { kind: "alreadyExists" };

export type UnlockResult =
  | { kind: "unlocked" }
  | { kind: "wrongPassword" }
  | { kind: "damagedBackupAvailable"; backupSavedAt: number }
  | { kind: "damagedNoBackup" }
  | { kind: "tooNew" };

/** Minimum password length, counted in Unicode code points like the backend (`chars().count()`). */
export const MIN_PASSWORD_LENGTH = 8;

export function passwordLength(password: string): number {
  return [...password].length;
}

export function vaultStatus(): Promise<VaultStatus> {
  return invoke<VaultStatus>("vault_status");
}

export function vaultSetup(password: string): Promise<SetupResult> {
  return invoke<SetupResult>("vault_setup", { password });
}

export function vaultUnlock(password: string): Promise<UnlockResult> {
  return invoke<UnlockResult>("vault_unlock", { password });
}

export function vaultRestoreBackup(password: string): Promise<UnlockResult> {
  return invoke<UnlockResult>("vault_restore_backup", { password });
}

/** Commands reject with a Polish message string; anything else is stringified. */
export function errorMessage(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : String(error);
}

export function formatBackupDate(ms: number): string {
  return new Date(ms).toLocaleString("pl-PL");
}
