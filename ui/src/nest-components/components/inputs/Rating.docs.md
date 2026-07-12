# Rating

A star rating input component for collecting user ratings.

## When to Use

Use Rating when:
- Collecting user feedback or reviews
- Allowing users to rate products, services, or content
- Displaying average ratings (read-only mode)
- Building review forms or surveys

## Props

| Prop         | Type                                      | Default     | Description                              |
|--------------|-------------------------------------------|-------------|------------------------------------------|
| `value`      | `number`                                  | -           | The rating value (controlled)            |
| `defaultValue`| `number`                                 | -           | Default rating value (uncontrolled)      |
| `onChange`   | `(event, value: number) => void`          | -           | Callback when rating changes             |
| `max`        | `number`                                  | `5`         | Maximum rating value                     |
| `min`        | `number`                                  | `0`         | Minimum rating value                     |
| `precision`  | `number`                                  | `1`         | Step increment (0.5 for half stars)      |
| `readOnly`   | `boolean`                                 | `false`     | If true, rating is read-only             |
| `disabled`   | `boolean`                                 | `false`     | If true, rating is disabled              |
| `size`       | `'small' \| 'medium' \| 'large'`          | `'medium'`  | The size of the rating                   |
| `color`      | `'primary' \| 'secondary' \| 'accent' \| 'warning' \| 'error'` | `'warning'` | The color of filled stars |
| `emptyIcon`  | `boolean`                                 | `false`     | If true, shows empty star outlines       |
| `className`  | `string`                                  | -           | Additional CSS classes                   |

## Examples

### Basic Usage

```tsx
import { Rating } from '@nest/components';
import { useState } from 'react';

// Uncontrolled
<Rating defaultValue={3} />

// Controlled
const [value, setValue] = useState(3);
<Rating value={value} onChange={(e, v) => setValue(v)} />
```

### Read-only

```tsx
import { Rating } from '@nest/components';

// Display only
<Rating value={4.5} readOnly />

// Product rating display
<div className="flex items-center gap-2">
  <Rating value={5} readOnly />
  <span>5.0 (128 reviews)</span>
</div>
```

### Sizes

```tsx
import { Rating } from '@nest/components';

<Rating defaultValue={3} size="small" />
<Rating defaultValue={3} size="medium" />
<Rating defaultValue={3} size="large" />
```

### Colors

```tsx
import { Rating } from '@nest/components';

<Rating defaultValue={4} color="primary" />
<Rating defaultValue={4} color="secondary" />
<Rating defaultValue={4} color="warning" />
<Rating defaultValue={4} color="error" />
<Rating defaultValue={4} color="accent" />
```

### Disabled State

```tsx
import { Rating } from '@nest/components';

<Rating defaultValue={3} disabled />
```

### Custom Max Value

```tsx
import { Rating } from '@nest/components';

// 10-star rating
<Rating defaultValue={7} max={10} />

// 3-star rating
<Rating defaultValue={2} max={3} />
```

### Half Star Precision

```tsx
import { Rating } from '@nest/components';

<Rating defaultValue={3.5} precision={0.5} />
```

### Review Form

```tsx
import { useState } from 'react';
import { Rating, TextField, Button } from '@nest/components';

function ReviewForm() {
  const [rating, setRating] = useState(0);
  const [review, setReview] = useState('');

  return (
    <form className="space-y-4">
      <div>
        <label className="block text-sm font-medium mb-1">Rating</label>
        <Rating value={rating} onChange={(e, v) => setRating(v)} />
      </div>
      <div>
        <label className="block text-sm font-medium mb-1">Review</label>
        <textarea
          value={review}
          onChange={(e) => setReview(e.target.value)}
          className="w-full border rounded-md px-3 py-2"
          rows={3}
          placeholder="Share your experience..."
        />
      </div>
      <Button type="submit" disabled={rating === 0}>
        Submit Review
      </Button>
    </form>
  );
}
```

## Accessibility

- Rating uses `role="radiogroup"` with individual stars as `role="radio"`
- Each star has `aria-label` indicating its value
- Keyboard navigation: Tab to focus, Arrow keys to change value, Enter/Space to select
- Disabled ratings have `aria-disabled="true"`
- Read-only ratings should have `aria-readonly="true"`

## Keyboard Navigation

| Key          | Action                        |
|--------------|-------------------------------|
| Tab          | Focus the rating              |
| Arrow Left   | Decrease rating by 1          |
| Arrow Right  | Increase rating by 1          |
| Enter/Space  | Select current star           |

## Tips

- Use `readOnly` for displaying existing ratings without allowing changes
- Use `precision={0.5}` to enable half-star ratings
- For review forms, consider requiring a minimum rating before submission
- Pair with text labels for better accessibility
