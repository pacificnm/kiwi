import { useState } from "react";
import {
  AlertTriangle,
  Box,
  BookOpen,
  CheckCircle,
  Code as CodeIcon,
  Eye,
  FolderOpen,
  Info,
  Save,
  Settings,
  type LucideIcon,
} from "lucide-react";
import {
  Alert,
  AppBar,
  Button,
  Dialog,
  IconButton,
  Menu,
  MenuBar,
  MenuBarItem,
  MenuDivider,
  MenuItem,
  Snackbar,
  TextField,
  Toolbar,
} from "@nest/components";
import { COMPONENTS } from "../lib/componentsLibrary";

type DetailTab = "preview" | "docs" | "code";

type ComponentDetailViewProps = {
  componentId: string;
};

/** Read-only Preview/Docs/Code viewer for one @nest/components component. */
export function ComponentDetailView({ componentId }: ComponentDetailViewProps) {
  const [tab, setTab] = useState<DetailTab>("preview");
  const def = COMPONENTS.find((c) => c.id === componentId);

  if (!def) {
    return (
      <div className="flex h-full items-center justify-center text-sm text-nest-muted">
        Unknown component &ldquo;{componentId}&rdquo;.
      </div>
    );
  }

  return (
    <div className="flex h-full min-h-0 flex-col bg-nest-background">
      <header className="border-b border-nest-border bg-nest-surface/60 px-6 py-5">
        <div className="flex items-center gap-3">
          <def.icon className="size-6 text-nest-primary" />
          <div>
            <h1 className="text-xl font-semibold text-nest-foreground">{def.name}</h1>
            <p className="mt-0.5 text-sm text-nest-muted">{def.description}</p>
          </div>
        </div>
        <div className="mt-4 flex gap-1">
          <DetailTabButton icon={Eye} label="Preview" active={tab === "preview"} onClick={() => setTab("preview")} />
          <DetailTabButton
            icon={BookOpen}
            label="Documentation"
            active={tab === "docs"}
            onClick={() => setTab("docs")}
          />
          <DetailTabButton icon={CodeIcon} label="Code" active={tab === "code"} onClick={() => setTab("code")} />
        </div>
      </header>
      <div className="min-h-0 flex-1 overflow-y-auto px-6 py-6">
        {tab === "preview" ? <ComponentPreview componentId={def.id} /> : null}
        {tab === "docs" ? <ComponentDocs componentId={def.id} /> : null}
        {tab === "code" ? <ComponentCode componentId={def.id} /> : null}
      </div>
    </div>
  );
}

function DetailTabButton({
  icon: Icon,
  label,
  active,
  onClick,
}: {
  icon: LucideIcon;
  label: string;
  active: boolean;
  onClick: () => void;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={[
        "flex items-center gap-1.5 rounded-t-nest-md px-3 py-2 text-[13px] font-medium transition-colors",
        active
          ? "bg-nest-background text-nest-primary"
          : "text-nest-muted hover:bg-nest-primary/5 hover:text-nest-foreground",
      ].join(" ")}
    >
      <Icon className="size-3.5" />
      {label}
    </button>
  );
}

function PreviewSection({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div className="mb-6">
      <h3 className="mb-4 text-[15px] font-semibold text-nest-foreground">{title}</h3>
      {children}
    </div>
  );
}

function PreviewCard({ label, children }: { label?: string; children: React.ReactNode }) {
  return (
    <div className="rounded-nest-lg border border-nest-border bg-nest-surface p-4">
      {label ? <span className="mb-3 block text-[13px] font-medium text-nest-muted">{label}</span> : null}
      {children}
    </div>
  );
}

function ComponentPreview({ componentId }: { componentId: string }) {
  switch (componentId) {
    case "button":
      return <ButtonPreview />;
    case "text-field":
      return <TextFieldPreview />;
    case "dialog":
      return <DialogPreview />;
    case "alert":
      return <AlertPreview />;
    case "snackbar":
      return <SnackbarPreview />;
    case "icon-button":
      return <IconButtonPreview />;
    case "app-bar":
      return <AppBarPreview />;
    case "menu":
      return <MenuPreview />;
    default:
      return (
        <div className="flex min-h-[200px] items-center justify-center text-nest-muted">
          <p>Preview coming soon for {componentId}</p>
        </div>
      );
  }
}

function ButtonPreview() {
  return (
    <PreviewSection title="Live Preview">
      <div className="flex flex-col gap-4">
        <PreviewCard label="Variants">
          <div className="flex flex-wrap gap-3">
            <Button variant="contained">Contained</Button>
            <Button variant="outlined">Outlined</Button>
            <Button variant="text">Text</Button>
          </div>
        </PreviewCard>
        <PreviewCard label="Colors">
          <div className="flex flex-wrap gap-3">
            <Button variant="contained" color="primary">Primary</Button>
            <Button variant="contained" color="secondary">Secondary</Button>
            <Button variant="contained" color="error">Error</Button>
          </div>
        </PreviewCard>
        <PreviewCard label="With Icons">
          <div className="flex flex-wrap gap-3">
            <Button startIcon={<CheckCircle className="size-4" />}>With Icon</Button>
            <Button loading>Loading</Button>
            <Button disabled>Disabled</Button>
          </div>
        </PreviewCard>
      </div>
    </PreviewSection>
  );
}

function TextFieldPreview() {
  return (
    <PreviewSection title="Live Preview">
      <div className="flex flex-col gap-4">
        <PreviewCard label="Basic Input">
          <TextField label="First Name" placeholder="Enter your name" />
        </PreviewCard>
        <PreviewCard label="With Helper Text">
          <TextField label="Email" helperText="We'll never share your email" />
        </PreviewCard>
        <PreviewCard label="Error State">
          <TextField label="Username" error="This username is taken" defaultValue="taken" />
        </PreviewCard>
      </div>
    </PreviewSection>
  );
}

function DialogPreview() {
  const [open, setOpen] = useState(false);

  return (
    <PreviewSection title="Live Preview">
      <PreviewCard>
        <Button variant="contained" onClick={() => setOpen(true)}>Open Dialog</Button>
      </PreviewCard>

      <Dialog
        open={open}
        onClose={() => setOpen(false)}
        title="Sample Dialog"
        actions={
          <>
            <Button variant="text" onClick={() => setOpen(false)}>Cancel</Button>
            <Button variant="contained" onClick={() => setOpen(false)}>Confirm</Button>
          </>
        }
      >
        <p className="text-nest-foreground">
          This is a sample dialog demonstrating the Dialog component from Nest UI Components.
        </p>
      </Dialog>
    </PreviewSection>
  );
}

function AlertPreview() {
  return (
    <PreviewSection title="Live Preview">
      <div className="flex flex-col gap-4">
        <Alert severity="success"><strong>Success!</strong> Your changes have been saved.</Alert>
        <Alert severity="error"><strong>Error!</strong> Something went wrong.</Alert>
        <Alert severity="warning"><strong>Warning!</strong> Please review before continuing.</Alert>
        <Alert severity="info"><strong>Info:</strong> A new update is available.</Alert>
      </div>
    </PreviewSection>
  );
}

function SnackbarPreview() {
  const [open, setOpen] = useState(false);

  return (
    <PreviewSection title="Live Preview">
      <PreviewCard>
        <Button variant="contained" onClick={() => setOpen(true)}>Show Snackbar</Button>
      </PreviewCard>

      <Snackbar open={open} onClose={() => setOpen(false)} severity="success">
        Action completed successfully!
      </Snackbar>
    </PreviewSection>
  );
}

function IconButtonPreview() {
  return (
    <PreviewSection title="Live Preview">
      <PreviewCard>
        <div className="flex gap-3">
          <IconButton aria-label="settings">
            <Box className="size-5" />
          </IconButton>
          <IconButton aria-label="delete" color="error">
            <AlertTriangle className="size-5" />
          </IconButton>
          <IconButton aria-label="info" color="primary">
            <Info className="size-5" />
          </IconButton>
          <IconButton aria-label="warning" color="warning">
            <AlertTriangle className="size-5" />
          </IconButton>
        </div>
      </PreviewCard>
    </PreviewSection>
  );
}

function AppBarPreview() {
  return (
    <PreviewSection title="Live Preview">
      <div className="flex flex-col gap-4">
        <PreviewCard label="Basic Toolbar">
          <div className="overflow-hidden rounded-nest-md border border-nest-border">
            <AppBar>
              <Toolbar>
                <span className="font-semibold text-nest-foreground">My App</span>
                <span className="flex-1" />
                <IconButton aria-label="Save" size="small">
                  <Save className="size-4" />
                </IconButton>
                <IconButton aria-label="Settings" size="small">
                  <Settings className="size-4" />
                </IconButton>
              </Toolbar>
            </AppBar>
          </div>
        </PreviewCard>

        <PreviewCard label="File Menu (Kiwi / Swift style)">
          <p className="mb-2 text-xs text-nest-muted">
            AppBar composed with MenuBar — the pattern used by Kiwi and Swift's top chrome.
          </p>
          <div className="overflow-hidden rounded-nest-md border border-nest-border">
            <AppBar elevation={false}>
              <Toolbar variant="dense">
                <MenuBar>
                  <MenuBarItem id="file" label="File">
                    <MenuItem endAdornment="Ctrl/Cmd+O" onClick={() => {}}>
                      <FolderOpen className="size-3.5" />
                      Open…
                    </MenuItem>
                    <MenuItem endAdornment="Ctrl/Cmd+S" onClick={() => {}}>
                      <Save className="size-3.5" />
                      Save
                    </MenuItem>
                    <MenuDivider />
                    <MenuItem disabled>Open Recent</MenuItem>
                  </MenuBarItem>
                  <MenuBarItem id="edit" label="Edit">
                    <MenuItem endAdornment="Ctrl/Cmd+Z" onClick={() => {}}>Undo</MenuItem>
                    <MenuItem endAdornment="Ctrl/Cmd+Shift+Z" onClick={() => {}}>Redo</MenuItem>
                  </MenuBarItem>
                  <MenuBarItem id="help" label="Help">
                    <MenuItem onClick={() => {}}>About</MenuItem>
                  </MenuBarItem>
                </MenuBar>
                <span className="flex-1" />
                <span className="self-center truncate text-[11px] text-nest-muted">Project Title</span>
              </Toolbar>
            </AppBar>
          </div>
        </PreviewCard>
      </div>
    </PreviewSection>
  );
}

function MenuPreview() {
  const [open, setOpen] = useState(false);

  return (
    <PreviewSection title="Live Preview">
      <PreviewCard label="Standalone Dropdown">
        <div className="relative inline-block">
          <Button variant="outlined" onClick={() => setOpen((value) => !value)}>
            Options
          </Button>
          <Menu open={open} onClose={() => setOpen(false)}>
            <MenuItem onClick={() => setOpen(false)}>Rename</MenuItem>
            <MenuItem onClick={() => setOpen(false)}>Duplicate</MenuItem>
            <MenuDivider />
            <MenuItem danger onClick={() => setOpen(false)}>Delete</MenuItem>
          </Menu>
        </div>
      </PreviewCard>
      <p className="mt-4 text-xs text-nest-muted">
        See the <strong>AppBar</strong> component for the File/Edit/Help menu-bar pattern built on top of Menu.
      </p>
    </PreviewSection>
  );
}

const DOCS: Record<string, { usage: string; props: string[] }> = {
  button: {
    usage: `<Button variant="contained" color="primary">\n  Click me\n</Button>`,
    props: [
      "variant: 'contained' | 'outlined' | 'text'",
      "color: 'primary' | 'secondary' | 'error'",
      "size: 'small' | 'medium' | 'large'",
      "startIcon?: ReactNode",
      "endIcon?: ReactNode",
      "loading?: boolean",
      "disabled?: boolean",
    ],
  },
  "text-field": {
    usage: `<TextField\n  label="Email"\n  value={email}\n  onChange={(e) => setEmail(e.target.value)}\n  helperText="We'll never share your email"\n/>`,
    props: [
      "label?: string",
      "value?: string",
      "onChange?: (e) => void",
      "error?: string",
      "helperText?: ReactNode",
      "variant: 'outlined' | 'filled' | 'standard'",
      "multiline?: boolean",
      "rows?: number",
    ],
  },
  dialog: {
    usage: `<Dialog\n  open={open}\n  onClose={() => setOpen(false)}\n  title="Confirm"\n  actions={<Button onClick={handleConfirm}>OK</Button>}\n>\n  <p>Are you sure?</p>\n</Dialog>`,
    props: [
      "open: boolean (required)",
      "onClose: () => void",
      "title?: ReactNode",
      "actions?: ReactNode",
      "disableBackdropClick?: boolean",
      "disableEscapeKeyDown?: boolean",
    ],
  },
  alert: {
    usage: `<Alert severity="success" onClose={() => setOpen(false)}>\n  Operation completed!\n</Alert>`,
    props: [
      "severity: 'success' | 'error' | 'warning' | 'info'",
      "variant: 'filled' | 'outlined' | 'standard'",
      "icon?: ReactNode",
      "onClose?: () => void",
      "action?: ReactNode",
    ],
  },
  snackbar: {
    usage: `<Snackbar\n  open={open}\n  onClose={() => setOpen(false)}\n  severity="success"\n>\n  Message here\n</Snackbar>`,
    props: [
      "open: boolean (required)",
      "onClose: () => void",
      "severity?: 'success' | 'error' | 'warning' | 'info'",
      "autoHideDuration?: number",
      "action?: ReactNode",
      "position?: ToastPosition",
    ],
  },
  "icon-button": {
    usage: `<IconButton aria-label="delete" color="error">\n  <TrashIcon />\n</IconButton>`,
    props: [
      "aria-label: string (required)",
      "color: 'default' | 'primary' | 'error' | etc.",
      "size: 'small' | 'medium' | 'large'",
      "disabled?: boolean",
    ],
  },
  "app-bar": {
    usage: `<AppBar>\n  <Toolbar>\n    <span>My App</span>\n  </Toolbar>\n</AppBar>`,
    props: [
      "position: 'static' | 'fixed' | 'sticky'",
      "color: 'surface' | 'primary' | 'transparent'",
      "elevation?: boolean",
      "— Toolbar —",
      "variant: 'regular' | 'dense'",
    ],
  },
  menu: {
    usage: `<div className="relative inline-block">\n  <Button onClick={() => setOpen(true)}>Options</Button>\n  <Menu open={open} onClose={() => setOpen(false)}>\n    <MenuItem onClick={() => { save(); setOpen(false); }}>Save</MenuItem>\n    <MenuDivider />\n    <MenuItem danger onClick={() => { remove(); setOpen(false); }}>Delete</MenuItem>\n  </Menu>\n</div>`,
    props: [
      "open: boolean (required)",
      "onClose: () => void (required) — call it yourself from each MenuItem's onClick",
      "— MenuItem —",
      "danger?: boolean",
      "endAdornment?: ReactNode",
      "disabled?: boolean",
      "— MenuBar / MenuBarItem —",
      "MenuBarItem id: string (required, unique per MenuBar)",
      "MenuBarItem label: string",
    ],
  },
};

function ComponentDocs({ componentId }: { componentId: string }) {
  const doc = DOCS[componentId];

  if (!doc) {
    return (
      <div className="flex min-h-[200px] items-center justify-center text-nest-muted">
        <p>Documentation coming soon</p>
      </div>
    );
  }

  return (
    <div className="max-w-[800px]">
      <section className="mb-6">
        <h3 className="mb-3 text-[15px] font-semibold text-nest-foreground">Usage</h3>
        <pre className="overflow-x-auto whitespace-pre rounded-nest-md border border-nest-border bg-nest-background p-4 font-mono text-[13px] text-nest-foreground">
          {doc.usage}
        </pre>
      </section>
      <section>
        <h3 className="mb-3 text-[15px] font-semibold text-nest-foreground">Props</h3>
        <ul className="list-none p-0">
          {doc.props.map((prop) => (
            <li
              key={prop}
              className="flex items-baseline gap-2 border-b border-nest-border/50 py-2 text-sm last:border-b-0"
            >
              <code className="font-mono text-[13px] font-semibold text-nest-primary">
                {prop.split(":")[0]}
              </code>
              <span className="text-nest-muted">{prop.includes(":") ? prop.split(":")[1] : ""}</span>
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}

const EXAMPLES: Record<string, string> = {
  button: `<Button variant="contained" color="primary">
  Click me
</Button>

<Button variant="outlined" startIcon={<Save />}>
  Save
</Button>

<Button loading onClick={handleSubmit}>
  Submit
</Button>`,
  "text-field": `<TextField
  label="Email"
  value={email}
  onChange={(e) => setEmail(e.target.value)}
  helperText="We'll never share your email"
/>

<TextField
  label="Password"
  type="password"
  error={passwordError}
  startAdornment={<Lock />}
/>`,
  dialog: `<Dialog
  open={open}
  onClose={() => setOpen(false)}
  title="Confirm Delete"
  actions={
    <>
      <Button variant="text" onClick={() => setOpen(false)}>
        Cancel
      </Button>
      <Button variant="contained" color="error" onClick={handleDelete}>
        Delete
      </Button>
    </>
  }
>
  <p>Are you sure you want to delete this item?</p>
</Dialog>`,
  alert: `<Alert severity="success" onClose={() => setOpen(false)}>
  Operation completed successfully!
</Alert>

<Alert
  severity="error"
  action={<Button size="small">Retry</Button>}
>
  Connection failed
</Alert>`,
  snackbar: `<Snackbar
  open={open}
  onClose={() => setOpen(false)}
  severity="success"
  action={<Button size="small" onClick={handleUndo}>Undo</Button>}
>
  Item deleted
</Snackbar>`,
  "icon-button": `<IconButton aria-label="delete" color="error">
  <TrashIcon />
</IconButton>

<IconButton aria-label="settings" size="large">
  <SettingsIcon />
</IconButton>`,
  "app-bar": `<AppBar>
  <Toolbar>
    <span className="font-semibold">My App</span>
    <span className="flex-1" />
    <IconButton aria-label="Save" size="small">
      <Save className="size-4" />
    </IconButton>
  </Toolbar>
</AppBar>

// File-menu bar (Kiwi / Swift style) — AppBar + MenuBar composed together
<AppBar elevation={false}>
  <Toolbar variant="dense">
    <MenuBar>
      <MenuBarItem id="file" label="File">
        <MenuItem endAdornment="Ctrl/Cmd+O" onClick={openFile}>
          Open…
        </MenuItem>
        <MenuDivider />
        <MenuItem endAdornment="Ctrl/Cmd+S" onClick={save}>
          Save
        </MenuItem>
      </MenuBarItem>
      <MenuBarItem id="help" label="Help">
        <MenuItem onClick={showAbout}>About</MenuItem>
      </MenuBarItem>
    </MenuBar>
  </Toolbar>
</AppBar>`,
  menu: `<div className="relative inline-block">
  <Button onClick={() => setOpen(true)}>Options</Button>
  <Menu open={open} onClose={() => setOpen(false)}>
    <MenuItem onClick={() => { rename(); setOpen(false); }}>
      Rename
    </MenuItem>
    <MenuDivider />
    <MenuItem danger onClick={() => { remove(); setOpen(false); }}>
      Delete
    </MenuItem>
  </Menu>
</div>`,
};

function ComponentCode({ componentId }: { componentId: string }) {
  return (
    <div className="max-w-[800px]">
      <pre className="overflow-x-auto whitespace-pre rounded-nest-md border border-nest-border bg-nest-background p-4 font-mono text-[13px] text-nest-foreground">
        <code>{EXAMPLES[componentId] || "// Example coming soon"}</code>
      </pre>
      <p className="mt-3 text-xs text-nest-muted">
        Import from <code>@nest/components</code>
      </p>
    </div>
  );
}
