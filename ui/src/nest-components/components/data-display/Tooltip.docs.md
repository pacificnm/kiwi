# Tooltip

A popup that displays helpful information when users hover or focus an element.

## When to Use

Use Tooltip when:
- Providing additional context for icons or buttons
- Showing help text for form fields
- Explaining abbreviations or technical terms
- Adding non-essential information that shouldn't clutter the UI

## Props

| Prop         | Type                                      | Default     | Description                              |
|--------------|-------------------------------------------|-------------|------------------------------------------|
| `title`      | `ReactNode`                               | -           | **Required.** The content of the tooltip |
| `children`   | `ReactNode`                               | -           | **Required.** The anchor element         |
| `placement`  | `'top' \| 'bottom' \| 'left' \| 'right' \| 'top-start' \| 'top-end' \| 'bottom-start' \| 'bottom-end' \| 'left-start' \| 'left-end' \| 'right-start' \| 'right-end'` | `'top'` | Position of tooltip |
| `open`       | `boolean`                                 | -           | Controlled open state                    |
| `onOpenChange`| `(open: boolean) => void`                | -           | Callback when open state changes         |
| `enterDelay` | `number`                                  | `0`         | Delay before opening (ms)                |
| `leaveDelay` | `number`                                  | `0`         | Delay before closing (ms)                |
| `arrow`      | `boolean`                                 | `false`     | Show arrow on tooltip                    |
| `className`  | `string`                                  | -           | Additional CSS classes                   |

## Examples

### Basic Usage

```tsx
import { Tooltip, Button } from '@nest/components';

<Tooltip title="Helpful information">
  <Button>Hover me</Button>
</Tooltip>
```

### Placements

```tsx
import { Tooltip, Button } from '@nest/components';

<Tooltip title="Top" placement="top">
  <Button>Top</Button>
</Tooltip>

<Tooltip title="Bottom" placement="bottom">
  <Button>Bottom</Button>
</Tooltip>

<Tooltip title="Left" placement="left">
  <Button>Left</Button>
</Tooltip>

<Tooltip title="Right" placement="right">
  <Button>Right</Button>
</Tooltip>
```

### With Delays

```tsx
import { Tooltip, Button } from '@nest/components';

// Opens after 1 second
<Tooltip title="Delayed open" enterDelay={1000}>
  <Button>Slow Open</Button>
</Tooltip>

// Closes after 2 seconds
<Tooltip title="Delayed close" leaveDelay={2000}>
  <Button>Slow Close</Button>
</Tooltip>
```

### Controlled

```tsx
import { useState } from 'react';
import { Tooltip, Button } from '@nest/components';

function ControlledTooltip() {
  const [open, setOpen] = useState(false);

  return (
    <Tooltip title="Controlled" open={open} onOpenChange={setOpen}>
      <Button onClick={() => setOpen(!open)}>Toggle</Button>
    </Tooltip>
  );
}
```

### With Arrow

```tsx
import { Tooltip, Button } from '@nest/components';

<Tooltip title="With arrow" arrow>
  <Button>Hover me</Button>
</Tooltip>
```

### Icon Buttons

```tsx
import { Tooltip } from '@nest/components';

<Tooltip title="Settings">
  <button aria-label="Settings">⚙️</button>
</Tooltip>

<Tooltip title="Delete">
  <button aria-label="Delete" className="text-error">🗑️</button>
</Tooltip>
```

### Form Field Help

```tsx
import { Tooltip, TextField } from '@nest/components';

<label>
  Email
  <Tooltip title="We'll never share your email">
    <span className="cursor-help">ℹ️</span>
  </Tooltip>
</label>
<TextField placeholder="Enter email" />
```

### Rich Content

```tsx
import { Tooltip, Button } from '@nest/components';

<Tooltip title={<span className="font-bold">Bold text</span>}>
  <Button>Rich content</Button>
</Tooltip>
```

## Accessibility

- Tooltip uses `role="tooltip"` for screen readers
- Trigger elements should have appropriate labels (`aria-label` for icon buttons)
- Tooltip is shown on both hover and focus
- Tooltip dismisses on outside click and Escape key
- Content is announced to screen readers when it appears

## Keyboard Support

| Key    | Action              |
|--------|---------------------|
| Tab    | Focus trigger       |
| Escape | Close tooltip       |

## Tips

- Keep tooltip content concise - one or two short sentences
- Don't put critical information in tooltips (they're hidden by default)
- Use for supplementary information, not primary content
- Avoid nesting tooltips
- For complex content, consider using a Popover instead
