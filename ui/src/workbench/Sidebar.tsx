import { ACTIVITIES, type ActivityId } from "./activity";
import { PanelPlaceholder } from "./ActivityBar";
import { ExplorerPanel } from "./ExplorerPanel";
import { AgentSidebar } from "./AgentSidebar";
import { SearchPanel } from "./SearchPanel";
import { IssuesPanel } from "./issues/IssuesPanel";
import { ToolsPanel } from "./ToolsPanel";
import { SourceControlPanel } from "./SourceControlPanel";
import { TasksPanel } from "./TasksPanel";
import { HelpPanel } from "./HelpPanel";
import { ComponentsPanel } from "./ComponentsPanel";
import { ThemesPanel } from "./ThemesPanel";

type SidebarProps = {
  activity: ActivityId;
  /** Collapses the sidebar (toggle lives in the panel header). */
  onToggleCollapse?: () => void;
};

export function Sidebar({ activity, onToggleCollapse }: SidebarProps) {
  if (activity === "explorer") {
    return <ExplorerPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "search") {
    return <SearchPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "issues") {
    return <IssuesPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "source-control") {
    return <SourceControlPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "tasks") {
    return <TasksPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "agent") {
    return <AgentSidebar onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "tools") {
    return <ToolsPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "help") {
    return <HelpPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "components") {
    return <ComponentsPanel onToggleCollapse={onToggleCollapse} />;
  }
  if (activity === "theme") {
    return <ThemesPanel onToggleCollapse={onToggleCollapse} />;
  }
  const label = ACTIVITIES.find((item) => item.id === activity)?.label ?? "Sidebar";
  return <PanelPlaceholder title={label} onToggleCollapse={onToggleCollapse} />;
}
