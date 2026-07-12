# CircularProgress

A circular progress indicator component showing either indeterminate (spinning) or determinate (percentage) progress.

## When to Use

Use `CircularProgress` for:
- Loading states (indeterminate)
- File upload/download progress (determinate)
- Form submission progress (determinate)
- Data fetching indicators (indeterminate)
- Task completion percentage (determinate)

## Variants

| Variant | Description |
|---------|-------------|
| `indeterminate` (default) | Continuous spinning animation |
| `determinate` | Shows progress percentage (0-100) |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'span'` | HTML element to render as |
| `variant` | `'indeterminate' \| 'determinate'` | `'indeterminate'` | Progress type |
| `color` | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info' \| 'inherit'` | `'primary'` | Color of the indicator |
| `size` | `'small' \| 'medium' \| 'large' \| number` | `'medium'` | Size in pixels or preset |
| `value` | `number` | `0` | Progress value (0-100) for determinate |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Indeterminate (Default)

```tsx
import { CircularProgress } from '@nest/components';

// Loading state
<CircularProgress />
```

### Determinate

```tsx
// Shows 50% progress
<CircularProgress variant="determinate" value={50} />
```

### Size Variants

```tsx
<CircularProgress size="small" />
<CircularProgress size="medium" />
<CircularProgress size="large" />

// Custom size
<CircularProgress size={48} />
```

### Color Variants

```tsx
<CircularProgress color="primary" />
<CircularProgress color="secondary" />
<CircularProgress color="success" />
<CircularProgress color="warning" />
<CircularProgress color="error" />
<CircularProgress color="info" />

// Inherit parent color
<CircularProgress color="inherit" />
```

### With Percentage Label

```tsx
<div className="relative">
  <CircularProgress variant="determinate" value={60} size="large" />
  <span className="absolute inset-0 flex items-center justify-center text-xs">
    60%
  </span>
</div>
```

### Loading State

```tsx
<div className="flex items-center gap-2">
  <CircularProgress size="small" />
  <span>Loading...</span>
</div>
```

### Centered Loading

```tsx
<div className="flex h-full items-center justify-center">
  <CircularProgress size="large" />
</div>
```

### File Upload Progress

```tsx
function FileUpload({ progress }) {
  return (
    <div className="space-y-2">
      <div className="flex justify-between text-sm">
        <span>Uploading...</span>
        <span>{progress}%</span>
      </div>
      <CircularProgress variant="determinate" value={progress} />
    </div>
  );
}
```

### Form Submission

```tsx
const [submitting, setSubmitting] = useState(false);

<button disabled={submitting} onClick={handleSubmit}>
  {submitting ? (
    <CircularProgress size="small" color="inherit" />
  ) : (
    'Submit'
  )}
</button>
```

## Size Guide

| Size | Pixels | Use Case |
|------|--------|----------|
| `small` | 16px | Inline with text, buttons |
| `medium` | 32px | Default, cards, panels |
| `large` | 48px | Centered loading screens |
| `number` | Custom | Specific design needs |

## Accessibility

- `role="progressbar"` is applied automatically
- `aria-valuenow` is set for determinate variant
- Indeterminate progress does not have `aria-valuenow`
- Add `aria-label` for context (e.g., "Loading content")

```tsx
<CircularProgress aria-label="Loading dashboard" />
<CircularProgress 
  variant="determinate" 
  value={50} 
  aria-label="Upload progress" 
/>
```

## Styling

CircularProgress uses:
- `text-*` color classes for theming
- `animate-spin` for indeterminate animation
- `transition-all duration-300 ease-out` for determinate progress

### Custom Styling

```tsx
// Custom color via className
<CircularProgress className="text-purple-500" />

// Custom size and color
<CircularProgress size={40} className="text-custom-color" />

// Inherit from parent
<div className="text-nest-success">
  <CircularProgress color="inherit" />
</div>
```

## Best Practices

1. **Use indeterminate for unknown duration** - When you don't know how long something will take
2. **Use determinate for measurable progress** - File uploads, downloads, multi-step processes
3. **Provide context** - Add labels or surrounding text explaining what's loading
4. **Consider size** - Use small for inline, large for full-screen loading
5. **Match brand colors** - Use appropriate color for the context (success for completion, etc.)
