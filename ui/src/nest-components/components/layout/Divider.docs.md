# Divider

A visual separator component that renders a horizontal or vertical line, optionally with content.

## When to Use

Use `Divider` for:
- Separating content sections
- Form separators (e.g., "OR" between login options)
- Toolbar separators between button groups
- Visual breaks in lists or cards
- Labeled section dividers

## Variants

| Orientation | Description |
|-------------|-------------|
| `horizontal` (default) | Horizontal line (border-top) |
| `vertical` | Vertical line (border-left) |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` | Divider direction |
| `fullWidth` | `boolean` | `true` | Full width/height |
| `flexItem` | `boolean` | `false` | Prevent flex shrinking |
| `children` | `ReactNode` | - | Content (text/icon) in divider |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Horizontal Divider

```tsx
import { Divider } from '@nest/components';

<div>
  <p>Content above</p>
  <Divider />
  <p>Content below</p>
</div>
```

### Vertical Divider

```tsx
<div className="flex items-center gap-4">
  <span>Left</span>
  <Divider orientation="vertical" className="h-12" />
  <span>Right</span>
</div>
```

### Text Divider

```tsx
<Divider>OR</Divider>
<Divider>Continue with</Divider>
<Divider>Section 2</Divider>
```

### Icon Divider

```tsx
<Divider>★</Divider>
<Divider><StarIcon /></Divider>
```

### Form Separator

```tsx
<form>
  <TextField label="Email" />
  <Divider>OR</Divider>
  <TextField label="Phone" />
</form>
```

### Card Divider

```tsx
<Paper>
  <div className="p-4">
    <h3>Header</h3>
  </div>
  <Divider />
  <div className="p-4">
    <p>Body content</p>
  </div>
  <Divider />
  <div className="p-4 bg-nest-muted/10">
    <p className="text-sm">Footer</p>
  </div>
</Paper>
```

### Toolbar Divider

```tsx
<div className="flex items-center gap-2 p-2">
  <Button>Bold</Button>
  <Button>Italic</Button>
  <Divider orientation="vertical" className="h-6" />
  <Button>Underline</Button>
  <Button>Strike</Button>
</div>
```

### Custom Element

```tsx
<Divider component="hr" />
<Divider component="li" /> {/* In a list */}
```

### Partial Width

```tsx
<Divider fullWidth={false} className="w-32" />
```

### Flex Item

```tsx
<div className="flex items-center gap-4">
  <span>Content</span>
  <Divider orientation="vertical" flexItem className="h-12" />
  <span>More content</span>
</div>
```

## Accessibility

- Automatically applies `role="separator"` for semantic meaning
- Vertical dividers include `aria-orientation="vertical"`
- When using text dividers, ensure text is descriptive of the section break
- Avoid overusing dividers; white space often suffices

## Styling

Divider uses `border-nest-border` for the line color, which adapts to light/dark themes.

For custom styling, use `className`:

```tsx
<Divider className="border-2 border-nest-primary" />
<Divider className="border-dashed" />
```
