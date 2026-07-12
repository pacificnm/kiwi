# Link

A styled anchor element component for navigation and external links.

## When to Use

Use `Link` for:
- Navigation links within the app
- External website links
- Inline text links
- Breadcrumb navigation
- Button-like links (with custom component)
- Any clickable text that navigates somewhere

## Variants

| Underline | Description |
|-----------|-------------|
| `hover` (default) | Underline appears on hover |
| `none` | No underline |
| `always` | Always underlined |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'a'` | HTML element to render as |
| `href` | `string` | - | URL for the link |
| `underline` | `'none' \| 'hover' \| 'always'` | `'hover'` | Underline behavior |
| `color` | `'primary' \| 'inherit'` | `'primary'` | Link color |
| `external` | `boolean` | `false` | Open in new tab with secure rel |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Basic Link

```tsx
import { Link } from '@nest/components';

<Link href="/page">Click me</Link>
```

### Underline Variants

```tsx
<Link href="/page" underline="hover">Hover to underline</Link>
<Link href="/page" underline="none">No underline</Link>
<Link href="/page" underline="always">Always underlined</Link>
```

### Color Variants

```tsx
<Link href="/page" color="primary">Primary colored link</Link>
<Link href="/page" color="inherit">Inherits parent color</Link>
```

### External Link

```tsx
<Link href="https://example.com" external>
  External Link
</Link>
// Renders with target="_blank" and rel="noopener noreferrer"
```

### Inline Link

```tsx
<p>
  Check out our{' '}
  <Link href="/docs">documentation</Link>{' '}
  for more information.
</p>
```

### Link in Muted Text

```tsx
<p className="text-nest-muted">
  Need help?{' '}
  <Link href="/help" color="inherit">Contact support</Link>
</p>
```

### Navigation Links

```tsx
<nav className="flex gap-4">
  <Link href="/" underline="none">Home</Link>
  <Link href="/about" underline="none">About</Link>
  <Link href="/contact" underline="none">Contact</Link>
</nav>
```

### Breadcrumb Links

```tsx
<div className="flex items-center gap-2">
  <Link href="/">Home</Link>
  <span>/</span>
  <Link href="/products">Products</Link>
  <span>/</span>
  <span>Current Page</span>
</div>
```

### Custom Component

```tsx
// As a button
<Link component="button" onClick={handleClick}>
  Button Link
</Link>

// As a span (for non-navigable clickable text)
<Link component="span" onClick={handleClick}>
  Clickable Text
</Link>

// With React Router
<Link component={RouterLink} to="/page">
  Router Link
</Link>
```

### Link with Icon

```tsx
<Link href="/docs" className="inline-flex items-center gap-1">
  <DocumentIcon />
  <span>Documentation</span>
</Link>

<Link href="/external" external className="inline-flex items-center gap-1">
  <span>External Resource</span>
  <ExternalLinkIcon />
</Link>
```

## Accessibility

- Always provide meaningful `href` for navigation links
- Use `external` prop for external links (adds `rel="noopener noreferrer"`)
- Add `aria-label` for icon-only links or ambiguous link text
- Ensure link text is descriptive ("Read more about X" vs "Click here")
- Focus ring is applied for keyboard navigation

## Styling

Link uses these base styles:
- `cursor-pointer` - Pointer cursor
- `font-nest-body` - Body font family
- `transition-colors duration-150` - Smooth color transitions
- `focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2` - Focus ring
- `rounded-nest-sm` - Rounded focus ring

### Color Tokens

| Color | Classes |
|-------|---------|
| `primary` | `text-nest-primary hover:text-nest-primary/80` |
| `inherit` | `text-inherit hover:text-inherit/80` |

### Custom Styling

```tsx
<Link href="/page" className="font-bold text-lg">
  Bold Large Link
</Link>

<Link href="/page" className="flex items-center gap-2">
  <Icon />
  <span>Link with Icon</span>
</Link>
```

## Best Practices

1. **Use `external` for external links** - Automatically adds security attributes
2. **Choose underline wisely** - `hover` for navigation, `none` for nav bars, `always` for emphasis
3. **Use `color="inherit"` in colored text** - Maintains visual hierarchy
4. **Provide descriptive link text** - Better for accessibility and SEO
5. **Consider custom component for routing** - Works with React Router, Next.js Link, etc.
