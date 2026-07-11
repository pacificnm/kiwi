import { kiwiInvoke } from "./ipc";

export async function kiwiConfigPath(): Promise<string> {
  return kiwiInvoke<string>("kiwi_config_path");
}

export async function kiwiConfigRead(): Promise<string> {
  return kiwiInvoke<string>("kiwi_config_read");
}

export async function kiwiConfigWrite(content: string): Promise<void> {
  return kiwiInvoke<void>("kiwi_config_write", { content });
}
