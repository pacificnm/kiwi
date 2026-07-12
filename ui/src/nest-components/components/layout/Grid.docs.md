# Grid

A CSS grid layout component for creating responsive 12-column (or custom) grid systems.

## When to Use

Use `Grid` for:
- Responsive page layouts
- Card grids and dashboards
- Form layouts with aligned fields
- Any layout requiring precise column control

For simpler linear layouts, consider:
- `Stack` for flex row/column layouts
- `Box` for custom layouts

## Variants

Grid has two modes:

| Mode | Props | Description |
|------|-------|-------------|
| Container | `container`, `spacing`, `columns` | Creates a grid wrapper |
| Item | `size`, `offset` | Creates a grid cell spanning columns |

## Props

### Container Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `container` | `boolean` | `false` | Enable container mode |
| `columns` | `number` | `12` | Number of columns in the grid |
| `spacing` | `0 \| 1 \| 2 \| 3 \| 4 \| 5 \| 6 \| 8` | `0` | Gap between items |

### Item Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `size` | `number \| 'auto' \| GridSize` | - | Columns to span |
| `offset` | `number \| GridSize` | - | Empty columns before item |

### GridSize Type

```ts
interface GridSize {
  xs?: number | 'auto';  // Mobile (default)
  sm?: number | 'auto';  // Small (≥640px)
  md?: number | 'auto';  // Medium (≥768px)
  lg?: number | 'auto';  // Large (≥1024px)
}
```

### Common Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Basic Grid

```tsx
import { Grid } from '@nest/components';

<Grid container spacing={2}>
  <Grid size={6}>
    <div>Half width</div>
  </Grid>
  <Grid size={6}>
    <div>Half width</div>
  </Grid>
</Grid>
```

### Responsive Grid

```tsx
<Grid container spacing={2}>
  <Grid size={{ xs: 12, md: 6, lg: 4 }}>
    <div>Responsive card</div>
  </Grid>
  <Grid size={{ xs: 12, md: 6, lg: 4 }}>
    <div>Responsive card</div>
  </Grid>
  <Grid size={{ xs: 12, md: 6, lg: 4 }}>
    <div>Responsive card</div>
  </Grid>
</Grid>
```

### Grid with Offset

```tsx
<Grid container spacing={2}>
  <Grid size={3}>
    <div>Sidebar</div>
  </Grid>
  <Grid size={6} offset={3}>
    <div>Content with offset</div>
  </Grid>
</Grid>
```

### Centered Item

```tsx
<Grid container spacing={2}>
  <Grid size={8} offset={2}>
    <div>Centered (8 cols with offset 2)</div>
  </Grid>
</Grid>
```

### Auto Width

```tsx
<Grid container spacing={2}>
  <Grid size="auto">
    <div>Fits content</div>
  </Grid>
  <Grid size={6}>
    <div>6 columns</div>
  </Grid>
  <Grid size="auto">
    <div>Fits content</div>
  </Grid>
</Grid>
```

### Custom Column Count

```tsx
<Grid container spacing={2} columns={6}>
  <Grid size={2}>
    <div>2/6</div>
  </Grid>
  <Grid size={2}>
    <div>2/6</div>
  </Grid>
  <Grid size={2}>
    <div>2/6</div>
  </Grid>
</Grid>
```

### Dashboard Layout

```tsx
<Grid container spacing={3}>
  {/* Header */}
  <Grid size={12}>
    <AppBar title="Dashboard" />
  </Grid>

  {/* Sidebar */}
  <Grid size={{ xs: 12, md: 3 }}>
    <NavMenu />
  </Grid>

  {/* Main Content */}
  <Grid size={{ xs: 12, md: 9 }}>
    <Grid container spacing={2}>
      <Grid size={{ xs: 12, lg: 6 }}>
        <Card title="Stats">...</Card>
      </Grid>
      <Grid size={{ xs: 12, lg: 6 }}>
        <Card title="Chart">...</Card>
      </Grid>
    </Grid>
  </Grid>
</Grid>
```

## Spacing Scale

| spacing | gap class | Pixels |
|---------|-----------|--------|
| 0 | `gap-0` | 0 |
| 1 | `gap-1` | 0.25rem (4px) |
| 2 | `gap-2` | 0.5rem (8px) |
| 3 | `gap-3` | 0.75rem (12px) |
| 4 | `gap-4` | 1rem (16px) |
| 5 | `gap-5` | 1.25rem (20px) |
| 6 | `gap-6` | 1.5rem (24px) |
| 8 | `gap-8` | 2rem (32px) |

## Breakpoints

| Breakpoint | Min Width |
|------------|-----------|
| xs | 0 (default) |
| sm | 640px |
| md | 768px |
| lg | 1024px |

## Accessibility

- Use semantic `component` values for content sections
- Ensure sufficient color contrast in grid items
- Consider keyboard navigation order in complex layouts
- Use appropriate heading hierarchy within grid sections
