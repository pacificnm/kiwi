import { kiwiInvoke } from "./ipc";

export type KiwiHostInfo = {
  name: string;
  shell: string;
};

export async function fetchKiwiHostInfo(): Promise<KiwiHostInfo> {
  return kiwiInvoke<KiwiHostInfo>("kiwi_host_info");
}
