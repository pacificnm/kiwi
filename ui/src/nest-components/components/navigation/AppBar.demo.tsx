import { AppBar, Toolbar } from './AppBar';
import { MenuBar, MenuBarItem, MenuDivider, MenuItem } from './Menu';
import { IconButton } from '../inputs/IconButton';
import { FolderOpen, Save, Settings } from 'lucide-react';

/**
 * AppBar Component Demos
 *
 * Copy these examples into your app to get started.
 */

export function AppBarDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Toolbar Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Toolbar</h2>
        <div className="overflow-hidden rounded-nest-md border border-nest-border">
          <AppBar>
            <Toolbar>
              <span className="font-semibold text-nest-foreground">My App</span>
              <span className="flex-1" />
              <IconButton aria-label="Save" size="small"><Save className="size-4" /></IconButton>
              <IconButton aria-label="Settings" size="small"><Settings className="size-4" /></IconButton>
            </Toolbar>
          </AppBar>
        </div>
      </section>

      {/* Colors Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-3">
          {(['surface', 'primary', 'transparent'] as const).map((color) => (
            <div key={color} className="overflow-hidden rounded-nest-md border border-nest-border">
              <AppBar color={color}>
                <Toolbar>
                  <span className="font-semibold">{color}</span>
                </Toolbar>
              </AppBar>
            </div>
          ))}
        </div>
      </section>

      {/* Menu bar Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Menu Bar (File / Edit / Help)</h2>
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
        <p className="mt-2 text-xs text-nest-muted">
          AppBar composed with {'<'}MenuBar{'>'} — the pattern used by Kiwi and Swift's top chrome.
        </p>
      </section>
    </div>
  );
}
