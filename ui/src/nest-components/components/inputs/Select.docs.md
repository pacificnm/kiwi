# Select

A dropdown for choosing a single option from a list, positioned with floating-ui.

## When to Use

- Choosing one value from a known, moderate-length set of options.
- When a native `<select>` doesn't give enough styling/label control.
- For free-text filtering over many options, use [Autocomplete](./Autocomplete.md) instead.

## Variants

| Prop   | Values               | Default    | Effect                                  |
| ------ | -------------------- | ---------- | --------------------------------------- |
| `size` | `small` \| `medium`  | `medium`   | Trigger padding / font size             |
| `error`| `boolean`            | `false`    | Red border error state                  |

## Props

| Prop           | Type                        | Default      | Description                                             |
| -------------- | --------------------------- | ------------ | ------------------------------------------------------ |
| `value`        | `string`                    | —            | Selected value (controlled).                           |
| `defaultValue` | `string`                    | —            | Initial value (uncontrolled).                          |
| `onChange`     | `(value: string) => void`   | —            | Fired when the selection changes.                      |
| `options`      | `SelectOption[]`            | `[]`         | `{ value, label, disabled? }` items to render.         |
| `children`     | `ReactNode`                 | —            | Extra content rendered below `options` in the listbox. |
| `placeholder`  | `string`                    | `'Select...'`| Shown when no value is selected.                       |
| `label`        | `ReactNode`                 | —            | Label rendered above the trigger.                      |
| `disabled`     | `boolean`                   | `false`      | Disables the trigger.                                  |
| `error`        | `boolean`                   | `false`      | Error styling.                                         |
| `size`         | `'small' \| 'medium'`       | `'medium'`   | Trigger size.                                          |
| `className`    | `string`                    | —            | Extra classes for the trigger button.                 |

## Examples

```tsx
import { Select } from '@nest/components';

<Select
  label="Framework"
  value={value}
  onChange={setValue}
  options={[
    { value: 'react', label: 'React' },
    { value: 'vue', label: 'Vue' },
    { value: 'svelte', label: 'Svelte', disabled: true },
  ]}
/>
```

## Accessibility

- Trigger is a `button` with `aria-haspopup="listbox"` and `aria-expanded`.
- Dropdown has `role="listbox"`; each option has `role="option"` and `aria-selected`.
- Closes on outside click and Escape (via floating-ui `useDismiss`).
- Disabled options are not selectable and are visibly dimmed.
