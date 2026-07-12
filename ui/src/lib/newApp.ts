/**
 * New Application Wizard - Tauri IPC wrappers.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/** A single live progress line emitted while scaffolding. */
export interface NewAppProgress {
  /** Human-readable step description. */
  message: string;
  /** Whether this line represents a failure. */
  error: boolean;
}

const PROGRESS_EVENT = "new-app://progress";

/**
 * Subscribe to scaffolding progress. Call the returned function to unsubscribe.
 */
export async function listenNewAppProgress(
  handler: (progress: NewAppProgress) => void
): Promise<UnlistenFn> {
  return listen<NewAppProgress>(PROGRESS_EVENT, (event) => handler(event.payload));
}

/** Application type for scaffolding. */
export type AppType =
  | "gui"
  | "tui"
  | "cli"
  | "system"
  | "api-server"
  | "api-server-web";

/** Per-app-type crate selection. */
export interface CrateProfile {
  /** Always included (shown locked in the wizard). */
  required: string[];
  /** Checked by default, user can uncheck. */
  recommended: string[];
  /** Every other core crate, unchecked by default. */
  optional: string[];
}

/** Request to scaffold a new application. */
export interface ScaffoldRequest {
  /** Application name. */
  name: string;
  /** Type of application to create. */
  appType: AppType;
  /** Selected Nest crates to include. */
  selectedCrates: string[];
}

/** Response from scaffolding operation. */
export interface ScaffoldResponse {
  /** Whether the operation succeeded. */
  success: boolean;
  /** Human-readable message. */
  message: string;
  /** Path to the created application. */
  appPath?: string;
}

/**
 * Get the list of available core crates.
 */
export async function newAppListCrates(): Promise<string[]> {
  return invoke<string[]>("plugin:new-app|new_app_list_crates");
}

/**
 * Get the crate profile (required/recommended/optional) for an app type.
 */
export async function newAppCrateProfile(appType: AppType): Promise<CrateProfile> {
  return invoke<CrateProfile>("plugin:new-app|new_app_crate_profile", { appType });
}

/**
 * Scaffold a new application.
 */
export async function newAppScaffold(request: ScaffoldRequest): Promise<ScaffoldResponse> {
  return invoke<ScaffoldResponse>("plugin:new-app|new_app_scaffold", { request });
}

/**
 * Scaffold and build a new application.
 */
export async function newAppBuild(request: ScaffoldRequest): Promise<ScaffoldResponse> {
  return invoke<ScaffoldResponse>("plugin:new-app|new_app_build", { request });
}
