# ToggleButton, ToggleButtonGroup

A group of buttons where users can toggle selection on/off.

## When to Use

Use ToggleButton/ToggleButtonGroup when:
- Users need to select from mutually exclusive options with immediate effect
- Multiple options can be selected simultaneously (like text formatting)
- You want a more compact alternative to radio buttons or checkboxes
- Building toolbars or formatting controls

## Props

### ToggleButtonGroup

| Prop         | Type                     | Default      | Description                                      |
|--------------|--------------------------|--------------|--------------------------------------------------|
| `value`      | `string \| string[]`     | -            | Selected value(s) (controlled)                   |
| `defaultValue`| `string \| string[]`    | -            | Default selected value(s) (uncontrolled)         |
| `onChange`   | `(value) => void`        | -            | Callback when selection changes                  |
| `exclusive`  | `boolean`                | `false`      | If true, only one button can be selected         |
| `disabled`   | `boolean`                | `false`      | If true, all buttons are disabled                |
| `color`      | `ToggleButtonColor`      | `'primary'`  | The color of selected buttons                    |
| `size`       | `'small' \| 'medium' \| 'large'` | `'medium'` | The size of the buttons |
| `row`        | `boolean`                | `false`      | If true, displays buttons in a row               |
| `className`  | `string`                 | -            | Additional CSS classes                           |

### ToggleButton

| Prop       | Type                  | Default | Description                       |
|------------|-----------------------|---------|-----------------------------------|
| `value`    | `string`              | -       | The value of the button           |
| `label`    | `ReactNode`           | -       | Label displayed on the button     |
| `color`    | `ToggleButtonColor`   | -       | Overrides group color             |
| `size`     | `ToggleButtonSize`    | -       | Overrides group size              |
| `disabled` | `boolean`             | `false` | If true, the button is disabled   |
| `className`| `string`              | -       | Additional CSS classes            |

## Examples

### Exclusive Selection (Single)

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

const [align, setAlign] = useState('left');

<ToggleButtonGroup value={align} onChange={(v) => setAlign(v)} exclusive row>
  <ToggleButton value="left" aria-label="Left align">L</ToggleButton>
  <ToggleButton value="center" aria-label="Center align">C</ToggleButton>
  <ToggleButton value="right" aria-label="Right align">R</ToggleButton>
</ToggleButtonGroup>
```

### Multiple Selection

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

const [formats, setFormats] = useState<string[]>(['bold']);

<ToggleButtonGroup value={formats} onChange={(v) => setFormats(v)} exclusive={false} row>
  <ToggleButton value="bold" aria-label="Bold">
    <strong>B</strong>
  </ToggleButton>
  <ToggleButton value="italic" aria-label="Italic">
    <em>I</em>
  </ToggleButton>
  <ToggleButton value="underline" aria-label="Underline">
    <u>U</u>
  </ToggleButton>
</ToggleButtonGroup>
```

### With Labels

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup value={view} onChange={(v) => setView(v)} exclusive row>
  <ToggleButton value="list" label="List View" />
  <ToggleButton value="grid" label="Grid View" />
  <ToggleButton value="tiles" label="Tiles View" />
</ToggleButtonGroup>
```

### Colors

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup defaultValue="primary" color="primary" exclusive row>
  <ToggleButton value="primary" label="Primary" />
</ToggleButtonGroup>

<ToggleButtonGroup defaultValue="success" color="success" exclusive row>
  <ToggleButton value="success" label="Success" />
</ToggleButtonGroup>

<ToggleButtonGroup defaultValue="error" color="error" exclusive row>
  <ToggleButton value="error" label="Error" />
</ToggleButtonGroup>
```

### Sizes

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup size="small" defaultValue="one" exclusive row>
  <ToggleButton value="one" label="Small" />
  <ToggleButton value="two" label="Small" />
</ToggleButtonGroup>

<ToggleButtonGroup size="large" defaultValue="one" exclusive row>
  <ToggleButton value="one" label="Large" />
  <ToggleButton value="two" label="Large" />
</ToggleButtonGroup>
```

### Vertical Layout

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup defaultValue="day" exclusive>
  <ToggleButton value="day" label="Day" />
  <ToggleButton value="week" label="Week" />
  <ToggleButton value="month" label="Month" />
</ToggleButtonGroup>
```

### Text Formatting Toolbar

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

function FormattingToolbar() {
  return (
    <div className="border rounded-md p-2 inline-flex flex-col gap-2">
      <ToggleButtonGroup defaultValue={['bold']} exclusive={false} row>
        <ToggleButton value="bold" aria-label="Bold">
          <strong>B</strong>
        </ToggleButton>
        <ToggleButton value="italic" aria-label="Italic">
          <em>I</em>
        </ToggleButton>
        <ToggleButton value="underline" aria-label="Underline">
          <u>U</u>
        </ToggleButton>
      </ToggleButtonGroup>
      <ToggleButtonGroup defaultValue="left" exclusive row>
        <ToggleButton value="left" aria-label="Left">≡</ToggleButton>
        <ToggleButton value="center" aria-label="Center">≡</ToggleButton>
        <ToggleButton value="right" aria-label="Right">≡</ToggleButton>
      </ToggleButtonGroup>
    </div>
  );
}
```

### Disabled State

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup disabled defaultValue="one" exclusive row>
  <ToggleButton value="one" label="Disabled One" />
  <ToggleButton value="two" label="Disabled Two" />
</ToggleButtonGroup>
```

### Uncontrolled

```tsx
import { ToggleButton, ToggleButtonGroup } from '@nest/components';

<ToggleButtonGroup defaultValue="first" exclusive row>
  <ToggleButton value="first" label="First" />
  <ToggleButton value="second" label="Second" />
  <ToggleButton value="third" label="Third" />
</ToggleButtonGroup>
```

## Accessibility

- ToggleButtonGroup uses `role="group"` to indicate related buttons
- Each ToggleButton has `aria-pressed` to indicate selection state
- For icon-only buttons, always provide `aria-label`:
  ```tsx
  <ToggleButton value="bold" aria-label="Bold">
    <BoldIcon />
  </ToggleButton>
  ```
- Use `exclusive` for single-selection scenarios (like radio buttons)
- Use `exclusive={false}` for multiple-selection scenarios (like checkboxes)
- Keyboard navigation: Tab between groups, click or Space to toggle

## Comparison with Other Components

| Component        | Selection Type      | Use Case                    |
|------------------|---------------------|-----------------------------|
| ToggleButton     | Immediate toggle    | Toolbars, formatting        |
| Radio/RadioGroup | Single, form-based  | Form selections             |
| Checkbox         | Multiple, form-based| Form multi-selections       |
| ButtonGroup      | Visual grouping     | Related actions, no state   |
