# Dialog

Dialogs inform users about a task and can contain critical information, require decisions, or involve multiple tasks.

## When to Use

- Confirming destructive actions (delete, discard)
- Collecting form input without leaving context
- Displaying important information requiring acknowledgment
- Multi-step workflows

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `boolean` | - | Dialog open state (required) |
| `onClose` | `() => void` | - | Callback when closing |
| `title` | `ReactNode` | - | Dialog title |
| `actions` | `ReactNode` | - | Action buttons (Cancel, Confirm, etc.) |
| `disableBackdropClick` | `boolean` | `false` | Prevent close on backdrop click |
| `disableEscapeKeyDown` | `boolean` | `false` | Prevent close on Escape key |

## Examples

### Basic Dialog

```tsx
import { Dialog } from '@nest/components';

const [open, setOpen] = useState(false);

<Dialog
  open={open}
  onClose={() => setOpen(false)}
  title="Welcome"
>
  <p>This is dialog content.</p>
</Dialog>
```

### Confirmation Dialog

```tsx
<Dialog
  open={open}
  onClose={() => setOpen(false)}
  title="Delete Item?"
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
  <p>Are you sure? This cannot be undone.</p>
</Dialog>
```

### Form Dialog

```tsx
<Dialog
  open={open}
  onClose={() => setOpen(false)}
  title="Create New"
  actions={
    <>
      <Button onClick={() => setOpen(false)}>Cancel</Button>
      <Button onClick={handleSubmit}>Create</Button>
    </>
  }
>
  <form onSubmit={handleSubmit}>
    <TextField label="Name" value={name} onChange={...} />
  </form>
</Dialog>
```

## Accessibility

- Focus is trapped within the dialog
- Escape key closes the dialog by default
- Backdrop click closes by default
- `aria-modal="true"` is set automatically
- Title is rendered as `h2`

## Best Practices

- Keep dialogs focused on a single task
- Provide clear actions (Cancel, Confirm)
- Use destructive color for dangerous actions
- Don't nest dialogs
