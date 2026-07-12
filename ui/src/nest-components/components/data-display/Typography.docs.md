# Typography

Display text with consistent styling following MUI Typography API conventions.

## When to Use

Use `Typography` for all text content in your application. It provides semantic HTML elements with consistent styling based on the `variant` prop.

## Variants

| Variant | Element | Description |
|---------|---------|-------------|
| `h1` | `<h1>` | Large display headings |
| `h2` | `<h2>` | Medium display headings |
| `h3` | `<h3>` | Section headings |
| `h4` | `<h4>` | Subsection headings |
| `h5` | `<h5>` | Card titles |
| `h6` | `<h6>` | Group titles |
| `subtitle1` | `<h6>` | Secondary headings (muted) |
| `subtitle2` | `<h6>` | Smaller secondary headings |
| `body1` | `<p>` | Default body text (default) |
| `body2` | `<p>` | Smaller body text |
| `caption` | `<span>` | Small annotations |
| `overline` | `<span>` | Uppercase labels |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `'h1' \| 'h2' \| 'h3' \| 'h4' \| 'h5' \| 'h6' \| 'subtitle1' \| 'subtitle2' \| 'body1' \| 'body2' \| 'caption' \| 'overline'` | `'body1'` | The variant of the typography |
| `align` | `'inherit' \| 'left' \| 'center' \| 'right' \| 'justify'` | `'inherit'` | Text alignment |
| `color` | `'primary' \| 'secondary' \| 'foreground' \| 'muted' \| 'error' \| 'success' \| 'warning'` | `'foreground'` | Text color |
| `gutterBottom` | `boolean` | `false` | Adds bottom margin for spacing |
| `noWrap` | `boolean` | `false` | Truncates text with ellipsis |
| `component` | `ElementType` | (auto) | Override rendered element |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Basic Usage

```tsx
import { Typography } from '@nest/components';

<Typography variant="h1">Page Title</Typography>
<Typography variant="body1">Main content text.</Typography>
```

### Headings

```tsx
<Typography variant="h1">Heading 1</Typography>
<Typography variant="h2">Heading 2</Typography>
<Typography variant="h3">Heading 3</Typography>
<Typography variant="h4">Heading 4</Typography>
<Typography variant="h5">Heading 5</Typography>
<Typography variant="h6">Heading 6</Typography>
```

### Colors

```tsx
<Typography color="primary">Primary text</Typography>
<Typography color="error">Error message</Typography>
<Typography color="muted">Secondary information</Typography>
```

### Truncation

```tsx
<div className="max-w-xs">
  <Typography noWrap>
    This long text will truncate with an ellipsis
  </Typography>
</div>
```

### Component Override

```tsx
{/* Render as <p> but style as h1 */}
<Typography variant="h1" component="p">
  Semantic paragraph styled as heading
</Typography>
```

## Accessibility

- Variants `h1`–`h6` render semantic heading elements (`<h1>`–`<h6>`)
- Use `component` prop to override semantics if needed for document structure
- `noWrap` truncation should include `title` attribute or tooltip for full text access
- Ensure sufficient color contrast when using colored variants
