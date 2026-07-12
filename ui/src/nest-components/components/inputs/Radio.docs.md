# Radio, RadioGroup

A set of mutually exclusive options where only one can be selected at a time.

## When to Use

Use Radio/RadioGroup when:
- Users must select exactly one option from multiple choices
- Options are mutually exclusive (selecting one deselects others)
- All options should be visible at once for comparison

For multiple selections, use Checkbox instead.

## Props

### RadioGroup

| Prop         | Type                                           | Default      | Description                                      |
|--------------|------------------------------------------------|--------------|--------------------------------------------------|
| `value`      | `string`                                       | -            | The selected value (controlled)                  |
| `defaultValue`| `string`                                      | -            | Default selected value (uncontrolled)            |
| `onChange`   | `(value: string, event: ChangeEvent) => void`  | -            | Callback when selection changes                  |
| `name`       | `string`                                       | -            | Name attribute for the radio inputs              |
| `disabled`   | `boolean`                                      | `false`      | If true, all radios are disabled                 |
| `color`      | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info'` | `'primary'` | The color of the radios |
| `size`       | `'small' \| 'medium'`                          | `'medium'`   | The size of the radios                           |
| `row`        | `boolean`                                      | `false`      | If true, displays radios in a row                |
| `className`  | `string`                                       | -            | Additional CSS classes                           |

### Radio

| Prop            | Type                                           | Default | Description                                    |
|-----------------|------------------------------------------------|---------|------------------------------------------------|
| `value`         | `string`                                       | -       | The value of the radio                         |
| `checked`       | `boolean`                                      | -       | Whether the radio is checked (controlled)      |
| `defaultChecked`| `boolean`                                      | `false` | Default checked state (uncontrolled)           |
| `onChange`      | `(event: ChangeEvent<HTMLInputElement>) => void` | -     | Callback when radio is checked                 |
| `label`         | `ReactNode`                                    | -       | Label displayed next to the radio              |
| `color`         | `RadioColor`                                   | -       | Overrides group color                          |
| `size`          | `'small' \| 'medium'`                          | -       | Overrides group size                           |
| `disabled`      | `boolean`                                      | `false` | If true, the radio is disabled                 |
| `className`     | `string`                                       | -       | Additional CSS classes                         |

## Examples

### Basic Usage

```tsx
import { Radio, RadioGroup } from '@nest/components';

// Controlled
const [value, setValue] = useState('one');
<RadioGroup value={value} onChange={(v) => setValue(v)}>
  <Radio value="one" label="Option One" />
  <Radio value="two" label="Option Two" />
</RadioGroup>

// Uncontrolled
<RadioGroup defaultValue="a">
  <Radio value="a" label="A" />
  <Radio value="b" label="B" />
</RadioGroup>
```

### Row Layout

```tsx
import { Radio, RadioGroup } from '@nest/components';

<RadioGroup row defaultValue="1">
  <Radio value="1" label="First" />
  <Radio value="2" label="Second" />
  <Radio value="3" label="Third" />
</RadioGroup>
```

### Colors

```tsx
import { Radio, RadioGroup } from '@nest/components';

<RadioGroup row color="secondary">
  <Radio value="s" label="Secondary" defaultChecked />
</RadioGroup>

<RadioGroup row color="success">
  <Radio value="g" label="Success" defaultChecked />
</RadioGroup>

<RadioGroup row color="error">
  <Radio value="e" label="Error" defaultChecked />
</RadioGroup>
```

### Sizes

```tsx
import { Radio, RadioGroup } from '@nest/components';

<RadioGroup row size="small">
  <Radio value="1" label="Small" defaultChecked />
  <Radio value="2" label="Small" />
</RadioGroup>

<RadioGroup row size="medium">
  <Radio value="1" label="Medium" defaultChecked />
  <Radio value="2" label="Medium" />
</RadioGroup>
```

### Disabled State

```tsx
import { Radio, RadioGroup } from '@nest/components';

// Entire group disabled
<RadioGroup disabled>
  <Radio value="one" label="Disabled" defaultChecked />
  <Radio value="two" label="Also disabled" />
</RadioGroup>

// Individual radio disabled
<RadioGroup>
  <Radio value="one" label="Disabled" disabled />
  <Radio value="two" label="Enabled" />
</RadioGroup>
```

### Without Labels

```tsx
import { Radio, RadioGroup } from '@nest/components';

<RadioGroup row defaultValue="1">
  <Radio value="1" aria-label="Option 1" />
  <Radio value="2" aria-label="Option 2" />
  <Radio value="3" aria-label="Option 3" />
</RadioGroup>
```

### In a Form

```tsx
import { Radio, RadioGroup } from '@nest/components';

function ContactForm() {
  return (
    <form>
      <fieldset className="border border-nest-border rounded-lg p-4">
        <legend className="text-sm font-medium px-2">Contact Method</legend>
        <RadioGroup name="contact" defaultValue="email">
          <Radio value="email" label="Email" />
          <Radio value="phone" label="Phone" />
          <Radio value="sms" label="SMS" />
        </RadioGroup>
      </fieldset>
    </form>
  );
}
```

## Accessibility

- RadioGroup uses `role="radiogroup"` for proper screen reader announcement
- Each Radio is a native `<input type="radio">` for full accessibility
- When using without visible labels, always provide `aria-label`
- For form contexts, use `fieldset` and `legend` to group related radios
- Keyboard navigation: Arrow keys move between options, Space selects
- Always associate a visible label with each radio option

```tsx
// Good: Visible labels
<Radio value="email" label="Email" />

// Good: aria-label for icon-only radios
<Radio value="1" aria-label="Option 1" />

// Best: Semantic grouping in forms
<fieldset>
  <legend>Payment Method</legend>
  <RadioGroup name="payment">
    <Radio value="card" label="Credit Card" />
    <Radio value="bank" label="Bank Transfer" />
  </RadioGroup>
</fieldset>
```
