# IconButton

A compact, square button for icon-only actions.

## When to Use

- Toolbar and title-bar actions where a label would be redundant (close, edit, delete).
- Inside dense UI (list rows, card headers) where a full [Button](./Button.md) is too large.
- Always pair with `aria-label` since there is no visible text.

## Variants

| Prop    | Values                                                                        | Default    | Effect                    |
| ------- | ----------------------------------------------------------------------------- | ---------- | ------------------------- |
| `size`  | `small` \| `medium` \| `large`                                                | `medium`   | `size-8` / `size-10` / `size-12` |
| `color` | `default` \| `primary` \| `secondary` \| `accent` \| `error` \| `success` \| `warning` \| `info` | `default`  | Icon color + hover tint   |

## Props

| Prop         | Type                | Default    | Description                                    |
| ------------ | ------------------- | ---------- | ---------------------------------------------- |
| `children`   | `ReactNode`         | —          | The icon element (e.g. a `lucide-react` icon). |
| `size`       | `IconButtonSize`    | `'medium'` | Button size.                                   |
| `color`      | `IconButtonColor`   | `'default'`| Color scheme.                                  |
| `fullWidth`  | `boolean`           | `false`    | Stretch to container width.                    |
| `aria-label` | `string`            | —          | Accessible name (required for icon-only).      |

Also accepts all native `<button>` attributes (`onClick`, `disabled`, `type`, …).

## Examples

```tsx
import { IconButton } from '@nest/components';
import { Trash2 } from 'lucide-react';

<IconButton color="error" aria-label="Delete" onClick={remove}>
  <Trash2 className="size-5" />
</IconButton>
```

## Accessibility

- Renders a native `<button>` — keyboard and focus behavior are built in.
- Standard focus ring: `focus:ring-2 focus:ring-nest-primary/50`.
- Provide `aria-label` (or otherwise-labelled content); an icon alone has no accessible name.
- `disabled` applies `disabled:opacity-50 disabled:cursor-not-allowed`.
