import { useEffect } from "react";
import { WorkbenchShell } from "./workbench/WorkbenchShell";
import { useStatusBar } from "./shell";
import { applyThemeRootBlock, fetchAppMetadata, fetchThemeCss } from "./lib/nest";

export function App() {
  const { setStatus } = useStatusBar();

  useEffect(() => {
    void (async () => {
      try {
        const [meta, theme] = await Promise.all([fetchAppMetadata(), fetchThemeCss()]);
        applyThemeRootBlock(theme.root_block);
        setStatus(`${meta.title} ready`, { variant: "success", timeoutMs: 3000 });
      } catch {
        setStatus("Kiwi (Vite dev — no Tauri host)", { variant: "info" });
      }
    })();
  }, [setStatus]);

  return (
    <WorkbenchShell
      statusLeft={<span className="text-nest-muted">No folder opened</span>}
      statusRight={<span>Kiwi</span>}
    />
  );
}
