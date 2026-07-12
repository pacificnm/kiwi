# Checkbox

A selectable input that allows users to choose one or more options.

## When to Use

Use Checkbox when:
- Users need to select multiple options from a list
- Toggling a single option on/off (like "I agree to terms")
- Creating a group of independent selections

For single-selection from multiple options, use Radio instead.

## Variants

| Variant        | Description                    | Use Case             |
|----------------|--------------------------------|----------------------|
| Default        | Standard checkbox              | Most use cases       |
| Indeterminate  | Shows a dash instead of check  | Parent of partially selected children |

## Props

| Prop            | Type                                           | Default     | Description                                    |
|-----------------|------------------------------------------------|-------------|------------------------------------------------|
| `checked`       | `boolean`                                      | -           | Whether the checkbox is checked (controlled)   |
| `defaultChecked`| `boolean`                                      | `false`     | Default checked state (uncontrolled)           |
| `onChange`      | `(event: ChangeEvent<HTMLInputElement>) => void` | -         | Callback when checked state changes            |
| `indeterminate` | `boolean`                                      | `false`     | If true, shows a dash instead of checkmark     |
| `color`         | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info'` | `'primary'` | The color of the checkbox |
| `size`          | `'small' \| 'medium'`                          | `'medium'`  | The size of the checkbox                       |
| `disabled`      | `boolean`                                      | `false`     | If true, the checkbox is disabled              |
| `className`     | `string`                                       | -           | Additional CSS classes                         |

## Examples

### Basic Usage

```tsx
import { Checkbox } from '@nest/components';

// Uncontrolled
<Checkbox defaultChecked />

// Controlled
const [checked, setChecked] = useState(false);
<Checkbox checked={checked} onChange={(e) => setChecked(e.target.checked)} />
```

### Indeterminate State

```tsx
import { Checkbox } from '@nest/components';

// Indeterminate checkbox (shows dash)
<Checkbox indeterminate />

// Parent checkbox that controls children
function ParentCheckbox() {
  const [children, setChildren] = useState([false, false, false]);
  const allChecked = children.every(Boolean);
  const someChecked = children.some(Boolean);
  
  return (
    <>
      <Checkbox
        checked={allChecked}
        indeterminate={someChecked && !allChecked}
        onChange={(e) => {
          const newChecked = e.target.checked;
          setChildren([newChecked, newChecked, newChecked]);
        }}
      />
      {children.map((checked, i) => (
        <Checkbox
          key={i}
          checked={checked}
          onChange={(e) => {
            const newChildren = [...children];
            newChildren[i] = e.target.checked;
            setChildren(newChildren);
          }}
        />
      ))}
    </>
  );
}
```

### Colors

```tsx
import { Checkbox } from '@nest/components';

<Checkbox defaultChecked color="primary" />
<Checkbox defaultChecked color="secondary" />
<Checkbox defaultChecked color="success" />
<Checkbox defaultChecked color="error" />
<Checkbox defaultChecked color="warning" />
<Checkbox defaultChecked color="info" />
```

### Sizes

```tsx
import { Checkbox } from '@nest/components';

<Checkbox size="small" defaultChecked />
<Checkbox size="medium" defaultChecked />
```

### With Label

```tsx
import { Checkbox, FormLabel } from '@nest/components';

<div className="flex items-center gap-2">
  <Checkbox id="terms" />
  <FormLabel htmlFor="terms">I agree to the terms and conditions</FormLabel>
</div>
```

### Checkbox Group

```tsx
import { Checkbox } from '@nest/components';

function FruitSelector() {
  const [fruits, setFruits] = useState<string[]>([]);
  
  const handleToggle = (fruit: string, checked: boolean) => {
    setFruits(prev => 
      checked ? [...prev, fruit] : prev.filter(f => f !== fruit)
    );
  };
  
  return (
    <div className="space-y-2">
      <div className="flex items-center gap-2">
        <Checkbox
          checked={fruits.includes('apple')}
          onChange={(e) => handleToggle('apple', e.target.checked)}
        />
        <label>Apple</label>
      </div>
      <div className="flex items-center gap-2">
        <Checkbox
          checked={fruits.includes('banana')}
          onChange={(e) => handleToggle('banana', e.target.checked)}
        />
        <label>Banana</label>
      </div>
      <div className="flex items-center gap-2">
        <Checkbox
          checked={fruits.includes('cherry')}
          onChange={(e) => handleToggle('cherry', e.target.checked)}
        />
        <label>Cherry</label>
      </div>
    </div>
  );
}
```

### Disabled State

```tsx
import { Checkbox } from '@nest/components';

<Checkbox disabled />
<Checkbox disabled defaultChecked />
<Checkbox disabled indeterminate />
```

## Accessibility

- Checkbox uses a native `<input type="checkbox">` for full accessibility support
- Always associate labels using `htmlFor` and `id`:
  ```tsx
  <Checkbox id="accept" />
  <label htmlFor="accept">Accept terms</label>
  ```
- For checkbox groups, consider using `fieldset` and `legend`:
  ```tsx
  <fieldset>
    <legend>Select fruits</legend>
    <div>
      <Checkbox id="apple" />
      <label htmlFor="apple">Apple</label>
    </div>
  </fieldset>
  ```
- The `indeterminate` state is visual only - screen readers won't announce it. Use it for parent checkboxes in a tree structure.
- Keyboard users can toggle with Space, and tab between checkboxes.
