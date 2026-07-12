import { useCallback, useEffect, useState } from "react";
import { isTauri, useToast } from "../shell";
import { formatIpcError } from "../lib/agent";
import type { UnlistenFn } from "@tauri-apps/api/event";
import {
  newAppCrateProfile,
  newAppScaffold,
  listenNewAppProgress,
  type AppType,
  type CrateProfile,
  type NewAppProgress,
  type ScaffoldResponse,
} from "../lib/newApp";

/** Selectable app types with their descriptions, in display order. */
const APP_TYPES: { value: AppType; label: string; blurb: string }[] = [
  { value: "gui", label: "GUI", blurb: "Desktop app — Tauri + React + Tailwind" },
  { value: "tui", label: "TUI", blurb: "Terminal UI — Ratatui" },
  { value: "cli", label: "CLI", blurb: "Command-line application" },
  { value: "system", label: "System", blurb: "Background service / daemon" },
  { value: "api-server", label: "API Server", blurb: "HTTP API — nest-http-serve" },
  { value: "api-server-web", label: "API + Web", blurb: "HTTP API + Vite/React front end" },
];

const EMPTY_PROFILE: CrateProfile = { required: [], recommended: [], optional: [] };

const inputClass =
  "h-7 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-xs";

const labelClass = "mb-0.5 block text-[11px] text-nest-muted";

/**
 * New Application Wizard - scaffold a new Nest app under apps/.
 */
export function NewAppWizard() {
  const toast = useToast();

  const [appName, setAppName] = useState("");
  const [appType, setAppType] = useState<AppType>("gui");
  const [profile, setProfile] = useState<CrateProfile>(EMPTY_PROFILE);
  const [selectedCrates, setSelectedCrates] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [done, setDone] = useState(false);
  const [buildMessage, setBuildMessage] = useState<string | null>(null);
  const [modalOpen, setModalOpen] = useState(false);
  const [progress, setProgress] = useState<NewAppProgress[]>([]);
  const [finished, setFinished] = useState<null | "success" | "error">(null);

  // Load the crate profile whenever the app type changes. Recommended crates
  // start checked; required are implicit (always wired) and optional start off.
  useEffect(() => {
    if (!isTauri()) {
      return;
    }
    let cancelled = false;
    newAppCrateProfile(appType)
      .then((p) => {
        if (cancelled) {
          return;
        }
        setProfile(p);
        setSelectedCrates(p.recommended);
      })
      .catch((err) => {
        if (!cancelled) {
          toast.error(formatIpcError(err));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [appType, toast]);

  const handleCreate = useCallback(async () => {
    if (!appName.trim()) {
      setError("Application name is required.");
      return;
    }

    // Validate app name
    const nameRegex = /^[a-zA-Z0-9_-]+$/;
    if (!nameRegex.test(appName.trim())) {
      setError("Application name must contain only alphanumeric characters, hyphens, and underscores.");
      return;
    }

    setLoading(true);
    setError(null);
    setDone(false);
    setBuildMessage(null);
    setProgress([]);
    setFinished(null);
    setModalOpen(true);

    let unlisten: UnlistenFn | undefined;
    try {
      // Stream backend progress so the modal shows exactly what is happening
      // (and where it fails). Subscribe before invoking to catch every event.
      unlisten = await listenNewAppProgress((line) => {
        setProgress((prev) => [...prev, line]);
      });

      // Scaffold only — write the files and return immediately. We deliberately
      // do NOT compile here: a GUI `cargo check` builds the whole Tauri stack and
      // would freeze the wizard for minutes. The generated app ships a `./build`
      // script the user runs to compile/run.
      const result: ScaffoldResponse = await newAppScaffold({
        name: appName.trim(),
        appType,
        selectedCrates,
      });

      if (result.success) {
        setDone(true);
        setFinished("success");
        const where = result.appPath ? ` at ${result.appPath}` : "";
        setBuildMessage(
          `${result.message}${where}. Build it with ./build dev (GUI) or ./build run (CLI/service).`
        );
        toast.success(result.message);
      } else {
        setError(result.message);
        setFinished("error");
        toast.error(result.message);
      }
    } catch (createError) {
      const message = formatIpcError(createError);
      setError(message);
      setFinished("error");
      setProgress((prev) => [...prev, { message, error: true }]);
      toast.error(message);
    } finally {
      if (unlisten) {
        unlisten();
      }
      setLoading(false);
    }
  }, [appName, appType, selectedCrates, toast]);

  const toggleCrate = useCallback((crateName: string) => {
    setSelectedCrates((prev) =>
      prev.includes(crateName)
        ? prev.filter((c) => c !== crateName)
        : [...prev, crateName]
    );
  }, []);

  if (!isTauri()) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        New Application Wizard is available in the desktop app.
      </div>
    );
  }

  return (
    <div className="relative flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <h1 className="text-xl font-semibold text-nest-foreground">New Application</h1>
        <p className="mt-0.5 text-sm text-nest-muted">
          Create a new Nest application. The app will be scaffolded under the{" "}
          <code className="font-mono text-[12px]">apps/</code> folder.
        </p>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        <div className="space-y-6">
          {/* Application Name */}
          <section className="max-w-[520px] rounded-nest-lg border border-nest-border bg-nest-surface p-4">
            <label className="block">
              <span className={labelClass}>Application Name</span>
              <input
                className={inputClass}
                value={appName}
                onChange={(e) => setAppName(e.target.value)}
                placeholder="my-app"
                disabled={loading}
                autoFocus
              />
            </label>
          </section>

          {/* Application Type */}
          <section className="max-w-[520px] rounded-nest-lg border border-nest-border bg-nest-surface p-4">
            <span className={labelClass}>Application Type</span>
            <div className="mt-2 space-y-2">
              {APP_TYPES.map((t) => (
                <label key={t.value} className="flex items-center gap-2 text-sm">
                  <input
                    type="radio"
                    name="appType"
                    value={t.value}
                    checked={appType === t.value}
                    onChange={() => setAppType(t.value)}
                    disabled={loading}
                    className="size-4"
                  />
                  <span className="text-nest-foreground">
                    <strong>{t.label}</strong> — {t.blurb}
                  </span>
                </label>
              ))}
            </div>
          </section>

          {/* Nest Crates Selection */}
          <section className="max-w-[520px] rounded-nest-lg border border-nest-border bg-nest-surface p-4">
            <span className={labelClass}>Nest Core Crates</span>
            <p className="mb-3 text-xs text-nest-muted">
              Required crates for this app type are always included. Recommended
              crates are pre-selected; add optional ones as needed.
            </p>

            {profile.required.length > 0 ? (
              <div className="mb-3">
                <span className="text-[10px] uppercase tracking-wide text-nest-muted">Required</span>
                <div className="mt-1 grid grid-cols-2 gap-2">
                  {profile.required.map((crateName) => (
                    <label key={crateName} className="flex items-center gap-2 text-xs opacity-70">
                      <input type="checkbox" checked disabled className="size-4" />
                      <span className="font-mono text-nest-foreground">{crateName}</span>
                    </label>
                  ))}
                </div>
              </div>
            ) : null}

            {profile.recommended.length > 0 ? (
              <div className="mb-3">
                <span className="text-[10px] uppercase tracking-wide text-nest-muted">Recommended</span>
                <div className="mt-1 grid grid-cols-2 gap-2">
                  {profile.recommended.map((crateName) => (
                    <label key={crateName} className="flex items-center gap-2 text-xs">
                      <input
                        type="checkbox"
                        checked={selectedCrates.includes(crateName)}
                        onChange={() => toggleCrate(crateName)}
                        disabled={loading}
                        className="size-4"
                      />
                      <span className="font-mono text-nest-foreground">{crateName}</span>
                    </label>
                  ))}
                </div>
              </div>
            ) : null}

            {profile.optional.length > 0 ? (
              <details>
                <summary className="cursor-pointer text-[10px] uppercase tracking-wide text-nest-muted">
                  Optional ({profile.optional.length})
                </summary>
                <div className="mt-1 grid max-h-48 grid-cols-2 gap-2 overflow-y-auto">
                  {profile.optional.map((crateName) => (
                    <label key={crateName} className="flex items-center gap-2 text-xs">
                      <input
                        type="checkbox"
                        checked={selectedCrates.includes(crateName)}
                        onChange={() => toggleCrate(crateName)}
                        disabled={loading}
                        className="size-4"
                      />
                      <span className="font-mono text-nest-foreground">{crateName}</span>
                    </label>
                  ))}
                </div>
              </details>
            ) : null}
          </section>

          {/* Action Buttons */}
          <section className="max-w-[520px]">
            <div className="flex gap-2">
              <button
                type="button"
                onClick={handleCreate}
                disabled={loading || !appName.trim()}
                className="h-7 rounded-nest-sm border border-nest-border bg-nest-muted/10 px-3 text-xs hover:bg-nest-muted/20 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {loading ? "Creating…" : "Create Application"}
              </button>
            </div>
            {error ? (
              <p className="mt-2 text-xs text-nest-error">{error}</p>
            ) : null}
            {done && !error ? (
              <p className="mt-2 text-xs text-nest-success">{buildMessage}</p>
            ) : null}
          </section>
        </div>
      </div>

      {modalOpen ? (
        <div className="absolute inset-0 z-50 flex items-center justify-center bg-black/40 p-6">
          <div className="flex max-h-[80%] w-full max-w-[560px] flex-col rounded-nest-lg border border-nest-border bg-nest-surface shadow-lg">
            <header className="flex items-center gap-2 border-b border-nest-border px-4 py-3">
              {finished === null ? (
                <span className="size-3 animate-spin rounded-full border-2 border-nest-muted border-t-transparent" />
              ) : finished === "success" ? (
                <span className="text-nest-success">✓</span>
              ) : (
                <span className="text-nest-error">✕</span>
              )}
              <h2 className="text-sm font-semibold text-nest-foreground">
                {finished === null
                  ? `Creating ${appName.trim()}…`
                  : finished === "success"
                    ? `Created ${appName.trim()}`
                    : `Failed to create ${appName.trim()}`}
              </h2>
            </header>

            <div className="min-h-0 flex-1 overflow-y-auto px-4 py-3 font-mono text-[11px] leading-relaxed">
              {progress.length === 0 ? (
                <p className="text-nest-muted">Starting…</p>
              ) : (
                progress.map((line, i) => (
                  <div
                    key={i}
                    className={line.error ? "text-nest-error" : "text-nest-foreground"}
                  >
                    <span className="text-nest-muted">›</span> {line.message}
                  </div>
                ))
              )}
              {finished === "success" && buildMessage ? (
                <p className="mt-2 whitespace-pre-wrap text-nest-success">{buildMessage}</p>
              ) : null}
            </div>

            <footer className="flex justify-end border-t border-nest-border px-4 py-3">
              <button
                type="button"
                onClick={() => setModalOpen(false)}
                disabled={finished === null}
                className="h-7 rounded-nest-sm border border-nest-border bg-nest-muted/10 px-3 text-xs hover:bg-nest-muted/20 disabled:cursor-not-allowed disabled:opacity-50"
              >
                {finished === null ? "Working…" : "Close"}
              </button>
            </footer>
          </div>
        </div>
      ) : null}
    </div>
  );
}
