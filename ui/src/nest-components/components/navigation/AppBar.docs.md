# AppBar

Top application bar for desktop-window chrome (title bar, menu row, toolbar). Exported
together with `Toolbar`.

## When to Use

- The persistent header of a desktop app window: title, menu bar, and toolbar actions.
- Pair with `Toolbar` for the flex row of title/actions, or with `MenuBar` (see
  [Menu](./Menu.md)) for a File/Edit-style dropdown row.

## Variants

### AppBar

| Prop        | Values                               | Default     | Effect                                    |
| ----------- | ------------------------------------ | ----------- | ----------------------------------------- |
| `position`  | `static` \| `fixed` \| `sticky`      | `static`    | CSS positioning of the bar                |
| `color`     | `surface` \| `primary` \| `transparent` | `surface` | Background + text color                   |
| `elevation` | `boolean`                            | `true`      | Adds a bottom border + shadow             |

### Toolbar

| Prop      | Values               | Default     | Effect                          |
| --------- | -------------------- | ----------- | ------------------------------- |
| `variant` | `regular` \| `dense` | `regular`   | Row height: 48px vs 32px chrome |

## Props

`AppBar` and `Toolbar` both extend native HTML attributes (`className`, `children`, …).
See the tables above for component-specific props.

## Examples

```tsx
import { AppBar, Toolbar, IconButton } from '@nest/components';
import { Settings } from 'lucide-react';

<AppBar position="sticky" color="surface">
  <Toolbar>
    <span className="font-semibold text-nest-foreground">My App</span>
    <span className="flex-1" />
    <IconButton aria-label="Settings"><Settings className="size-5" /></IconButton>
  </Toolbar>
</AppBar>
```

## Accessibility

- `AppBar` renders a semantic `<header>` element.
- Compose interactive children (`IconButton`, `MenuBar`) that carry their own roles.
- Use `position="sticky"`/`"fixed"` for scroll-persistent chrome; ensure content below is offset.
