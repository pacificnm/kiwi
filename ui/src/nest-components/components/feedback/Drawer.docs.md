# Drawer

A slide-out panel component that anchors to one edge of the viewport. Drawers are commonly used for navigation menus, settings panels, or contextual content that needs to be temporarily accessible.

## When to Use

- **Navigation menus**: Side navigation that slides in from the left or right
- **Settings panels**: Quick access to settings without leaving the current page
- **Contextual information**: Details, filters, or tools related to the main content
- **Temporary workflows**: Multi-step forms or actions that need focus

## Anatomy

The Drawer consists of:
- **Backdrop**: Semi-transparent overlay covering the viewport (optional)
- **Panel**: The slide-out content container
- **Anchor point**: The edge from which the drawer slides (left, right, top, bottom)

## Usage

### Left Navigation Drawer

```tsx
import { Drawer } from '@nest/components';

function NavigationDrawer({ open, onClose }) {
  return (
    <Drawer open={open} onClose={onClose} anchor="left" width={280}>
      <nav className="p-4">
        <List>
          <ListItem button>
            <ListItemText primary="Dashboard" />
          </ListItem>
          <ListItem button>
            <ListItemText primary="Settings" />
          </ListItem>
        </List>
      </nav>
    </Drawer>
  );
}
```

### Right Settings Panel

```tsx
<Drawer 
  open={settingsOpen} 
  onClose={() => setSettingsOpen(false)} 
  anchor="right" 
  width={320}
>
  <div className="p-4">
    <Typography variant="h6">Settings</Typography>
    {/* Settings content */}
  </div>
</Drawer>
```

### Top Notification Panel

```tsx
<Drawer 
  open={notificationsOpen} 
  onClose={() => setNotificationsOpen(false)} 
  anchor="top" 
  height={200}
>
  <div className="p-4">
    <Typography variant="h6">Notifications</Typography>
    {/* Notification list */}
  </div>
</Drawer>
```

### Bottom Player Bar

```tsx
<Drawer 
  open={playerOpen} 
  onClose={() => setPlayerOpen(false)} 
  anchor="bottom" 
  height={100}
>
  <div className="p-4 flex items-center justify-between">
    <div>
      <Typography variant="subtitle1">Now Playing</Typography>
      <Typography variant="body2">Song - Artist</Typography>
    </div>
    {/* Player controls */}
  </div>
</Drawer>
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `boolean` | `false` | Controls drawer visibility |
| `onClose` | `() => void` | - | Callback when drawer requests to close |
| `anchor` | `'left' \| 'right' \| 'top' \| 'bottom'` | `'left'` | Edge from which drawer slides in |
| `width` | `number \| string` | `320` | Width for left/right anchors |
| `height` | `number \| string` | `256` | Height for top/bottom anchors |
| `closeOnOutsideClick` | `boolean` | `true` | Clicking backdrop closes drawer |
| `closeOnEscape` | `boolean` | `true` | Pressing Escape closes drawer |
| `trapFocus` | `boolean` | `true` | Traps keyboard focus inside drawer |
| `className` | `string` | - | Additional classes for drawer panel |
| `backdropClassName` | `string` | - | Additional classes for backdrop |

## Accessibility

- Drawer uses `role="dialog"` and `aria-modal="true"`
- Focus is automatically moved to the first focusable element when opened
- Tab key is trapped within the drawer while open
- Escape key closes the drawer (unless disabled)
- Backdrop click closes the drawer (unless disabled)

## Best Practices

- **Choose the right anchor**:
  - `left`: Primary navigation, app menus
  - `right`: Contextual panels, settings, details
  - `top`: Notifications, search, quick actions
  - `bottom`: Media players, quick toolbars

- **Size appropriately**: Drawers should be wide/tall enough for content but not overwhelm the viewport

- **Provide clear exit**: Always give users a way to close (button, Escape, or outside click)

- **Consider responsive behavior**: On mobile, drawers often take full width/height

## Related Components

- **Modal**: Centered dialog for focused interactions
- **Dialog**: Pre-styled modal with header/content/footer
- **AppBar**: Fixed top bar alternative for persistent navigation
- **Popover**: Lighter overlay for contextual content
