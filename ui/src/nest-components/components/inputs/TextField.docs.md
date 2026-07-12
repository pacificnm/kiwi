# TextField

Text fields allow users to enter and edit text input.

## When to Use

- Form data entry (name, email, password)
- Search inputs
- Multi-line text areas
- Any text-based user input

## Variants

| Variant | When to Use |
|---------|-------------|
| `outlined` | Default, works in most contexts |
| `filled` | Dense forms, data-heavy interfaces |
| `standard` | Minimal UI, simple forms |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `label` | `string` | - | Input label |
| `value` | `string` | - | Controlled value |
| `onChange` | `(e) => void` | - | Change handler |
| `placeholder` | `string` | - | Placeholder text |
| `variant` | `'outlined' \| 'filled' \| 'standard'` | `'outlined'` | Visual style |
| `size` | `'small' \| 'medium'` | `'medium'` | Input size |
| `error` | `string` | - | Error message (sets invalid state) |
| `helperText` | `ReactNode` | - | Helper text below input |
| `startAdornment` | `ReactNode` | - | Element before input |
| `endAdornment` | `ReactNode` | - | Element after input |
| `multiline` | `boolean` | `false` | Use textarea |
| `rows` | `number` | `3` | Rows for multiline |
| `fullWidth` | `boolean` | `false` | Full width input |
| `disabled` | `boolean` | `false` | Disable input |
| `type` | `string` | `'text'` | HTML input type |

## Examples

### Basic Usage

```tsx
import { TextField } from '@nest/components';

<TextField label="Email" value={email} onChange={(e) => setEmail(e.target.value)} />
```

### With Validation

```tsx
<TextField
  label="Username"
  value={username}
  onChange={(e) => setUsername(e.target.value)}
  error={usernameError}
  helperText="Choose a unique username"
/>
```

### With Icons

```tsx
import { Search } from 'lucide-react';

<TextField
  label="Search"
  startAdornment={<Search className="size-4" />}
  placeholder="Search..."
/>
```

### Password Toggle

```tsx
const [showPassword, setShowPassword] = useState(false);

<TextField
  label="Password"
  type={showPassword ? 'text' : 'password'}
  endAdornment={
    <button onClick={() => setShowPassword(!showPassword)}>
      {showPassword ? <EyeOff /> : <Eye />}
    </button>
  }
/>
```

### Multiline

```tsx
<TextField
  label="Description"
  multiline
  rows={4}
  placeholder="Enter detailed description..."
/>
```

## Accessibility

- Labels are associated with inputs via `htmlFor`
- Error messages use `role="alert"` for screen readers
- Invalid state communicated via `aria-invalid`
