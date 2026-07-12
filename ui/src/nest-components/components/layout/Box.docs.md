# Box

A flexible layout primitive that renders a configurable HTML element.

## When to Use

Use `Box` as an escape-hatch component when:
- You need a custom HTML element not provided by other layout components
- You need a simple wrapper with className merging
- You're building higher-level components that need element flexibility
- Other layout components (`Stack`, `Grid`, `Container`) don't fit your needs

For common layouts, prefer the more semantic components:
- Use `Stack` for flex row/column layouts with spacing
- Use `Grid` for CSS grid layouts
- Use `Container` for max-width centered content

## Variants

Box has no variants. It renders as the element specified in the `component` prop.

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | The HTML element to render as |
| `children` | `ReactNode` | - | The content to display |
| `className` | `string` | - | Additional CSS classes (merged with `cn()`) |

## Examples

### Basic Usage

```tsx
import { Box } from '@nest/components';

<Box className="p-4 border">Simple box</Box>
```

### Custom Element

```tsx
// Semantic section
<Box component="section" className="p-6">
  Section content
</Box>

// Inline element
<Box component="span" className="text-primary">
  Inline text
</Box>

// Interactive element
<Box
  component="button"
  className="btn btn-primary"
  onClick={handleClick}
>
  Click me
</Box>
```

### Flex Layout

```tsx
<Box className="flex items-center gap-4">
  <Box className="h-10 w-10 bg-primary rounded" />
  <Box>Content next to icon</Box>
</Box>
```

### Grid Layout

```tsx
<Box className="grid grid-cols-3 gap-4">
  <Box className="col-span-2">Wide item</Box>
  <Box>Narrow item</Box>
</Box>
```

### Stack Pattern

```tsx
<Box className="flex flex-col gap-2">
  <Box>Item 1</Box>
  <Box>Item 2</Box>
  <Box>Item 3</Box>
</Box>
```

### With Accessibility Props

```tsx
// As a button
<Box
  component="button"
  role="button"
  aria-label="Submit form"
  onClick={handleSubmit}
>
  Submit
</Box>

// As a landmark
<Box component="nav" aria-label="Main navigation">
  {/* Navigation links */}
</Box>

// As a time element
<Box component="time" dateTime="2024-01-15">
  January 15, 2024
</Box>
```

### Nested Boxes

```tsx
<Box className="border p-4">
  <Box className="border p-4">
    <Box className="border p-4">
      Nested content
    </Box>
  </Box>
</Box>
```

## Accessibility

- Use semantic `component` values when possible (`section`, `article`, `nav`, etc.)
- Add appropriate ARIA attributes when using Box for interactive elements
- Ensure proper heading hierarchy when using heading elements
- Box itself has no inherent accessibility features — it inherits from the rendered element
