# Alert

Alerts display prominent messages that require user attention.

## When to Use

- Form validation errors
- Success/failure notifications
- Warnings about potential issues
- Important contextual information

## Severity Levels

| Severity | When to Use |
|----------|-------------|
| `success` | Operations completed successfully |
| `error` | Errors, failures, critical issues |
| `warning` | Caution, potential problems |
| `info` | General information, neutral messages |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `severity` | `'success' \| 'error' \| 'warning' \| 'info'` | `'info'` | Message type |
| `variant` | `'filled' \| 'outlined' \| 'standard'` | `'filled'` | Visual style |
| `icon` | `ReactNode` | auto | Custom icon (or `null` to hide) |
| `onClose` | `() => void` | - | Close callback (shows X button) |
| `action` | `ReactNode` | - | Action button on right side |

## Examples

### Basic Usage

```tsx
import { Alert } from '@nest/components';

<Alert severity="success">Operation completed!</Alert>
```

### Dismissible Alert

```tsx
const [open, setOpen] = useState(true);

{open && (
  <Alert severity="info" onClose={() => setOpen(false)}>
    This message can be dismissed.
  </Alert>
)}
```

### With Action

```tsx
<Alert
  severity="error"
  action={<Button onClick={handleRetry}>Retry</Button>}
>
  Connection failed. Please try again.
</Alert>
```

### Without Icon

```tsx
<Alert severity="success" icon={null}>
  Simple message without icon.
</Alert>
```

## Accessibility

- `role="alert"` for immediate screen reader announcement
- Error alerts should be associated with form fields
- Close buttons are keyboard accessible
