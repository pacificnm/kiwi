# Button

Buttons allow users to take actions with a single click.

## When to Use

- Submitting forms
- Triggering actions (save, delete, cancel)
- Navigation (when the action is primary)
- Opening dialogs or menus

## Variants

| Variant | When to Use |
|---------|-------------|
| `contained` | Primary actions, high emphasis |
| `outlined` | Secondary actions, medium emphasis |
| `text` | Tertiary actions, low emphasis |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `'contained' \| 'outlined' \| 'text'` | `'contained'` | Visual style |
| `color` | `'primary' \| 'secondary' \| 'accent' \| 'error' \| 'success' \| 'warning'` | `'primary'` | Color scheme |
| `size` | `'small' \| 'medium' \| 'large'` | `'medium'` | Button size |
| `startIcon` | `ReactNode` | - | Icon before label |
| `endIcon` | `ReactNode` | - | Icon after label |
| `loading` | `boolean` | `false` | Show loading spinner |
| `disabled` | `boolean` | `false` | Disable interactions |
| `fullWidth` | `boolean` | `false` | Full width button |
| `onClick` | `(e) => void` | - | Click handler |

## Examples

### Basic Usage

```tsx
import { Button } from '@nest/components';

<Button>Click me</Button>
```

### With Icon

```tsx
import { Button } from '@nest/components';
import { Save } from 'lucide-react';

<Button startIcon={<Save />}>Save Document</Button>
```

### Loading State

```tsx
<Button loading onClick={handleSubmit}>
  Submitting...
</Button>
```

### Color Variants

```tsx
<Button color="primary">Primary</Button>
<Button color="secondary">Secondary</Button>
<Button color="error">Delete</Button>
<Button color="success">Confirm</Button>
```

## Accessibility

- Buttons are natively focusable and keyboard accessible
- Use descriptive text or `aria-label` for icon-only buttons
- Loading state automatically sets `aria-busy`
