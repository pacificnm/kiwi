# Autocomplete

A text input with a filterable dropdown of suggestions.

## When to Use

- Selecting one value from a large list where typing to filter is faster than scrolling.
- When you need a text filter on top of a [Select](./Select.md)-style dropdown.
- For a simple fixed list with no filtering, prefer [Select](./Select.md).

> **Note:** the current implementation uses simple absolute positioning and an
> outside-click handler. A floating-ui upgrade (portal, collision handling, keyboard
> list navigation) is planned — see the port plan's Tier 3 notes.

## Props

| Prop            | Type                                        | Default         | Description                                          |
| --------------- | ------------------------------------------- | --------------- | ---------------------------------------------------- |
| `options`       | `AutocompleteOption[]`                      | —               | `{ value, label, description?, disabled? }` items.   |
| `value`         | `string`                                    | —               | Selected value (controlled).                         |
| `defaultValue`  | `string`                                    | —               | Initial value (uncontrolled).                        |
| `onChange`      | `(value: string) => void`                   | —               | Fired when an option is selected.                    |
| `onInputChange` | `(value: string) => void`                   | —               | Fired as the filter text changes.                    |
| `placeholder`   | `string`                                    | —               | Input placeholder.                                   |
| `label`         | `string`                                    | —               | Field label.                                         |
| `helperText`    | `string`                                    | —               | Helper text under the field.                         |
| `disabled`      | `boolean`                                   | `false`         | Disables the field.                                  |
| `required`      | `boolean`                                   | `false`         | Marks the field required.                            |
| `error`         | `boolean`                                   | `false`         | Error state (pairs with `errorMessage`).             |
| `errorMessage`  | `string`                                    | —               | Message shown when `error` is set.                   |
| `loading`       | `boolean`                                   | `false`         | Shows a loading row in the dropdown.                 |
| `noOptionsText` | `string`                                    | `'No options'`  | Empty-state text.                                    |
| `maxItems`      | `number`                                    | `8`             | Max options rendered at once.                        |
| `renderOption`  | `(option) => ReactNode`                     | —               | Custom option renderer.                              |
| `popoverClassName` | `string`                                 | —               | Extra classes for the dropdown panel.                |

## Examples

```tsx
import { Autocomplete } from '@nest/components';

<Autocomplete
  label="Language"
  options={[
    { value: 'ts', label: 'TypeScript' },
    { value: 'rs', label: 'Rust', description: 'Systems language' },
    { value: 'go', label: 'Go' },
  ]}
  onChange={setValue}
/>
```

## Accessibility

- The input carries `role="combobox"`, `aria-expanded`, `aria-controls`, and `aria-autocomplete="list"`.
- The dropdown is `role="listbox"`; options are `role="option"` with `aria-selected`.
- `ArrowDown` opens the list; `Escape` closes it; clicking outside dismisses it.
- Full roving-tabindex keyboard navigation is pending the floating-ui upgrade noted above.
