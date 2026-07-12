# Breadcrumbs

A navigation component that displays the path to the current page.

## When to Use

Use `Breadcrumbs` for:
- Showing navigation hierarchy
- Helping users understand their location in the site
- Providing quick navigation to parent pages
- Deep page navigation (3+ levels deep)

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'nav'` | HTML element to render as |
| `items` | `BreadcrumbItem[]` | - | Array of breadcrumb items |
| `separator` | `ReactNode` | `<ChevronRight />` | Separator between items |
| `maxItems` | `number` | `0` | Max items before collapsing |
| `ariaLabel` | `string` | `'breadcrumb'` | Aria-label for nav element |
| `className` | `string` | - | Additional CSS classes |

### BreadcrumbItem

| Prop | Type | Description |
|------|------|-------------|
| `label` | `ReactNode` | Display text or element |
| `href` | `string` | Link URL (optional) |
| `current` | `boolean` | Is current page (not clickable) |

## Examples

### Basic Breadcrumbs

```tsx
import { Breadcrumbs } from '@nest/components';

<Breadcrumbs items={[
  { label: 'Home', href: '/' },
  { label: 'Products', href: '/products' },
  { label: 'Electronics', current: true }
]} />
```

### Custom Separator

```tsx
// Text separator
<Breadcrumbs items={items} separator="/" />

// Custom icon
import { Slash } from 'lucide-react';
<Breadcrumbs items={items} separator={<Slash />} />

// Arrow
<Breadcrumbs items={items} separator="→" />
```

### Max Items (Collapsing)

```tsx
// Shows +N collapsed indicator when path exceeds maxItems
<Breadcrumbs
  items={[
    { label: 'Home', href: '/' },
    { label: 'Level 1', href: '/1' },
    { label: 'Level 2', href: '/2' },
    { label: 'Level 3', href: '/3' },
    { label: 'Current', current: true }
  ]}
  maxItems={3}
/>
// Displays: +3 / Level 3 / Current
```

### With Icons

```tsx
import { Home } from 'lucide-react';

<Breadcrumbs items={[
  { label: <Home className="h-4 w-4" />, href: '/' },
  { label: 'Products', href: '/products' },
  { label: 'Current', current: true }
]} />
```

### All Links (No Current)

```tsx
<Breadcrumbs items={[
  { label: 'Home', href: '/' },
  { label: 'Products', href: '/products' },
  { label: 'Electronics', href: '/products/electronics' }
]} />
```

### Custom Component

```tsx
// As div instead of nav
<Breadcrumbs items={items} component="div" />
```

### Custom Aria Label

```tsx
<Breadcrumbs
  items={items}
  ariaLabel="You are here"
/>
```

## Accessibility

- Root element is `<nav>` with `aria-label="breadcrumb"`
- Current item has `aria-current="page"`
- Separators have `aria-hidden="true"`
- Links have proper focus states
- Use descriptive labels for better screen reader experience

## Styling

Breadcrumbs uses these base styles:
- `flex items-center gap-1` - Horizontal layout
- `text-sm` - Small text size

Link items have:
- `text-nest-primary hover:text-nest-primary/80` - Primary color with hover
- `hover:underline` - Underline on hover
- `focus:ring-2 focus:ring-nest-primary/50` - Focus ring

Current item has:
- `font-medium text-nest-foreground` - Emphasized text

Non-current plain items have:
- `text-nest-muted` - Muted text color

## Best Practices

1. **Always include Home** - Start with a home link for easy navigation reset
2. **Mark current page** - Use `current: true` for the last item
3. **Use meaningful labels** - Clear, concise labels help navigation
4. **Consider collapsing** - Use `maxItems` for deep hierarchies
5. **Choose separator wisely** - Chevron is standard, slash works for simple paths
