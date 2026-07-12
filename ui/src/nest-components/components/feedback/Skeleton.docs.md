# Skeleton

A placeholder loading state component that shows a simplified version of what will be loaded.

## When to Use

Use Skeleton to indicate that content is loading while maintaining the layout structure. It provides a better user experience than a spinner by showing users what the content will look like.

## Variants

| Variant     | Description                        | Example Use          |
|-------------|------------------------------------|----------------------|
| `text`      | Default, thin horizontal bar       | Loading text content |
| `circular`  | Circle shape                       | Avatar placeholders  |
| `rectangular` | Square corners                   | Image placeholders   |
| `rounded`   | Rounded corners                    | Card placeholders    |

## Props

| Prop        | Type                      | Default     | Description                                    |
|-------------|---------------------------|-------------|------------------------------------------------|
| `variant`   | `'text' \| 'circular' \| 'rectangular' \| 'rounded'` | `'text'` | The variant of the skeleton |
| `animation` | `'pulse' \| 'wave' \| false` | `'pulse'` | The animation to apply |
| `width`     | `string \| number`        | -           | The width of the skeleton |
| `height`    | `string \| number`        | -           | The height of the skeleton |
| `component` | `ElementType`             | `'span'`    | The component to render as |
| `className` | `string`                  | -           | Additional CSS classes |

## Examples

### Basic Usage

```tsx
import { Skeleton } from '@nest/components';

// Default text skeleton
<Skeleton />

// Circular avatar skeleton
<Skeleton variant="circular" width={40} height={40} />

// Rectangular image skeleton
<Skeleton variant="rectangular" width={200} height={100} />
```

### Card Loading State

```tsx
import { Skeleton } from '@nest/components';

function CardSkeleton() {
  return (
    <div className="border rounded-lg p-4">
      <div className="flex items-center gap-3 mb-4">
        <Skeleton variant="circular" width={40} height={40} />
        <div>
          <Skeleton width={120} height={16} />
          <Skeleton width={80} height={12} />
        </div>
      </div>
      <Skeleton variant="rounded" width="100%" height={150} />
    </div>
  );
}
```

### Custom Sizes

```tsx
import { Skeleton } from '@nest/components';

// Using numbers (pixels)
<Skeleton width={100} height={50} />

// Using strings (CSS values)
<Skeleton width="50%" height="2rem" />
```

### Disable Animation

```tsx
import { Skeleton } from '@nest/components';

<Skeleton animation={false} width={200} height={30} />
```

## Accessibility

- Skeleton uses `role="presentation"` by default, indicating it is a decorative placeholder
- The pulse animation is subtle and does not violate WCAG motion guidelines
- When loading content, consider pairing with a live region or status message for screen readers:
  ```tsx
  <div aria-live="polite" aria-busy="true">
    <Skeleton />
    <p>Loading content...</p>
  </div>
  ```
