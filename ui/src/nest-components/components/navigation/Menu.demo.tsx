import { useState } from 'react';
import { Menu, MenuBar, MenuBarItem, MenuDivider, MenuItem } from './Menu';
import { Button } from '../inputs/Button';
import { Copy, Pencil, Trash2 } from 'lucide-react';

/**
 * Menu Component Demos
 *
 * Copy these examples into your app to get started.
 */

export function MenuDemos() {
  const [open, setOpen] = useState(false);

  return (
    <div className="space-y-8 p-6">
      {/* Standalone dropdown Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Dropdown</h2>
        <div className="relative inline-block">
          <Button variant="outlined" onClick={() => setOpen((value) => !value)}>
            Options
          </Button>
          <Menu open={open} onClose={() => setOpen(false)}>
            <MenuItem onClick={() => setOpen(false)}>
              <Pencil className="size-3.5" />
              Rename
            </MenuItem>
            <MenuItem onClick={() => setOpen(false)}>
              <Copy className="size-3.5" />
              Duplicate
            </MenuItem>
            <MenuDivider />
            <MenuItem danger onClick={() => setOpen(false)}>
              <Trash2 className="size-3.5" />
              Delete
            </MenuItem>
          </Menu>
        </div>
      </section>

      {/* Menu bar Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Menu Bar</h2>
        <div className="inline-flex rounded-nest-md border border-nest-border bg-nest-surface">
          <MenuBar>
            <MenuBarItem id="file" label="File">
              <MenuItem endAdornment="Ctrl/Cmd+O" onClick={() => {}}>Open…</MenuItem>
              <MenuItem endAdornment="Ctrl/Cmd+S" onClick={() => {}}>Save</MenuItem>
              <MenuDivider />
              <MenuItem disabled>Open Recent</MenuItem>
            </MenuBarItem>
            <MenuBarItem id="edit" label="Edit">
              <MenuItem onClick={() => {}}>Undo</MenuItem>
              <MenuItem onClick={() => {}}>Redo</MenuItem>
            </MenuBarItem>
            <MenuBarItem id="help" label="Help">
              <MenuItem onClick={() => {}}>About</MenuItem>
            </MenuBarItem>
          </MenuBar>
        </div>
      </section>
    </div>
  );
}
