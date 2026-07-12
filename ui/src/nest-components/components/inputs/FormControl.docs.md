# FormControl, FormLabel, FormHelperText

Layout components for building accessible form fields with consistent styling.

## When to Use

Use these components together to create properly structured form fields:
- **FormControl**: Wraps a complete form field (label + input + helper text)
- **FormLabel**: The label that identifies what the field is for
- **FormHelperText**: Additional guidance or error messages below the field

## Components

### FormControl

Wraps form inputs and their associated labels/helper text, providing consistent spacing and state handling.

### FormLabel

A label component that supports required indicators, error states, and proper `htmlFor` association.

### FormHelperText

Helper or error text displayed below form fields.

## Props

### FormControl

| Prop        | Type      | Default | Description                          |
|-------------|-----------|---------|--------------------------------------|
| `error`     | `boolean` | `false` | If true, indicates an error state    |
| `disabled`  | `boolean` | `false` | If true, the field is disabled       |
| `required`  | `boolean` | `false` | If true, the field is required       |
| `focused`   | `boolean` | `false` | If true, displays focused styling    |
| `fullWidth` | `boolean` | `false` | If true, takes full width            |
| `component` | `ElementType` | `'div'` | The component to render as     |
| `className` | `string`  | -       | Additional CSS classes               |

### FormLabel

| Prop       | Type      | Default | Description                          |
|------------|-----------|---------|--------------------------------------|
| `error`    | `boolean` | `false` | If true, displays error styling      |
| `disabled` | `boolean` | `false` | If true, displays disabled styling   |
| `focused`  | `boolean` | `false` | If true, displays focused styling    |
| `required` | `boolean` | `false` | If true, shows required asterisk     |
| `htmlFor`  | `string`  | -       | Associates label with input by ID    |
| `className`| `string`  | -       | Additional CSS classes               |

### FormHelperText

| Prop            | Type      | Default | Description                          |
|-----------------|-----------|---------|--------------------------------------|
| `error`         | `boolean` | `false` | If true, displays error styling      |
| `disabled`      | `boolean` | `false` | If true, displays disabled styling   |
| `visuallyHidden`| `boolean` | `false` | If true, hides visually but keeps for screen readers |
| `className`     | `string`  | -       | Additional CSS classes               |

## Examples

### Basic Form Field

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

<FormControl>
  <FormLabel htmlFor="email">Email</FormLabel>
  <TextField id="email" type="email" placeholder="Enter email" />
  <FormHelperText>We'll never share your email</FormHelperText>
</FormControl>
```

### Required Field

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

<FormControl required>
  <FormLabel htmlFor="name" required>Name</FormLabel>
  <TextField id="name" placeholder="Enter your name" required />
  <FormHelperText>Enter your full legal name</FormHelperText>
</FormControl>
```

### Error State

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

<FormControl error>
  <FormLabel error htmlFor="email">Email</FormLabel>
  <TextField id="email" type="email" defaultValue="invalid" />
  <FormHelperText error>Please enter a valid email address</FormHelperText>
</FormControl>
```

### Disabled Field

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

<FormControl disabled>
  <FormLabel disabled htmlFor="disabled">Disabled</FormLabel>
  <TextField id="disabled" disabled placeholder="Cannot edit" />
  <FormHelperText disabled>This field is read-only</FormHelperText>
</FormControl>
```

### Visually Hidden Helper

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

<FormControl>
  <FormLabel htmlFor="search">Search</FormLabel>
  <TextField id="search" placeholder="Search..." />
  <FormHelperText visuallyHidden>
    Type to search. Results will appear below.
  </FormHelperText>
</FormControl>
```

### Complete Form

```tsx
import { FormControl, FormLabel, FormHelperText, TextField } from '@nest/components';

function ContactForm() {
  return (
    <div className="space-y-4">
      <FormControl>
        <FormLabel htmlFor="name">Name</FormLabel>
        <TextField id="name" placeholder="John Doe" />
      </FormControl>

      <FormControl>
        <FormLabel htmlFor="email">Email</FormLabel>
        <TextField id="email" type="email" placeholder="john@example.com" />
        <FormHelperText>We'll send a confirmation</FormHelperText>
      </FormControl>

      <FormControl>
        <FormLabel htmlFor="message">Message</FormLabel>
        <TextField id="message" multiline rows={4} placeholder="Your message" />
      </FormControl>
    </div>
  );
}
```

## Accessibility

- Always use `htmlFor` on FormLabel matching the input's `id` for proper association
- Use `required` on both FormControl/FormLabel and the input element itself
- Error states should be communicated to screen readers via `aria-invalid` on the input
- Visually hidden helper text (`visuallyHidden`) is useful for screen reader-only instructions
- FormHelperText with `error` should be associated with the input via `aria-describedby`

```tsx
<FormControl error>
  <FormLabel error htmlFor="email">Email</FormLabel>
  <TextField 
    id="email" 
    type="email" 
    aria-invalid="true"
    aria-describedby="email-error"
  />
  <FormHelperText error id="email-error">
    Please enter a valid email
  </FormHelperText>
</FormControl>
```
