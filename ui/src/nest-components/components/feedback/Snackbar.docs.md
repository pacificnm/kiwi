# Snackbar

Snackbars display brief messages about app operations at the bottom of the screen.

## When to Use

- Confirmation of completed actions
- Brief notifications that don't require interaction
- Undo/redo opportunities
- Non-critical status updates

## Differences from Alert

| Snackbar | Alert |
|----------|-------|
| Temporary, auto-dismisses | Persistent until dismissed |
| Bottom of screen (toast) | Inline with content |
| Single message | Can be part of content flow |
| For feedback | For important messages |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `boolean` | - | Snackbar open state (required) |
| `onClose` | `() => void` | - | Close callback |
| `severity` | `'success' \| 'error' \| 'warning' \| 'info'` | `'info'` | Message type |
| `autoHideDuration` | `number` | `5000` | Auto-close time (ms), 0 to disable |
| `action` | `ReactNode` | - | Action button (e.g., Undo) |
| `position` | See below | `'bottom-center'` | Screen position |

### Positions

- `top-left`, `top-center`, `top-right`
- `bottom-left`, `bottom-center`, `bottom-right`

## Examples

### Basic Usage

```tsx
import { Snackbar } from '@nest/components';

const [open, setOpen] = useState(false);

<Snackbar open={open} onClose={() => setOpen(false)}>
  Message sent!
</Snackbar>
```

### With Undo Action

```tsx
<Snackbar
  open={open}
  onClose={() => setOpen(false)}
  action={
    <Button size="small" onClick={handleUndo}>
      Undo
    </Button>
  }
>
  Item deleted
</Snackbar>
```

### Different Severities

```tsx
<Snackbar open={open} onClose={() => setOpen(false)} severity="success">
  Saved successfully!
</Snackbar>

<Snackbar open={open} onClose={() => setOpen(false)} severity="error">
  Failed to save
</Snackbar>
```

### Custom Position

```tsx
<Snackbar open={open} onClose={() => setOpen(false)} position="top-right">
  Notification in top-right
</Snackbar>
```

## Accessibility

- `role="status"` with `aria-live="polite"`
- Escape key closes the snackbar
- Focus is not trapped (user can interact with page)

## Best Practices

- Auto-dismiss after 3-5 seconds
- Use for non-critical messages only
- Provide undo actions when appropriate
- Don't stack multiple snackbars
