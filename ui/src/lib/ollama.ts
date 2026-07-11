import { kiwiInvoke } from "./ipc";

/** One model from `ollama list` on the inference server. */
export type OllamaModel = {
  name: string;
  size?: string | null;
  modified?: string | null;
};

/** Local ollama.com account status. */
export type OllamaAuthStatus = {
  signedIn: boolean;
  detail: string;
};

/** Lists models on the Ollama server at `host` (`host:port` or URL). */
export async function listOllamaModels(host: string): Promise<OllamaModel[]> {
  return kiwiInvoke<OllamaModel[]>("ollama_list_models", { host });
}

/** Returns whether the configured Ollama server is signed in to ollama.com. */
export async function ollamaAuthStatus(host: string): Promise<OllamaAuthStatus> {
  return kiwiInvoke<OllamaAuthStatus>("ollama_auth_status", { host });
}

/** Opens the ollama.com sign-in flow in the system browser. */
export async function ollamaSignIn(host: string): Promise<void> {
  return kiwiInvoke("ollama_signin", { host });
}

/** Signs out of ollama.com on the configured Ollama server. */
export async function ollamaSignOut(host: string): Promise<void> {
  return kiwiInvoke("ollama_signout", { host });
}

/** Native agent-account status (Codex `codex login status`). */
export type AccountStatus = {
  signedIn: boolean;
  detail: string;
};

/** Returns Codex account sign-in status for direct connection mode. */
export async function codexAccountStatus(): Promise<AccountStatus> {
  return kiwiInvoke<AccountStatus>("codex_account_status");
}

/** Launches `codex login` (browser OAuth) for direct account mode. */
export async function codexLogin(): Promise<void> {
  return kiwiInvoke("codex_login");
}

/** Clears stored Codex credentials (`codex logout`). */
export async function codexLogout(): Promise<void> {
  return kiwiInvoke("codex_logout");
}
