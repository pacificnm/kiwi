# Badge

A component that displays a small status or notification badge overlaid on its children.

## When to Use

Use `Badge` for:
- Notification counts on icons
- Unread message indicators
- Status dots (online, away, busy)
- "New" labels on items
- Sale tags on products
- Overdue indicators

## Variants

| Variant | Description |
|---------|-------------|
| `standard` (default) | Shows badge content as text |
| `dot` | Small dot indicator without content |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'span'` | HTML element to render as |
| `badgeContent` | `ReactNode` | - | Content to display in badge |
| `color` | `'default' \| 'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info'` | `'default'` | Badge color |
| `variant` | `'standard' \| 'dot'` | `'standard'` | Visual style |
| `max` | `number` | `99` | Max value before showing "N+" |
| `showZero` | `boolean` | `false` | Show badge when content is 0 |
| `anchorOrigin` | `{ vertical: 'top' \| 'bottom', horizontal: 'left' \| 'right' }` | `{ top, right }` | Badge position |
| `invisible` | `boolean` | `false` | Hide badge |
| `children` | `ReactNode` | - | Element to wrap badge around |
| `className` | `string` | - | Additional CSS classes for badge |
| `wrapperClassName` | `string` | - | Additional CSS classes for wrapper |

## Examples

### Basic Badge

```tsx
import { Badge, Avatar } from '@nest/components';

<Badge badgeContent={4}>
  <Avatar src="/user.jpg" />
</Badge>
```

### Color Variants

```tsx
<Badge badgeContent={1} color="default">
  <Avatar>Default</Avatar>
</Badge>
<Badge badgeContent={1} color="primary">
  <Avatar>Primary</Avatar>
</Badge>
<Badge badgeContent={1} color="success">
  <Avatar>Success</Avatar>
</Badge>
<Badge badgeContent={1} color="error">
  <Avatar>Error</Avatar>
</Badge>
```

### Dot Variant

```tsx
// Status indicator
<Badge variant="dot" color="success">
  <Avatar>Online</Avatar>
</Badge>

// Notification dot
<Badge variant="dot" color="error">
  <BellIcon />
</Badge>
```

### Max Value

```tsx
// Shows "99+" when count exceeds max
<Badge badgeContent={999} max={99}>
  <Avatar>Notifications</Avatar>
</Badge>
```

### Anchor Positions

```tsx
// Top Right (default)
<Badge badgeContent={1}>
  <Avatar>TR</Avatar>
</Badge>

// Top Left
<Badge badgeContent={1} anchorOrigin={{ vertical: 'top', horizontal: 'left' }}>
  <Avatar>TL</Avatar>
</Badge>

// Bottom Right
<Badge badgeContent={1} anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}>
  <Avatar>BR</Avatar>
</Badge>

// Bottom Left
<Badge badgeContent={1} anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}>
  <Avatar>BL</Avatar>
</Badge>
```

### showZero

```tsx
// Hidden when 0 (default)
<Badge badgeContent={0}>
  <Avatar>Hidden</Avatar>
</Badge>

// Visible when 0
<Badge badgeContent={0} showZero>
  <Avatar>Visible</Avatar>
</Badge>
```

### Icon Buttons with Badges

```tsx
import { Bell, Mail } from 'lucide-react';

// Notifications with count
<Badge badgeContent={3} color="error">
  <IconButton aria-label="notifications">
    <Bell />
  </IconButton>
</Badge>

// Unread indicator
<Badge variant="dot" color="error">
  <IconButton aria-label="messages">
    <Mail />
  </IconButton>
</Badge>
```

### Status Indicators

```tsx
<div className="relative">
  <Avatar src="/user.jpg" alt="Online" />
  <Badge
    variant="dot"
    color="success"
    className="!static bottom-0 right-0 !translate-x-0 !translate-y-0"
  />
</div>
```

### Invisible Badge

```tsx
// Conditionally show badge
<Badge badgeContent={count} invisible={count === 0}>
  <Avatar>Notifications</Avatar>
</Badge>
```

### Custom Styling

```tsx
// Gradient background
<Badge
  badgeContent={5}
  className="bg-gradient-to-br from-primary to-accent text-white"
>
  <Avatar>Gradient</Avatar>
</Badge>

// Bordered badge
<Badge
  badgeContent={3}
  className="border-2 border-white"
  color="primary"
>
  <Avatar>Bordered</Avatar>
</Badge>

// Custom text badge
<Badge
  badgeContent="Pro"
  className="bg-foreground text-background font-bold"
>
  <Avatar>Custom</Avatar>
</Badge>
```

## Color Guide

| Color | Use Case |
|-------|----------|
| `default` | Neutral indicators |
| `primary` | Primary actions/notifications |
| `secondary` | Secondary information |
| `accent` | Highlighted items |
| `success` | Online, completed, positive |
| `warning` | Away, pending, caution |
| `error` | Busy, errors, urgent |
| `info` | Informational |

## Accessibility

- Provide `aria-label` on the badge for screen readers when content is numeric
- Use `invisible` instead of conditional rendering to maintain layout
- Ensure sufficient color contrast for badge text
- Dot variants should have `aria-label` describing the status

## Styling Notes

Badge uses absolute positioning relative to its wrapper:

```tsx
// Default positioning classes
top-0 right-0 -translate-y-1/2 translate-x-1/2  // top-right
```

For custom positioning, use `className` with `!static` on wrapper:

```tsx
<Badge className="!static bottom-0 right-0">
  <Avatar>Custom Position</Avatar>
</Badge>
```
