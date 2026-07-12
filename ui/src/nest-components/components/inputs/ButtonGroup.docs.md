# ButtonGroup

A component that groups multiple buttons together as a single unit.

## When to Use

Use ButtonGroup when you have related actions that should be visually connected. Common use cases include:
- Text formatting tools (bold, italic, underline)
- View toggles (list/grid/table)
- Pagination controls
- Related action buttons that belong together

## Variants

| Variant     | Description                        | Use Case             |
|-------------|------------------------------------|----------------------|
| `text`      | Minimal styling, no border         | Subtle button groups |
| `outlined`  | Border around the group            | Standard use         |
| `contained` | Shadow and filled appearance       | Prominent actions    |

## Props

| Prop          | Type                                          | Default       | Description                          |
|---------------|-----------------------------------------------|---------------|--------------------------------------|
| `variant`     | `'text' \| 'outlined' \| 'contained'`         | `'text'`      | The variant of the button group      |
| `color`       | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info' \| 'inherit'` | `'primary'` | The color of the buttons |
| `size`        | `'small' \| 'medium' \| 'large'`              | `'medium'`    | The size of the buttons              |
| `orientation` | `'horizontal' \| 'vertical'`                  | `'horizontal'`| The orientation of the group         |
| `fullWidth`   | `boolean`                                     | `false`       | If true, the group takes full width  |
| `component`   | `ElementType`                                 | `'div'`       | The component to render as           |
| `className`   | `string`                                      | -             | Additional CSS classes               |
| `children`    | `ReactNode`                                   | -             | Button components                    |

## Examples

### Basic Usage

```tsx
import { ButtonGroup, Button } from '@nest/components';

// Default button group
<ButtonGroup>
  <Button>One</Button>
  <Button>Two</Button>
  <Button>Three</Button>
</ButtonGroup>
```

### Outlined Variant

```tsx
import { ButtonGroup, Button } from '@nest/components';

<ButtonGroup variant="outlined">
  <Button>Left</Button>
  <Button>Center</Button>
  <Button>Right</Button>
</ButtonGroup>
```

### Contained with Color

```tsx
import { ButtonGroup, Button } from '@nest/components';

// Primary (default)
<ButtonGroup variant="contained">
  <Button>Action 1</Button>
  <Button>Action 2</Button>
</ButtonGroup>

// Secondary color
<ButtonGroup variant="contained" color="secondary">
  <Button>Action 1</Button>
  <Button>Action 2</Button>
</ButtonGroup>

// Success color
<ButtonGroup variant="contained" color="success">
  <Button>Approve</Button>
  <Button>Confirm</Button>
</ButtonGroup>
```

### Vertical Orientation

```tsx
import { ButtonGroup, Button } from '@nest/components';

<ButtonGroup orientation="vertical" variant="outlined">
  <Button>Top</Button>
  <Button>Middle</Button>
  <Button>Bottom</Button>
</ButtonGroup>
```

### Full Width

```tsx
import { ButtonGroup, Button } from '@nest/components';

<div className="w-64">
  <ButtonGroup fullWidth variant="outlined">
    <Button>Stretch</Button>
    <Button>Across</Button>
    <Button>Container</Button>
  </ButtonGroup>
</div>
```

### Different Sizes

```tsx
import { ButtonGroup, Button } from '@nest/components';

// Small
<ButtonGroup size="small" variant="outlined">
  <Button>Small 1</Button>
  <Button>Small 2</Button>
</ButtonGroup>

// Medium (default)
<ButtonGroup size="medium" variant="outlined">
  <Button>Medium 1</Button>
  <Button>Medium 2</Button>
</ButtonGroup>

// Large
<ButtonGroup size="large" variant="outlined">
  <Button>Large 1</Button>
  <Button>Large 2</Button>
</ButtonGroup>
```

## Accessibility

- The ButtonGroup uses `role="group"` to indicate that the buttons are related
- Each button within the group should have accessible labels (especially icon-only buttons)
- When using icon buttons, provide `aria-label` for screen readers:
  ```tsx
  <ButtonGroup>
    <Button aria-label="Bold">B</Button>
    <Button aria-label="Italic">I</Button>
    <Button aria-label="Underline">U</Button>
  </ButtonGroup>
  ```
- Keyboard navigation works naturally - Tab moves between buttons, Enter/Space activates them
