# Switch

A toggle input that represents a binary on/off state.

## When to Use

Use Switch when:
- Toggling a single setting on/off (like notifications, dark mode)
- The change takes effect immediately without a submit button
- Users need a clear visual indication of state

Use Checkbox instead when:
- Selecting items from a list
- The change requires a form submission

## Props

| Prop             | Type                                           | Default     | Description                                    |
|------------------|------------------------------------------------|-------------|------------------------------------------------|
| `checked`        | `boolean`                                      | -           | Whether the switch is checked (controlled)     |
| `defaultChecked` | `boolean`                                      | `false`     | Default checked state (uncontrolled)           |
| `onChange`       | `(event: ChangeEvent<HTMLInputElement>) => void` | -         | Callback when checked state changes            |
| `color`          | `'primary' \| 'secondary' \| 'accent' \| 'success' \| 'warning' \| 'error' \| 'info'` | `'primary'` | The color of the switch |
| `size`           | `'small' \| 'medium'`                          | `'medium'`  | The size of the switch                         |
| `disabled`       | `boolean`                                      | `false`     | If true, the switch is disabled                |
| `className`      | `string`                                       | -           | Additional CSS classes                         |

## Examples

### Basic Usage

```tsx
import { Switch } from '@nest/components';

// Uncontrolled
<Switch defaultChecked />

// Controlled
const [checked, setChecked] = useState(false);
<Switch checked={checked} onChange={(e) => setChecked(e.target.checked)} />
```

### Colors

```tsx
import { Switch } from '@nest/components';

<Switch defaultChecked color="primary" />
<Switch defaultChecked color="secondary" />
<Switch defaultChecked color="success" />
<Switch defaultChecked color="error" />
<Switch defaultChecked color="warning" />
<Switch defaultChecked color="info" />
```

### Sizes

```tsx
import { Switch } from '@nest/components';

<Switch size="small" defaultChecked />
<Switch size="medium" defaultChecked />
```

### With Label

```tsx
import { Switch, FormLabel } from '@nest/components';

<div className="flex items-center gap-3">
  <Switch id="notifications" defaultChecked />
  <FormLabel htmlFor="notifications">Enable notifications</FormLabel>
</div>
```

### Settings List

```tsx
import { Switch, FormLabel } from '@nest/components';

function SettingsPanel() {
  return (
    <div className="border rounded-lg divide-y">
      <div className="flex items-center justify-between p-4">
        <div>
          <FormLabel>Notifications</FormLabel>
          <p className="text-xs text-nest-muted">Receive push notifications</p>
        </div>
        <Switch defaultChecked />
      </div>
      <div className="flex items-center justify-between p-4">
        <div>
          <FormLabel>Email Updates</FormLabel>
          <p className="text-xs text-nest-muted">Weekly digest</p>
        </div>
        <Switch />
      </div>
      <div className="flex items-center justify-between p-4">
        <div>
          <FormLabel>Dark Mode</FormLabel>
          <p className="text-xs text-nest-muted">Use dark theme</p>
        </div>
        <Switch defaultChecked />
      </div>
    </div>
  );
}
```

### Disabled State

```tsx
import { Switch } from '@nest/components';

<Switch disabled />
<Switch disabled defaultChecked />
```

## Accessibility

- Switch uses `role="switch"` to indicate its purpose to screen readers
- The underlying input is `type="checkbox"` for native form behavior
- Always associate labels using `htmlFor` and `id`:
  ```tsx
  <Switch id="wifi" />
  <label htmlFor="wifi">Wi-Fi</label>
  ```
- For settings panels, provide additional context with descriptive text:
  ```tsx
  <div className="flex items-center justify-between">
    <div>
      <FormLabel>Notifications</FormLabel>
      <p className="text-xs text-nest-muted">Receive push notifications</p>
    </div>
    <Switch aria-describedby="notifications-desc" />
  </div>
  ```
- Keyboard users can toggle with Space, and tab between switches.

## Differences from Checkbox

| Aspect       | Switch                    | Checkbox              |
|--------------|---------------------------|-----------------------|
| Use case     | Immediate settings toggle | Form selection        |
| Visual       | Sliding toggle            | Checkmark in box      |
| Semantics    | `role="switch"`           | `type="checkbox"`     |
| Labeling     | Often has adjacent label  | Often inline label    |
