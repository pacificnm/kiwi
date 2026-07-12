# Popover

A popup that displays content relative to an anchor element.

## When to Use

Use Popover when:
- Showing contextual actions or information
- Creating dropdown menus (use Menu for menu-specific behavior)
- Displaying quick forms or inputs
- Showing user profile cards or previews
- Building custom tooltips with interactive content

## Props

| Prop                | Type                                      | Default     | Description                              |
|---------------------|-------------------------------------------|-------------|------------------------------------------|
| `children`          | `ReactNode`                               | -           | The content of the popover               |
| `trigger`           | `ReactNode`                               | -           | The element that triggers the popover    |
| `open`              | `boolean`                                 | -           | Controlled open state                    |
| `onOpenChange`      | `(open: boolean) => void`                 | -           | Callback when open state changes         |
| `placement`         | `PopoverPlacement`                        | `'bottom'`  | Position relative to anchor              |
| `closeOnOutsideClick`| `boolean`                                | `true`      | Close when clicking outside              |
| `closeOnEscape`     | `boolean`                                 | `true`      | Close when pressing Escape               |
| `className`         | `string`                                  | -           | Additional CSS classes                   |

## Examples

### Basic Usage

```tsx
import { Popover, Button } from '@nest/components';
import { useState } from 'react';

function BasicPopover() {
  const [open, setOpen] = useState(false);

  return (
    <Popover
      trigger={<Button>Open popover</Button>}
      open={open}
      onOpenChange={setOpen}
    >
      <p>Popover content goes here</p>
    </Popover>
  );
}
```

### Uncontrolled

```tsx
import { Popover, Button } from '@nest/components';

<Popover trigger={<Button>Click me</Button>}>
  <p>This popover toggles automatically</p>
</Popover>
```

### Placements

```tsx
import { Popover, Button } from '@nest/components';

<Popover trigger={<Button>Top</Button>} placement="top">
  <p>Above anchor</p>
</Popover>

<Popover trigger={<Button>Bottom</Button>} placement="bottom">
  <p>Below anchor</p>
</Popover>

<Popover trigger={<Button>Left</Button>} placement="left">
  <p>Left of anchor</p>
</Popover>

<Popover trigger={<Button>Right</Button>} placement="right">
  <p>Right of anchor</p>
</Popover>
```

### Controlled

```tsx
import { useState } from 'react';
import { Popover, Button } from '@nest/components';

function ControlledPopover() {
  const [open, setOpen] = useState(false);

  return (
    <>
      <Popover
        trigger={<Button>Open</Button>}
        open={open}
        onOpenChange={setOpen}
      >
        <p>Controlled content</p>
      </Popover>
      <Button onClick={() => setOpen(!open)}>
        {open ? 'Close' : 'Open'} programmatically
      </Button>
    </>
  );
}
```

### Without Outside Click Close

```tsx
import { Popover, Button } from '@nest/components';

<Popover
  trigger={<Button>Open</Button>}
  closeOnOutsideClick={false}
>
  <p>Click trigger again or press Escape to close</p>
</Popover>
```

### User Profile Card

```tsx
import { Popover, Button, Avatar } from '@nest/components';

<Popover
  trigger={
    <div className="flex items-center gap-2 cursor-pointer">
      <Avatar src="/avatar.jpg" alt="User" />
      <span>John Doe</span>
    </div>
  }
>
  <div className="space-y-3 min-w-[250px]">
    <div className="flex items-center gap-3">
      <Avatar src="/avatar.jpg" alt="User" />
      <div>
        <p className="font-medium">John Doe</p>
        <p className="text-xs text-muted">john@example.com</p>
      </div>
    </div>
    <div className="border-t pt-2">
      <button className="block w-full text-left px-2 py-1 hover:bg-muted rounded">
        View Profile
      </button>
      <button className="block w-full text-left px-2 py-1 hover:bg-muted rounded">
        Settings
      </button>
    </div>
  </div>
</Popover>
```

### Quick Form

```tsx
import { Popover, Button, TextField } from '@nest/components';

<Popover trigger={<Button>Quick Add</Button>}>
  <form className="space-y-3" onSubmit={handleSubmit}>
    <TextField name="title" placeholder="Title" />
    <TextField name="description" placeholder="Description" multiline />
    <Button type="submit" size="small">Add</Button>
  </form>
</Popover>
```

## Accessibility

- Popover uses `role="dialog"` for screen readers
- Content is focusable and receives focus when opened
- Closes on Escape key by default
- Closes on outside click by default
- Anchor element should be keyboard accessible

## Keyboard Support

| Key    | Action              |
|--------|---------------------|
| Tab    | Navigate content    |
| Escape | Close popover       |
| Click outside | Close popover |

## Tips

- Use Popover for interactive content (tooltips are for static info)
- Use Menu component for menu-specific behavior
- Keep content concise - consider scrolling for long content
- Use `closeOnOutsideClick={false}` for forms that shouldn't accidentally close
- Consider using `placement` to avoid edge cases near viewport boundaries
