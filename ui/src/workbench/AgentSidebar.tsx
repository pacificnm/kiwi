import { useState } from "react";
import { Icon, isTauri } from "../shell";
import { AGENT_RUNTIMES } from "../lib/agent";
import {
  faCircleCheck,
  faChevronLeft,
  faPlus,
  faRotateRight,
  faTrash,
  faUser,
} from "../lib/fontawesome";
import { useWorkbench } from "./state";
import { useAgentSettings } from "./agentSettings";

/** Agent activity sidebar — connection mode, endpoint, models, and accounts. */
export function AgentSidebar({ onToggleCollapse }: { onToggleCollapse?: () => void }) {
  const { workspace } = useWorkbench();
  const {
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
  } = useAgentSettings();
  const [newModel, setNewModel] = useState("");

  if (!isTauri()) {
    return (
      <PanelFrame title="Agent" onToggleCollapse={onToggleCollapse}>
        <p className="text-xs text-nest-muted">
          Agent settings are available in the desktop app.
        </p>
      </PanelFrame>
    );
  }

  const isAccountMode = settings.connection === "account";
  const remoteByName = new Map(remoteModels.map((model) => [model.name, model]));

  return (
    <PanelFrame title="Agent" onToggleCollapse={onToggleCollapse}>
      <section className="space-y-1">
        <SectionLabel>Workspace</SectionLabel>
        <p className="truncate text-xs text-nest-muted" title={workspace?.root}>
          {workspace?.root ?? "No folder opened"}
        </p>
      </section>

      <section className="space-y-2">
        <SectionLabel>Connection</SectionLabel>
        <div className="grid grid-cols-2 gap-1 rounded-nest-sm border border-nest-border p-0.5">
          <ModeButton
            active={!isAccountMode}
            onClick={() => updateSettings({ connection: "ollama" })}
            title="Ollama server"
            subtitle="Local / self-hosted models"
          />
          <ModeButton
            active={isAccountMode}
            onClick={() => updateSettings({ connection: "account" })}
            title="Account"
            subtitle="Claude / Codex login"
          />
        </div>
        <p className="text-[11px] text-nest-muted">
          {isAccountMode
            ? "Runs the agent CLI directly using its own signed-in account and cloud models."
            : "Routes the agent through your Ollama server via ollama launch."}
        </p>
      </section>

      <section className="space-y-2">
        <SectionLabel>Runtime</SectionLabel>
        <p className="text-[11px] text-nest-muted">
          Agent launched by the <strong>Launch</strong> button.
        </p>
        <select
          value={settings.runtime}
          onChange={(event) => updateSettings({ runtime: event.target.value })}
          className="h-7 w-full appearance-none rounded-nest-sm border border-nest-border bg-nest-surface px-2 text-xs leading-none"
        >
          {AGENT_RUNTIMES.map((item) => (
            <option key={item.id} value={item.id}>
              {item.label}
            </option>
          ))}
        </select>
      </section>

      {isAccountMode ? (
        <AccountSection
          runtime={settings.runtime}
          codexAccount={codexAccount}
          onRefreshCodex={() => void refreshCodexAccount()}
          onCodexSignIn={() => void codexSignIn()}
          onCodexSignOut={() => void codexSignOut()}
        />
      ) : (
        <>
          <section className="space-y-2">
            <SectionLabel>Ollama endpoint</SectionLabel>
            <div className="flex gap-2">
              <label className="min-w-0 flex-1">
                <span className="mb-0.5 block text-[11px] text-nest-muted">Host</span>
                <input
                  value={settings.host}
                  onChange={(event) => updateSettings({ host: event.target.value })}
                  spellCheck={false}
                  placeholder="server.lan"
                  className="h-7 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 font-mono text-xs"
                />
              </label>
              <label className="w-20 shrink-0">
                <span className="mb-0.5 block text-[11px] text-nest-muted">Port</span>
                <input
                  value={settings.port}
                  onChange={(event) => {
                    const port = Number.parseInt(event.target.value, 10);
                    if (!Number.isNaN(port)) {
                      updateSettings({ port });
                    }
                  }}
                  spellCheck={false}
                  className="h-7 w-full rounded-nest-sm border border-nest-border bg-nest-surface px-2 font-mono text-xs"
                />
              </label>
            </div>
            <p className="font-mono text-[11px] text-nest-muted">{ollamaHost}</p>
          </section>

          <section className="space-y-2">
            <div className="flex items-center gap-2">
              <SectionLabel>Models</SectionLabel>
              <button
                type="button"
                onClick={() => void refreshModels()}
                disabled={loadingModels}
                title="Refresh from ollama list"
                className="ml-auto inline-flex h-6 items-center gap-1 rounded-nest-sm border border-nest-border px-2 text-[11px] text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground disabled:opacity-50"
              >
                <Icon
                  icon={faRotateRight}
                  className={["size-3", loadingModels ? "animate-spin" : ""].join(" ")}
                />
                Refresh
              </button>
            </div>
            <p className="text-[11px] text-nest-muted">
              Pulled from <code className="text-nest-foreground">ollama list</code> on the server.
            </p>

            <ul className="max-h-48 space-y-0.5 overflow-auto rounded-nest-sm border border-nest-border">
              {settings.models.length === 0 ? (
                <li className="px-2 py-2 text-xs text-nest-muted">No models — click Refresh</li>
              ) : (
                settings.models.map((name) => {
                  const remote = remoteByName.get(name);
                  const selected = settings.model === name;
                  return (
                    <li
                      key={name}
                      className={[
                        "group flex items-center gap-1 px-1 py-0.5",
                        selected ? "bg-nest-accent/15" : "hover:bg-nest-muted/10",
                      ].join(" ")}
                    >
                      <button
                        type="button"
                        onClick={() => setModel(name)}
                        className="flex min-w-0 flex-1 flex-col items-start px-1 py-0.5 text-left"
                      >
                        <span className="truncate font-mono text-xs text-nest-foreground">
                          {name}
                        </span>
                        {remote?.size ? (
                          <span className="text-[10px] text-nest-muted">
                            {remote.size}
                            {remote.modified ? ` · ${remote.modified}` : ""}
                          </span>
                        ) : null}
                      </button>
                      <button
                        type="button"
                        onClick={() => removeModel(name)}
                        title="Remove from list"
                        className="shrink-0 rounded-nest-sm p-1 text-nest-muted opacity-0 hover:bg-nest-muted/20 hover:text-nest-error group-hover:opacity-100"
                      >
                        <Icon icon={faTrash} className="size-2.5" />
                      </button>
                    </li>
                  );
                })
              )}
            </ul>

            <div className="flex gap-1">
              <input
                value={newModel}
                onChange={(event) => setNewModel(event.target.value)}
                onKeyDown={(event) => {
                  if (event.key === "Enter") {
                    addModel(newModel);
                    setNewModel("");
                  }
                }}
                placeholder="Add model tag…"
                spellCheck={false}
                className="h-7 min-w-0 flex-1 rounded-nest-sm border border-nest-border bg-nest-surface px-2 font-mono text-xs"
              />
              <button
                type="button"
                onClick={() => {
                  addModel(newModel);
                  setNewModel("");
                }}
                className="inline-flex h-7 items-center gap-1 rounded-nest-sm border border-nest-border px-2 text-xs hover:bg-nest-muted/10"
              >
                <Icon icon={faPlus} className="size-3" />
                Add
              </button>
            </div>
          </section>

          <section className="space-y-2">
            <div className="flex items-center gap-2">
              <SectionLabel>Ollama account</SectionLabel>
              <button
                type="button"
                onClick={() => void refreshAuth()}
                title="Refresh account status"
                className="ml-auto rounded-nest-sm p-1 text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
              >
                <Icon icon={faRotateRight} className="size-3" />
              </button>
            </div>
            <p className="text-[11px] text-nest-muted">
              Sign in to ollama.com for cloud models via your Ollama server.
            </p>
            <AccountBadge
              signedIn={auth?.signedIn ?? false}
              label={
                auth?.detail ??
                (auth?.signedIn ? "Signed in to ollama.com" : "Not signed in")
              }
            />
            <div className="flex gap-2">
              {auth?.signedIn ? (
                <button
                  type="button"
                  onClick={() => void signOut()}
                  className="h-7 flex-1 rounded-nest-sm border border-nest-border text-xs hover:bg-nest-muted/10"
                >
                  Sign out
                </button>
              ) : (
                <button
                  type="button"
                  onClick={() => void signIn()}
                  className="h-7 flex-1 rounded-nest-sm bg-nest-primary text-xs font-medium text-white hover:opacity-90"
                >
                  Sign in
                </button>
              )}
            </div>
          </section>
        </>
      )}

      <button
        type="button"
        onClick={() => void saveSettings()}
        disabled={saving}
        className="h-8 w-full rounded-nest-sm bg-nest-primary text-xs font-medium text-white hover:bg-nest-secondary disabled:opacity-50"
      >
        {saving ? "Saving…" : "Save settings"}
      </button>
    </PanelFrame>
  );
}

/** Account section for direct (native login) connection mode. */
function AccountSection({
  runtime,
  codexAccount,
  onRefreshCodex,
  onCodexSignIn,
  onCodexSignOut,
}: {
  runtime: string;
  codexAccount: { signedIn: boolean; detail: string } | null;
  onRefreshCodex: () => void;
  onCodexSignIn: () => void;
  onCodexSignOut: () => void;
}) {
  const isCodex = runtime === "codex" || runtime === "codex-app";
  const isClaude = runtime === "claude";

  return (
    <section className="space-y-2">
      <div className="flex items-center gap-2">
        <SectionLabel>Account</SectionLabel>
        {isCodex ? (
          <button
            type="button"
            onClick={onRefreshCodex}
            title="Refresh Codex status"
            className="ml-auto rounded-nest-sm p-1 text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faRotateRight} className="size-3" />
          </button>
        ) : null}
      </div>

      {isCodex ? (
        <>
          <p className="text-[11px] text-nest-muted">
            Codex signs in with your ChatGPT / OpenAI account.
          </p>
          <AccountBadge
            signedIn={codexAccount?.signedIn ?? false}
            label={codexAccount?.detail ?? "Checking…"}
          />
          <div className="flex gap-2">
            {codexAccount?.signedIn ? (
              <button
                type="button"
                onClick={onCodexSignOut}
                className="h-7 flex-1 rounded-nest-sm border border-nest-border text-xs hover:bg-nest-muted/10"
              >
                Sign out
              </button>
            ) : (
              <button
                type="button"
                onClick={onCodexSignIn}
                className="h-7 flex-1 rounded-nest-sm bg-nest-primary text-xs font-medium text-white hover:opacity-90"
              >
                Sign in with Codex
              </button>
            )}
          </div>
        </>
      ) : isClaude ? (
        <p className="rounded-nest-sm border border-nest-border bg-nest-surface p-2 text-[11px] leading-relaxed text-nest-muted">
          Claude Code signs in interactively. Press <strong>Launch</strong>, then run{" "}
          <code className="text-nest-foreground">/login</code> in the Agent terminal to
          authenticate with your Anthropic account. Or set{" "}
          <code className="text-nest-foreground">ANTHROPIC_API_KEY</code> in your environment.
        </p>
      ) : (
        <p className="rounded-nest-sm border border-nest-border bg-nest-surface p-2 text-[11px] leading-relaxed text-nest-muted">
          This runtime handles its own authentication. Press <strong>Launch</strong> and follow
          its sign-in prompt in the Agent terminal.
        </p>
      )}
    </section>
  );
}

function AccountBadge({ signedIn, label }: { signedIn: boolean; label: string }) {
  return (
    <div className="flex items-center gap-2 rounded-nest-sm border border-nest-border bg-nest-surface px-2 py-1.5">
      <Icon
        icon={faUser}
        className={signedIn ? "size-3.5 text-nest-success" : "size-3.5 text-nest-muted"}
      />
      <span className="min-w-0 truncate text-xs" title={label}>
        {label}
      </span>
      {signedIn ? (
        <Icon icon={faCircleCheck} className="ml-auto size-3 shrink-0 text-nest-success" />
      ) : null}
    </div>
  );
}

function ModeButton({
  active,
  onClick,
  title,
  subtitle,
}: {
  active: boolean;
  onClick: () => void;
  title: string;
  subtitle: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "flex flex-col items-start rounded-nest-sm px-2 py-1 text-left transition-colors",
        active
          ? "bg-nest-accent/15 text-nest-foreground"
          : "text-nest-muted hover:bg-nest-muted/10",
      ].join(" ")}
    >
      <span className="text-xs font-medium">{title}</span>
      <span className="text-[10px] leading-tight">{subtitle}</span>
    </button>
  );
}

function PanelFrame({
  title,
  children,
  onToggleCollapse,
}: {
  title: string;
  children: React.ReactNode;
  onToggleCollapse?: () => void;
}) {
  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="flex h-9 shrink-0 items-center border-b border-nest-border px-3">
        <span className="text-xs font-semibold uppercase tracking-wide text-nest-muted">
          {title}
        </span>
        {onToggleCollapse ? (
          <button
            type="button"
            onClick={onToggleCollapse}
            title="Hide sidebar"
            aria-label="Hide sidebar"
            className="ml-auto flex size-6 items-center justify-center rounded-nest-sm text-nest-muted hover:bg-nest-muted/10 hover:text-nest-foreground"
          >
            <Icon icon={faChevronLeft} className="size-3" />
          </button>
        ) : null}
      </header>
      <div className="min-h-0 flex-1 space-y-4 overflow-auto p-3">{children}</div>
    </div>
  );
}

function SectionLabel({ children }: { children: React.ReactNode }) {
  return <h3 className="text-xs font-semibold text-nest-foreground">{children}</h3>;
}
