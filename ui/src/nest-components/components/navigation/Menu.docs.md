# Menu

A dropdown menu panel, plus `MenuItem`, `MenuDivider`, and a `MenuBar` / `MenuBarItem`
pair for File/Edit-style top menus. All exported from this module.

## When to Use

- A contextual list of actions triggered from a button (options menu, row actions).
- A desktop-style menu bar (`MenuBar` + `MenuBarItem`) with File/Edit/Help dropdowns.
- For choosing a *value* (not an action), use [Select](./Select.md) instead.

> The panel positions itself absolutely under its nearest `position: relative` ancestor
> (no collision handling). A floating-ui upgrade is planned — see the port plan's Tier 3.

## Props

### Menu

| Prop      | Type           | Default | Description                                                         |
| --------- | -------------- | ------- | ------------------------------------------------------------------- |
| `open`    | `boolean`      | —       | Whether the panel is visible.                                       |
| `onClose` | `() => void`   | —       | Called on outside click / Escape (not on item click — see below).   |
| `children`| `ReactNode`    | —       | `MenuItem` / `MenuDivider` rows.                                     |

### MenuItem

| Prop           | Type         | Default | Description                              |
| -------------- | ------------ | ------- | ---------------------------------------- |
| `danger`       | `boolean`    | `false` | Renders in the error color (destructive).|
| `endAdornment` | `ReactNode`  | —       | Trailing content (shortcut, chevron).    |
| `disabled`     | `boolean`    | `false` | Disables the row.                        |

Also accepts native `<button>` attributes (`onClick`, …).

### MenuBarItem

| Prop       | Type      | Default | Description                              |
| ---------- | --------- | ------- | ---------------------------------------- |
| `id`       | `string`  | —       | Unique id within the `MenuBar`.          |
| `label`    | `string`  | —       | Trigger text (e.g. "File").              |
| `disabled` | `boolean` | `false` | Disables the top-level trigger.          |

## Examples

`Menu` does not close on item selection — call `onClose` from each item's `onClick`:

```tsx
import { Menu, MenuItem, MenuDivider } from '@nest/components';

<div className="relative inline-block">
  <Button onClick={() => setOpen(true)}>Options</Button>
  <Menu open={open} onClose={() => setOpen(false)}>
    <MenuItem onClick={() => { save(); setOpen(false); }}>Save</MenuItem>
    <MenuDivider />
    <MenuItem danger onClick={() => { remove(); setOpen(false); }}>Delete</MenuItem>
  </Menu>
</div>
```

Menu bar:

```tsx
import { MenuBar, MenuBarItem, MenuItem } from '@nest/components';

<MenuBar>
  <MenuBarItem id="file" label="File">
    <MenuItem onClick={openFile}>Open…</MenuItem>
  </MenuBarItem>
  <MenuBarItem id="edit" label="Edit">
    <MenuItem onClick={undo}>Undo</MenuItem>
  </MenuBarItem>
</MenuBar>
```

## Accessibility

- `Menu` panel is `role="menu"`; `MenuItem` is `role="menuitem"`; `MenuDivider` is `role="separator"`.
- `MenuBar` is `role="menubar"`; each `MenuBarItem` trigger has `aria-haspopup` and `aria-expanded`.
- Closes on outside click and Escape. Only one `MenuBarItem` dropdown is open at a time.
