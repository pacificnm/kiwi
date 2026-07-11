import { useEffect, useState, type ReactNode } from "react";
import { WindowControls } from "../components/WindowControls";
import { Icon } from "../components/Icon";
import { faChevronDown } from "../lib/fontawesome";
import { issueNumberFromTab } from "../lib/github";
import { isTauri, useToast } from "../shell";
import { useEditorCommands } from "./editorCommands";
import { useIssuesActions } from "./issues/issuesActions";
import { useWorkbench } from "./state";
import { LAYOUT } from "./activity";

type MenuBarProps = {
  title: string;
  model?: string;
  onOpenFolder?: () => void;
  onOpenSettings?: () => void;
  /** Switches to the Issues sidebar before a Git/GitHub action. */
  onFocusIssues?: () => void;
  /** Opens the Search sidebar and focuses the find field. */
  onFind?: () => void;
  /** Opens the Search sidebar and focuses the replace field. */
  onReplace?: () => void;
};

type OpenMenu = "file" | "edit" | "git" | null;

const menuButtonClass =
  "h-full px-2.5 text-[12px] text-nest-foreground hover:bg-nest-muted/12";

const menuDropdownClass =
  "absolute left-0 top-full z-[80] min-w-52 rounded-nest-md border border-nest-border bg-nest-background py-1 shadow-lg";

const menuItemClass =
  "flex w-full items-center justify-between gap-6 px-3 py-1.5 text-left text-[12px] hover:bg-nest-muted/10";

const menuItemDisabledClass =
  "flex w-full cursor-default items-center justify-between gap-6 px-3 py-1.5 text-left text-[12px] text-nest-muted/50";

/**
 * Frameless title bar: File/Edit/Git menus, centered workspace title, settings + window controls.
 */
export function MenuBar({
  title,
  model,
  onOpenFolder,
  onOpenSettings,
  onFocusIssues,
  onFind,
  onReplace,
}: MenuBarProps) {
  const [openMenu, setOpenMenu] = useState<OpenMenu>(null);
  const { activePath } = useWorkbench();
  const editor = useEditorCommands();
  const issues = useIssuesActions();
  const toast = useToast();
  const close = () => setOpenMenu(null);
  const showWindowChrome = isTauri();

  useEffect(() => {
    if (!openMenu) {
      return;
    }
    const onPointerDown = (event: MouseEvent) => {
      const target = event.target;
      if (!(target instanceof Element)) {
        return;
      }
      if (target.closest("[data-kiwi-menu]")) {
        return;
      }
      close();
    };
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        close();
      }
    };
    window.addEventListener("mousedown", onPointerDown, true);
    window.addEventListener("keydown", onKeyDown);
    return () => {
      window.removeEventListener("mousedown", onPointerDown, true);
      window.removeEventListener("keydown", onKeyDown);
    };
  }, [openMenu]);

  const runGitAction = (action: () => void) => {
    if (!isTauri()) {
      toast.info("GitHub actions require the desktop app");
      close();
      return;
    }
    onFocusIssues?.();
    action();
    close();
  };

  const runEditAction = (action: () => void) => {
    action();
    close();
  };

  const toggleMenu = (menu: Exclude<OpenMenu, null>) => {
    setOpenMenu((current) => (current === menu ? null : menu));
  };

  return (
    <header
      className="relative flex shrink-0 items-stretch border-b border-nest-border bg-nest-surface text-[13px]"
      style={{ height: LAYOUT.menuBarHeight }}
    >
      <div className="relative z-10 flex h-full shrink-0 items-stretch pl-2" data-kiwi-menu>
        <MenuDropdown
          label="File"
          open={openMenu === "file"}
          onToggle={() => toggleMenu("file")}
        >
          <MenuItem
            label="Open Folder…"
            onClick={() => {
              onOpenFolder?.();
              close();
            }}
          />
          <MenuItem label="Open Recent" disabled trailing={<Chevron />} />
        </MenuDropdown>

        <MenuDropdown
          label="Edit"
          open={openMenu === "edit"}
          onToggle={() => toggleMenu("edit")}
        >
          <MenuItem
            label="Undo"
            shortcut="Ctrl/Cmd+Z"
            disabled={!editor.canUndo}
            onClick={() => runEditAction(editor.undo)}
          />
          <MenuItem
            label="Redo"
            shortcut="Ctrl/Cmd+Shift+Z"
            disabled={!editor.canRedo}
            onClick={() => runEditAction(editor.redo)}
          />
          <MenuSeparator />
          <MenuItem
            label="Cut"
            shortcut="Ctrl/Cmd+X"
            disabled={!editor.canCut}
            onClick={() => runEditAction(editor.cut)}
          />
          <MenuItem
            label="Copy"
            shortcut="Ctrl/Cmd+C"
            disabled={!editor.canCopy}
            onClick={() => runEditAction(editor.copy)}
          />
          <MenuItem
            label="Paste"
            shortcut="Ctrl/Cmd+V"
            disabled={!editor.canPaste}
            onClick={() => runEditAction(editor.paste)}
          />
          <MenuSeparator />
          <MenuItem
            label="Find"
            shortcut="Ctrl/Cmd+Shift+F"
            onClick={() => {
              onFind?.();
              close();
            }}
          />
          <MenuItem
            label="Replace"
            shortcut="Ctrl/Cmd+Shift+H"
            onClick={() => {
              onReplace?.();
              close();
            }}
          />
        </MenuDropdown>

        <MenuDropdown label="Git" open={openMenu === "git"} onToggle={() => toggleMenu("git")}>
          <MenuItem
            label="New Issue"
            onClick={() => runGitAction(() => issues.openNewIssue())}
          />
          <MenuItem
            label="New Comment"
            onClick={() =>
              runGitAction(() =>
                issues.openNewComment(issueNumberFromTab(activePath ?? "") ?? undefined),
              )
            }
          />
          <MenuItem
            label="Manage Labels"
            onClick={() => runGitAction(() => void issues.openManageLabels())}
          />
          <MenuItem
            label="Manage Milestones"
            onClick={() => runGitAction(() => void issues.openManageMilestones())}
          />
        </MenuDropdown>
      </div>

      {showWindowChrome ? (
        <div className="min-w-0 flex-1" data-tauri-drag-region />
      ) : (
        <div className="min-w-0 flex-1" />
      )}

      <div className="relative z-10 flex h-full shrink-0 items-stretch gap-3 pr-0 pl-2">
        {model ? (
          <span
            className="hidden max-w-[140px] truncate self-center text-[11px] text-nest-muted sm:inline"
            title="Active model"
          >
            {model}
          </span>
        ) : null}
        <button
          type="button"
          className={`${menuButtonClass} font-medium`}
          onClick={onOpenSettings}
        >
          Settings
        </button>
        {showWindowChrome ? <WindowControls /> : null}
      </div>

      <p
        className="pointer-events-none absolute inset-0 z-0 flex items-center justify-center px-32"
        aria-hidden
      >
        <span className="truncate text-[12px] font-medium text-nest-foreground">
          {title}
        </span>
      </p>
    </header>
  );
}

function MenuDropdown({
  label,
  open,
  onToggle,
  children,
}: {
  label: string;
  open: boolean;
  onToggle: () => void;
  children: ReactNode;
}) {
  return (
    <div className="relative flex h-full items-stretch">
      <button type="button" className={menuButtonClass} onClick={onToggle}>
        {label}
      </button>
      {open ? (
        <div className={menuDropdownClass} role="menu" data-kiwi-menu>
          {children}
        </div>
      ) : null}
    </div>
  );
}

function MenuItem({
  label,
  shortcut,
  disabled,
  trailing,
  onClick,
}: {
  label: string;
  shortcut?: string;
  disabled?: boolean;
  trailing?: ReactNode;
  onClick?: () => void;
}) {
  return (
    <button
      type="button"
      role="menuitem"
      disabled={disabled}
      className={disabled ? menuItemDisabledClass : menuItemClass}
      onClick={onClick}
    >
      <span className="flex items-center gap-1">
        {label}
        {trailing}
      </span>
      {shortcut ? <span className="text-[11px] text-nest-muted">{shortcut}</span> : null}
    </button>
  );
}

function MenuSeparator() {
  return <div className="my-1 h-px bg-nest-border" role="separator" />;
}

function Chevron() {
  return <Icon icon={faChevronDown} className="inline size-3 opacity-50" />;
}
