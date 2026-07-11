import { invoke } from "@tauri-apps/api/core";
import { kerr, klog } from "./log";

/** Invokes a `plugin:kiwi|<command>` with enter/ok/err logging. */
export async function kiwiInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  klog("ipc", `kiwi|${command} invoke`, args ?? {});
  try {
    const result = await invoke<T>(`plugin:kiwi|${command}`, args ?? {});
    klog("ipc", `kiwi|${command} ok`, result);
    return result;
  } catch (error) {
    kerr("ipc", `kiwi|${command} err`, error);
    throw error;
  }
}

/** Invokes a core nest command with enter/ok/err logging. */
export async function nestInvoke<T>(
  command: string,
  args?: Record<string, unknown>,
): Promise<T> {
  klog("ipc", `${command} invoke`, args ?? {});
  try {
    const result = await invoke<T>(command, args ?? {});
    klog("ipc", `${command} ok`, result);
    return result;
  } catch (error) {
    kerr("ipc", `${command} err`, error);
    throw error;
  }
}
