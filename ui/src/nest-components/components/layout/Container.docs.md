# Container

A centered layout wrapper that constrains content width for better readability.

## When to Use

Use `Container` for:
- Page-level content wrappers
- Centering content horizontally
- Constraining max-width for readability
- Creating consistent page layouts

For more complex layouts, combine with:
- `Grid` for internal grid layouts
- `Stack` for vertical/horizontal stacking

## Variants

| maxWidth | Description | Tailwind Class |
|----------|-------------|----------------|
| `sm` | Small | `max-w-screen-sm` (640px) |
| `md` | Medium | `max-w-screen-md` (768px) |
| `lg` | Large (default) | `max-w-screen-lg` (1024px) |
| `xl` | Extra Large | `max-w-screen-xl` (1280px) |
| `xxl` | Double Extra Large | `max-w-screen-2xl` (1536px) |
| `false` | Full width | `max-w-full` |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `maxWidth` | `'sm' \| 'md' \| 'lg' \| 'xl' \| 'xxl' \| false` | `'lg'` | Maximum width breakpoint |
| `disableGutters` | `boolean` | `false` | Remove horizontal padding |
| `fixed` | `boolean` | `false` | Use fixed width instead of max-width |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Default Container

```tsx
import { Container } from '@nest/components';

<Container>
  <h1>Page Title</h1>
  <p>Content constrained to max-w-screen-lg</p>
</Container>
```

### Different Max Widths

```tsx
<Container maxWidth="sm">
  <p>Narrow content (640px max)</p>
</Container>

<Container maxWidth="md">
  <p>Medium width content (768px max)</p>
</Container>

<Container maxWidth="xl">
  <p>Wide content (1280px max)</p>
</Container>
```

### Full Width

```tsx
<Container maxWidth={false}>
  <p>Full width content (no max-width constraint)</p>
</Container>
```

### Disable Gutters

```tsx
<Container disableGutters>
  <p>Edge-to-edge content (no horizontal padding)</p>
</Container>
```

### Fixed Width

```tsx
<Container maxWidth="md" fixed>
  <p>Fixed width at 768px (won't shrink on smaller screens)</p>
</Container>
```

### Semantic Elements

```tsx
<Container component="header" maxWidth="lg">
  <Navigation />
</Container>

<Container component="main" maxWidth="lg">
  <PageContent />
</Container>

<Container component="footer" maxWidth="lg">
  <Footer />
</Container>
```

### Page Layout

```tsx
<div className="min-h-screen">
  <Container component="header" maxWidth="lg" className="border-b">
    <Header />
  </Container>

  <Container component="main" maxWidth="lg" className="py-8">
    <Article />
  </Container>

  <Container component="footer" maxWidth="lg" className="border-t">
    <Footer />
  </Container>
</div>
```

### With Grid

```tsx
<Container maxWidth="xl">
  <Grid container spacing={3}>
    <Grid size={{ xs: 12, md: 8 }}>
      <MainContent />
    </Grid>
    <Grid size={{ xs: 12, md: 4 }}>
      <Sidebar />
    </Grid>
  </Grid>
</Container>
```

## Gutters

By default, Container applies `px-4` (1rem/16px) horizontal padding for consistent spacing from the viewport edges.

```tsx
// With gutters (default)
<Container>Content with padding</Container>

// Without gutters
<Container disableGutters>Edge-to-edge content</Container>
```

## Fixed vs Max Width

| Prop | Behavior | Use Case |
|------|----------|----------|
| `maxWidth="lg"` (default) | Content shrinks on smaller screens | Responsive layouts |
| `maxWidth="lg" fixed` | Content stays at fixed width | Dashboards, data tables |

## Accessibility

- Use semantic `component` values (`main`, `header`, `footer`, `section`)
- Add `aria-label` for landmark regions when needed
- Ensure sufficient color contrast for content inside containers
- Consider `disableGutters` for touch targets near edges
