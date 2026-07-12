# LinearProgress

A linear progress indicator component showing either indeterminate (animated) or determinate (percentage) progress.

## When to Use

Use `LinearProgress` for:
- Page loading indicators (indeterminate)
- File upload/download progress (determinate)
- Form submission progress (determinate)
- Multi-step process progress (determinate)
- Buffer/loading ahead (buffer variant)

## Variants

| Variant | Description |
|---------|-------------|
| `indeterminate` (default) | Continuous left-to-right animation |
| `determinate` | Shows progress percentage (0-100) |
| `buffer` | Shows progress with buffer ahead |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'span'` | HTML element to render as |
| `variant` | `'indeterminate' \| 'determinate' \| 'buffer'` | `'indeterminate'` | Progress type |
| `color` | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info' \| 'inherit'` | `'primary'` | Color of the indicator |
| `value` | `number` | `0` | Progress value (0-100) |
| `bufferValue` | `number` | `0` | Buffer value (0-100) for buffer variant |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Indeterminate (Default)

```tsx
import { LinearProgress } from '@nest/components';

// Page loading
<LinearProgress />
```

### Determinate

```tsx
// Shows 50% progress
<LinearProgress variant="determinate" value={50} />
```

### Buffer

```tsx
// Shows 50% progress with 75% buffered
<LinearProgress variant="buffer" value={50} bufferValue={75} />
```

### Color Variants

```tsx
<LinearProgress color="primary" />
<LinearProgress color="secondary" />
<LinearProgress color="success" />
<LinearProgress color="warning" />
<LinearProgress color="error" />
<LinearProgress color="info" />

// Inherit parent color
<div className="text-nest-success">
  <LinearProgress color="inherit" />
</div>
```

### File Upload

```tsx
function FileUpload({ progress }) {
  return (
    <div>
      <div className="flex justify-between text-sm mb-1">
        <span>Uploading file.txt</span>
        <span>{progress}%</span>
      </div>
      <LinearProgress variant="determinate" value={progress} />
    </div>
  );
}
```

### Multiple Downloads

```tsx
<div className="space-y-2">
  {downloads.map(file => (
    <div key={file.id}>
      <div className="flex justify-between text-xs mb-1">
        <span>{file.name}</span>
        <span>{file.progress}%</span>
      </div>
      <LinearProgress 
        variant="determinate" 
        value={file.progress}
        color={file.progress === 100 ? 'success' : 'primary'}
      />
    </div>
  ))}
</div>
```

### Custom Height

```tsx
// Default height
<LinearProgress />

// Thicker progress bar
<LinearProgress className="h-2" />
<LinearProgress className="h-4" />
```

### Page Top Loading

```tsx
// Fixed at top of page
<div className="fixed top-0 left-0 right-0 z-50">
  <LinearProgress />
</div>
```

### Card Loading

```tsx
<Card>
  <LinearProgress />
  <CardContent>
    <p>Card content loading...</p>
  </CardContent>
</Card>
```

## Accessibility

- `role="progressbar"` is applied automatically
- `aria-valuenow` is set for determinate and buffer variants
- Indeterminate progress does not have `aria-valuenow`
- Add `aria-label` for context (e.g., "File upload progress")

```tsx
<LinearProgress aria-label="Page loading" />
<LinearProgress 
  variant="determinate" 
  value={50} 
  aria-label="Upload progress" 
/>
```

## Styling

LinearProgress uses:
- `h-1 w-full rounded-nest-full` - Default dimensions
- `bg-nest-surface` - Track background
- `bg-nest-primary` (or other color) - Progress fill
- `transition-all duration-300 ease-out` - Smooth progress animation
- Custom keyframe animations for indeterminate

### Custom Styling

```tsx
// Custom height
<LinearProgress className="h-2" />

// Custom color via className
<LinearProgress className="[&>div]:bg-purple-500" />

// Rounded or square ends (default is rounded)
<LinearProgress className="rounded-none" />
```

## Buffer Variant

The buffer variant shows two layers:
1. Background buffer (bufferValue) - shows loaded/available content
2. Foreground progress (value) - shows actual progress

Useful for:
- Video streaming (buffered vs played)
- File downloads (downloaded vs installed)
- Data loading (fetched vs processed)

```tsx
<LinearProgress variant="buffer" value={40} bufferValue={80} />
```

## Best Practices

1. **Use indeterminate for unknown duration** - When you don't know how long something will take
2. **Use determinate for measurable progress** - File uploads, downloads, multi-step processes
3. **Show percentage** - Display the numeric value alongside the bar
4. **Use appropriate colors** - Success for completion, warning for slow progress, etc.
5. **Consider placement** - Top of page for global loading, inline for specific items
