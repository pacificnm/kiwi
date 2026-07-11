import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from "react";
import { formatIpcError } from "../lib/agent";
import { kiwiInvoke } from "../lib/ipc";
import { klog } from "../lib/log";
import {
  codexAccountStatus,
  codexLogin,
  codexLogout,
  listOllamaModels,
  ollamaAuthStatus,
  ollamaSignIn,
  ollamaSignOut,
  type AccountStatus,
  type OllamaAuthStatus,
  type OllamaModel,
} from "../lib/ollama";
import { isTauri, useToast } from "../shell";

/** Connection mode for launching agents. */
export type AgentConnection = "ollama" | "account";

/** Agent sidebar settings persisted in Kiwi `config.toml`. */
export type AgentSettings = {
  host: string;
  port: number;
  model: string;
  models: string[];
  runtime: string;
  connection: AgentConnection;
};

type AgentSettingsContextValue = {
  settings: AgentSettings;
  /** Models returned by the last `ollama list` refresh. */
  remoteModels: OllamaModel[];
  auth: OllamaAuthStatus | null;
  /** Codex native-account status (direct mode). */
  codexAccount: AccountStatus | null;
  loadingModels: boolean;
  saving: boolean;
  /** `host:port` for `OLLAMA_HOST` / `ollama list`. */
  ollamaHost: string;
  updateSettings: (patch: Partial<AgentSettings>) => void;
  setModel: (model: string) => void;
  addModel: (name: string) => void;
  removeModel: (name: string) => void;
  refreshModels: () => Promise<void>;
  refreshAuth: () => Promise<void>;
  refreshCodexAccount: () => Promise<void>;
  saveSettings: () => Promise<void>;
  signIn: () => Promise<void>;
  signOut: () => Promise<void>;
  codexSignIn: () => Promise<void>;
  codexSignOut: () => Promise<void>;
};

const AgentSettingsContext = createContext<AgentSettingsContextValue | null>(null);

const DEFAULT_SETTINGS: AgentSettings = {
  host: "192.168.88.10",
  port: 11434,
  model: "qwen3.5:2b",
  models: ["qwen3.5:2b"],
  runtime: "claude",
  connection: "ollama",
};

export function AgentSettingsProvider({ children }: { children: ReactNode }) {
  const toast = useToast();
  const [settings, setSettings] = useState<AgentSettings>(DEFAULT_SETTINGS);
  const [remoteModels, setRemoteModels] = useState<OllamaModel[]>([]);
  const [auth, setAuth] = useState<OllamaAuthStatus | null>(null);
  const [codexAccount, setCodexAccount] = useState<AccountStatus | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);
  const [saving, setSaving] = useState(false);
  const [loaded, setLoaded] = useState(false);

  const ollamaHost = useMemo(
    () => `${settings.host.trim()}:${settings.port}`,
    [settings.host, settings.port],
  );

  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    void kiwiInvoke<AgentSettings>("agent_settings_get")
      .then((value) => {
        setSettings(value);
        setLoaded(true);
        klog("agent-settings", `loaded host=${value.host} model=${value.model}`);
      })
      .catch((error) => toast.error(formatIpcError(error)));
  }, [toast]);

  const refreshAuth = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    const status = await ollamaAuthStatus(ollamaHost);
    setAuth(status);
  }, [ollamaHost]);

  useEffect(() => {
    void refreshAuth();
  }, [refreshAuth]);

  const refreshCodexAccount = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    try {
      setCodexAccount(await codexAccountStatus());
    } catch (error) {
      klog("agent-settings", `codex status failed: ${formatIpcError(error)}`);
    }
  }, []);

  useEffect(() => {
    void refreshCodexAccount();
  }, [refreshCodexAccount]);

  const refreshModels = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setLoadingModels(true);
    try {
      const models = await listOllamaModels(ollamaHost);
      setRemoteModels(models);
      klog("agent-settings", `ollama list count=${models.length} host=${ollamaHost}`);
      // Merge remote names into the saved list without dropping custom entries.
      setSettings((current) => {
        const merged = [...current.models];
        for (const remote of models) {
          if (!merged.includes(remote.name)) {
            merged.push(remote.name);
          }
        }
        return { ...current, models: merged };
      });
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setLoadingModels(false);
    }
  }, [ollamaHost, toast]);

  // Auto-refresh models once settings load from disk.
  useEffect(() => {
    if (loaded && isTauri()) {
      void refreshModels();
    }
  }, [loaded, refreshModels]);

  const updateSettings = useCallback((patch: Partial<AgentSettings>) => {
    setSettings((current) => ({ ...current, ...patch }));
  }, []);

  const setModel = useCallback((model: string) => {
    setSettings((current) => {
      const models = current.models.includes(model)
        ? current.models
        : [model, ...current.models];
      return { ...current, model, models };
    });
  }, []);

  const addModel = useCallback((name: string) => {
    const trimmed = name.trim();
    if (!trimmed) {
      return;
    }
    setSettings((current) => {
      if (current.models.includes(trimmed)) {
        return current;
      }
      return { ...current, models: [...current.models, trimmed] };
    });
  }, []);

  const removeModel = useCallback((name: string) => {
    setSettings((current) => {
      const models = current.models.filter((item) => item !== name);
      const model =
        current.model === name ? (models[0] ?? current.model) : current.model;
      return { ...current, models, model };
    });
  }, []);

  const saveSettings = useCallback(async () => {
    if (!isTauri()) {
      return;
    }
    setSaving(true);
    try {
      const saved = await kiwiInvoke<AgentSettings>("agent_settings_save", {
        settings,
      });
      setSettings(saved);
      toast.success(`Saved agent settings (${saved.model})`);
    } catch (error) {
      toast.error(formatIpcError(error));
    } finally {
      setSaving(false);
    }
  }, [settings, toast]);

  const signIn = useCallback(async () => {
    try {
      await ollamaSignIn(ollamaHost);
      const immediate = await ollamaAuthStatus(ollamaHost);
      setAuth(immediate);
      if (immediate.signedIn) {
        toast.success(immediate.detail || "Signed in to ollama.com");
        return;
      }
      toast.info("Complete sign-in in your browser, then click Refresh");
      for (let attempt = 0; attempt < 10; attempt += 1) {
        await new Promise((resolve) => window.setTimeout(resolve, 3000));
        const status = await ollamaAuthStatus(ollamaHost);
        setAuth(status);
        if (status.signedIn) {
          toast.success(status.detail || "Signed in to ollama.com");
          return;
        }
      }
    } catch (error) {
      toast.error(formatIpcError(error));
    }
  }, [toast, ollamaHost]);

  const signOut = useCallback(async () => {
    try {
      await ollamaSignOut(ollamaHost);
      await refreshAuth();
      toast.success("Signed out of ollama.com");
    } catch (error) {
      toast.error(formatIpcError(error));
    }
  }, [toast, refreshAuth, ollamaHost]);

  const codexSignIn = useCallback(async () => {
    try {
      await codexLogin();
      toast.info("Complete Codex sign-in in your browser, then click Refresh");
      window.setTimeout(() => void refreshCodexAccount(), 3000);
    } catch (error) {
      toast.error(formatIpcError(error));
    }
  }, [toast, refreshCodexAccount]);

  const codexSignOut = useCallback(async () => {
    try {
      await codexLogout();
      await refreshCodexAccount();
      toast.success("Signed out of Codex");
    } catch (error) {
      toast.error(formatIpcError(error));
    }
  }, [toast, refreshCodexAccount]);

  const value = useMemo<AgentSettingsContextValue>(
    () => ({
      settings,
      remoteModels,
      auth,
      codexAccount,
      loadingModels,
      saving,
      ollamaHost,
      updateSettings,
      setModel,
      addModel,
      removeModel,
      refreshModels,
      refreshAuth,
      refreshCodexAccount,
      saveSettings,
      signIn,
      signOut,
      codexSignIn,
      codexSignOut,
    }),
    [
      settings,
      remoteModels,
      auth,
      codexAccount,
      loadingModels,
      saving,
      ollamaHost,
      updateSettings,
      setModel,
      addModel,
      removeModel,
      refreshModels,
      refreshAuth,
      refreshCodexAccount,
      saveSettings,
      signIn,
      signOut,
      codexSignIn,
      codexSignOut,
    ],
  );

  return (
    <AgentSettingsContext.Provider value={value}>{children}</AgentSettingsContext.Provider>
  );
}

export function useAgentSettings(): AgentSettingsContextValue {
  const value = useContext(AgentSettingsContext);
  if (!value) {
    throw new Error("useAgentSettings must be used within AgentSettingsProvider");
  }
  return value;
}
