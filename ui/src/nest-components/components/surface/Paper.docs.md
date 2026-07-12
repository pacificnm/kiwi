# Paper

A surface container component with elevation (shadow) or border styling.

## When to Use

Use `Paper` for:
- Card-like containers
- Form containers
- Modal content areas
- Elevated content sections
- Grouping related content

For more specialized surfaces, consider:
- `Card` for pre-styled content cards with header/body/footer
- `Dialog` for modal overlays

## Variants

| Variant | Description |
|---------|-------------|
| `elevation` (default) | Uses shadow for depth |
| `outlined` | Uses border for definition |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `elevation` | `0 \| 1 \| 2 \| 3 \| 4` | `1` | Shadow depth |
| `variant` | `'elevation' \| 'outlined'` | `'elevation'` | Visual style |
| `square` | `boolean` | `false` | Remove border radius |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Default Paper

```tsx
import { Paper } from '@nest/components';

<Paper>
  <p>Content on a surface</p>
</Paper>
```

### Elevation Levels

```tsx
<Paper elevation={0}>No shadow (flat)</Paper>
<Paper elevation={1}>Small shadow (default)</Paper>
<Paper elevation={2}>Medium shadow</Paper>
<Paper elevation={3}>Large shadow</Paper>
<Paper elevation={4}>Extra large shadow</Paper>
```

### Outlined Variant

```tsx
<Paper variant="outlined">
  <p>Content with border instead of shadow</p>
</Paper>
```

### Square Corners

```tsx
<Paper square>
  <p>Sharp corners, no border radius</p>
</Paper>

<Paper square variant="outlined">
  <p>Square outlined corners</p>
</Paper>
```

### Custom Element

```tsx
<Paper component="article" elevation={2}>
  <h2>Article Title</h2>
  <p>Article content...</p>
</Paper>

<Paper component="section" variant="outlined">
  <h3>Section Heading</h3>
  <p>Section content...</p>
</Paper>
```

### Interactive Paper

```tsx
// As a clickable card
<Paper
  component="button"
  elevation={2}
  onClick={handleClick}
  className="cursor-pointer hover:bg-nest-primary/5"
>
  <h3>Clickable Card</h3>
  <p>Click me</p>
</Paper>

// As a link
<Paper
  component="a"
  href="/details"
  elevation={2}
  className="block hover:bg-nest-primary/5"
>
  <h3>Link Card</h3>
  <p>Go to details</p>
</Paper>
```

### Card-like Pattern

```tsx
<Paper elevation={2}>
  <div className="border-b border-nest-border bg-nest-muted/10 p-4">
    <h3 className="font-semibold">Card Header</h3>
  </div>
  <div className="p-4">
    <p>Card body content</p>
  </div>
  <div className="border-t border-nest-border bg-nest-muted/10 p-4">
    <p className="text-sm text-nest-muted">Card footer</p>
  </div>
</Paper>
```

### Form Container

```tsx
<Paper elevation={2} className="p-6 max-w-md mx-auto">
  <h3 className="mb-4 text-lg font-semibold">Login</h3>
  <div className="space-y-4">
    <div>
      <label className="mb-1 block text-sm font-medium">Email</label>
      <input
        type="email"
        className="w-full rounded-nest-md border border-nest-border p-2"
        placeholder="you@example.com"
      />
    </div>
    <div>
      <label className="mb-1 block text-sm font-medium">Password</label>
      <input
        type="password"
        className="w-full rounded-nest-md border border-nest-border p-2"
        placeholder="••••••••"
      />
    </div>
    <button className="w-full rounded-nest-md bg-nest-primary px-4 py-2 text-white">
      Sign In
    </button>
  </div>
</Paper>
```

### Nested Papers

```tsx
<Paper elevation={3} className="p-6">
  <h2 className="mb-4 text-xl font-semibold">Outer Paper</h2>
  <Paper elevation={1} className="p-4">
    <h3 className="mb-2 font-medium">Inner Paper</h3>
    <p className="text-sm text-nest-muted">
      Lower elevation for visual hierarchy
    </p>
  </Paper>
</Paper>
```

## Elevation Guide

| Elevation | Shadow Class | Use Case |
|-----------|--------------|----------|
| 0 | `shadow-none` | Flat surfaces, embedded content |
| 1 | `shadow-sm` | Subtle elevation (default) |
| 2 | `shadow` | Cards, panels |
| 3 | `shadow-md` | Elevated panels, dropdowns |
| 4 | `shadow-lg` | Modals, popovers |

## Accessibility

- Use semantic `component` values (`article`, `section`, `aside`)
- When using `component="button"` or `component="a"`, ensure proper focus states
- Ensure sufficient color contrast between Paper background and content
- Consider keyboard navigation for interactive Papers
