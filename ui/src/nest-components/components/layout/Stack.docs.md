# Stack

A flexbox layout component for stacking children with consistent spacing.

## When to Use

Use `Stack` for:
- Vertical or horizontal lists of items with consistent spacing
- Form layouts with labeled inputs
- Button groups and action rows
- Card content layouts
- Navigation items

For more complex layouts, consider:
- `Grid` for 2D grid layouts
- `Box` for custom flex/grid configurations

## Variants

| Direction | Description |
|-----------|-------------|
| `column` (default) | Stacks children vertically |
| `row` | Stacks children horizontally |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | The HTML element to render as |
| `direction` | `'row' \| 'column'` | `'column'` | Flex direction |
| `spacing` | `0 \| 1 \| 2 \| 3 \| 4 \| 5 \| 6 \| 8` | `2` | Gap between children |
| `align` | `'start' \| 'center' \| 'end' \| 'stretch'` | `'stretch'` | Cross-axis alignment |
| `justify` | `'start' \| 'center' \| 'end' \| 'between' \| 'around'` | `'start'` | Main-axis distribution |
| `wrap` | `boolean` | `false` | Allow wrapping to next line |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Default Column Stack

```tsx
import { Stack } from '@nest/components';

<Stack spacing={2}>
  <div>Item 1</div>
  <div>Item 2</div>
  <div>Item 3</div>
</Stack>
```

### Horizontal Row

```tsx
<Stack direction="row" spacing={4}>
  <Button>Save</Button>
  <Button variant="outlined">Cancel</Button>
</Stack>
```

### Centered Content

```tsx
<Stack direction="row" spacing={2} align="center" justify="center">
  <Avatar src="/user.jpg" />
  <Typography variant="body1">Username</Typography>
</Stack>
```

### Form Layout

```tsx
<Stack spacing={4}>
  <Stack spacing={1}>
    <label className="text-sm font-medium">Email</label>
    <TextField placeholder="you@example.com" />
  </Stack>
  <Stack spacing={1}>
    <label className="text-sm font-medium">Password</label>
    <TextField type="password" placeholder="••••••••" />
  </Stack>
  <Stack direction="row" spacing={2} justify="end">
    <Button variant="outlined">Cancel</Button>
    <Button>Submit</Button>
  </Stack>
</Stack>
```

### Spacing with Justify Between

```tsx
<Stack direction="row" spacing={2} justify="between" align="center">
  <Typography variant="h6">Card Title</Typography>
  <IconButton aria-label="more">
    <MoreIcon />
  </IconButton>
</Stack>
```

### Wrapped Row

```tsx
<Stack direction="row" spacing={2} wrap className="w-64">
  <Chip label="Tag 1" />
  <Chip label="Tag 2" />
  <Chip label="Tag 3" />
  <Chip label="Tag 4" />
  <Chip label="Tag 5" />
</Stack>
```

### Custom Element

```tsx
<Stack component="nav" direction="row" spacing={4}>
  <a href="/home">Home</a>
  <a href="/about">About</a>
  <a href="/contact">Contact</a>
</Stack>
```

## Spacing Scale

The `spacing` prop maps to Tailwind's gap utility:

| spacing | gap class | Pixels (default) |
|---------|-----------|------------------|
| 0 | `gap-0` | 0 |
| 1 | `gap-1` | 0.25rem (4px) |
| 2 | `gap-2` | 0.5rem (8px) |
| 3 | `gap-3` | 0.75rem (12px) |
| 4 | `gap-4` | 1rem (16px) |
| 5 | `gap-5` | 1.25rem (20px) |
| 6 | `gap-6` | 1.5rem (24px) |
| 8 | `gap-8` | 2rem (32px) |

## Accessibility

- Use semantic `component` values for landmark regions (`nav`, `section`, etc.)
- Ensure adequate spacing for touch targets (minimum 44x44px)
- When `direction="row"`, consider `wrap` for responsive layouts
- Add appropriate ARIA labels when used for navigation
