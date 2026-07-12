# Chip

A compact component that represents an input, attribute, or action.

## When to Use

Use `Chip` for:
- Tags and labels
- Filter selections
- Contact chips (email, phone recipients)
- Status indicators
- Action triggers
- Removable items in a list

## Variants

| Variant | Description |
|---------|-------------|
| `filled` (default) | Solid background color |
| `outlined` | Border with transparent background |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `variant` | `'filled' \| 'outlined'` | `'filled'` | Visual style |
| `color` | `'default' \| 'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info'` | `'default'` | Color scheme |
| `size` | `'small' \| 'medium'` | `'medium'` | Chip size |
| `label` | `ReactNode` | - | Chip content |
| `icon` | `ReactNode` | - | Icon before label |
| `onDelete` | `() => void` | - | Delete callback (shows delete icon) |
| `deleteLabel` | `string` | `'Delete'` | Accessibility label for delete button |
| `clickable` | `boolean` | `false` | Apply clickable styles |
| `disabled` | `boolean` | `false` | Disable chip |
| `onClick` | `() => void` | - | Click handler |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Basic Chip

```tsx
import { Chip } from '@nest/components';

<Chip label="Basic Chip" />
```

### Color Variants

```tsx
<Chip label="Default" color="default" />
<Chip label="Primary" color="primary" />
<Chip label="Secondary" color="secondary" />
<Chip label="Success" color="success" />
<Chip label="Warning" color="warning" />
<Chip label="Error" color="error" />
<Chip label="Info" color="info" />
```

### Outlined Variant

```tsx
<Chip label="Outlined" variant="outlined" />
<Chip label="Primary" variant="outlined" color="primary" />
<Chip label="Error" variant="outlined" color="error" />
```

### Size Variants

```tsx
<Chip label="Small" size="small" />
<Chip label="Medium" size="medium" />
```

### Chip with Icon

```tsx
import { Star, Mail } from 'lucide-react';

<Chip label="With Icon" icon={<Star />} />
<Chip label="Email" icon={<Mail />} color="primary" />
```

### Deletable Chip

```tsx
<Chip
  label="Deletable"
  onDelete={() => console.log('deleted')}
/>
```

### Clickable Chip

```tsx
<Chip
  label="Clickable"
  clickable
  onClick={() => console.log('clicked')}
/>
```

### Chip as Selection

```tsx
const [selected, setSelected] = useState('option-1');

<div className="flex gap-2">
  <Chip
    label="Option 1"
    variant={selected === 'option-1' ? 'filled' : 'outlined'}
    color={selected === 'option-1' ? 'primary' : 'default'}
    clickable
    onClick={() => setSelected('option-1')}
  />
  <Chip
    label="Option 2"
    variant={selected === 'option-2' ? 'filled' : 'outlined'}
    color={selected === 'option-2' ? 'primary' : 'default'}
    clickable
    onClick={() => setSelected('option-2')}
  />
</div>
```

### Contact Chips

```tsx
<div className="flex flex-wrap gap-2">
  <Chip
    label="john@example.com"
    icon={<Mail className="h-4 w-4" />}
    onDelete={() => removeContact('john')}
  />
  <Chip
    label="jane@example.com"
    icon={<Mail className="h-4 w-4" />}
    onDelete={() => removeContact('jane')}
  />
</div>
```

### Filter Tags

```tsx
const [filters, setFilters] = useState(['React', 'TypeScript']);

<div className="flex flex-wrap gap-2">
  {filters.map((filter) => (
    <Chip
      key={filter}
      label={filter}
      onDelete={() => setFilters(filters.filter((f) => f !== filter))}
      color="primary"
    />
  ))}
</div>
```

### Status Chips

```tsx
<Chip label="Active" color="success" />
<Chip label="Pending" color="warning" />
<Chip label="Error" color="error" />
<Chip label="Info" color="info" />
```

### Disabled Chip

```tsx
<Chip label="Disabled" disabled />
<Chip label="Disabled Primary" color="primary" disabled />
```

## Accessibility

- Deletable chips include a button with `aria-label` (customize with `deleteLabel`)
- Clickable chips should use `component="button"` (automatic when `clickable` or `onClick` set)
- Use descriptive labels for screen readers
- Ensure sufficient color contrast for text

## Styling

Chips use Nest design tokens for colors:

| Color Token | Filled Background | Outlined Border |
|-------------|-------------------|-----------------|
| `default` | `bg-nest-surface` | `border-nest-border` |
| `primary` | `bg-nest-primary` | `border-nest-primary` |
| `secondary` | `bg-nest-secondary` | `border-nest-secondary` |
| `accent` | `bg-nest-accent` | `border-nest-accent` |
| `success` | `bg-nest-success` | `border-nest-success` |
| `warning` | `bg-nest-warning` | `border-nest-warning` |
| `error` | `bg-nest-error` | `border-nest-error` |
| `info` | `bg-nest-info` | `border-nest-info` |
