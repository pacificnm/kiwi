# List

A collection of components for displaying lists with various content types.

## When to Use

Use `List` for:
- Navigation menus
- Contact lists
- Settings panels
- Message inboxes
- Any vertical list of items

## Components

| Component | Description |
|-----------|-------------|
| `List` | Container for list items |
| `ListItem` | Basic list item |
| `ListItemButton` | Interactive list item with button behavior |
| `ListItemText` | Text content (primary/secondary) |
| `ListItemIcon` | Icon container |
| `ListItemAvatar` | Avatar container |

## Examples

### Basic List

```tsx
import { List, ListItem } from '@nest/components';

<List>
  <ListItem>Item 1</ListItem>
  <ListItem>Item 2</ListItem>
  <ListItem>Item 3</ListItem>
</List>
```

### Dense List

```tsx
<List dense>
  <ListItem>Dense item 1</ListItem>
  <ListItem>Dense item 2</ListItem>
</List>
```

### List with Icons

```tsx
import { List, ListItem, ListItemIcon, ListItemText } from '@nest/components';
import { Inbox, Star } from 'lucide-react';

<List>
  <ListItem>
    <ListItemIcon>
      <Inbox />
    </ListItemIcon>
    <ListItemText primary="Inbox" secondary="12 messages" />
  </ListItem>
  <ListItem>
    <ListItemIcon>
      <Star />
    </ListItemIcon>
    <ListItemText primary="Starred" />
  </ListItem>
</List>
```

### List with Avatars

```tsx
import { List, ListItem, ListItemAvatar, ListItemText } from '@nest/components';
import { Avatar } from '@nest/components';

<List>
  <ListItem>
    <ListItemAvatar>
      <Avatar src="/user.jpg" alt="User" />
    </ListItemAvatar>
    <ListItemText primary="John Doe" secondary="john@example.com" />
  </ListItem>
</List>
```

### Interactive List

```tsx
const [selectedIndex, setSelectedIndex] = useState(0);

<List>
  <ListItemButton
    selected={selectedIndex === 0}
    onClick={() => setSelectedIndex(0)}
  >
    <ListItemIcon>
      <Inbox />
    </ListItemIcon>
    <ListItemText primary="Inbox" />
  </ListItemButton>
  <ListItemButton
    selected={selectedIndex === 1}
    onClick={() => setSelectedIndex(1)}
  >
    <ListItemIcon>
      <Star />
    </ListItemIcon>
    <ListItemText primary="Starred" />
  </ListItemButton>
</List>
```

### Contact List

```tsx
<List>
  <ListItem>
    <ListItemAvatar>
      <Avatar>JD</Avatar>
    </ListItemAvatar>
    <ListItemText
      primary="John Doe"
      secondary="+1 (555) 123-4567"
    />
  </ListItem>
  <ListItem>
    <ListItemAvatar>
      <Avatar>JS</Avatar>
    </ListItemAvatar>
    <ListItemText
      primary="Jane Smith"
      secondary="jane@example.com"
    />
  </ListItem>
</List>
```

### Settings List

```tsx
<List>
  <ListItemButton>
    <ListItemIcon>
      <UserIcon />
    </ListItemIcon>
    <ListItemText
      primary="Account"
      secondary="Manage your account"
    />
  </ListItemButton>
  <ListItemButton>
    <ListItemIcon>
      <MailIcon />
    </ListItemIcon>
    <ListItemText
      primary="Notifications"
      secondary="Email preferences"
    />
  </ListItemButton>
</List>
```

### Disabled List Items

```tsx
<List>
  <ListItemButton>
    <ListItemText primary="Enabled" />
  </ListItemButton>
  <ListItemButton disabled>
    <ListItemText primary="Disabled" />
  </ListItemButton>
</List>
```

## Props

### List

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'ul'` | HTML element to render as |
| `dense` | `boolean` | `false` | Compact vertical padding |
| `className` | `string` | - | Additional CSS classes |

### ListItem

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'li'` | HTML element to render as |
| `className` | `string` | - | Additional CSS classes |

### ListItemButton

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'button'` | HTML element to render as |
| `selected` | `boolean` | `false` | Selected state |
| `disabled` | `boolean` | `false` | Disabled state |
| `onClick` | `() => void` | - | Click handler |
| `className` | `string` | - | Additional CSS classes |

### ListItemText

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `primary` | `ReactNode` | - | Primary content |
| `secondary` | `ReactNode` | - | Secondary content |
| `className` | `string` | - | Additional CSS classes |

### ListItemIcon

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `className` | `string` | - | Additional CSS classes |

### ListItemAvatar

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `className` | `string` | - | Additional CSS classes |

## Accessibility

- `List` has `role="list"`
- `ListItem` has `role="listitem"`
- `ListItemButton` is a native button element
- Long text is truncated with `truncate` class
- Use `aria-label` or `aria-labelledby` for lists without visible labels

## Styling

### List
- `flex flex-col` - Vertical layout
- `py-2` (normal) or `py-1` (dense) - Vertical padding

### ListItem
- `flex items-center gap-3 px-3` - Horizontal layout with spacing

### ListItemButton
- `w-full text-left` - Full width, left-aligned text
- `rounded-nest-sm` - Rounded corners
- `bg-nest-primary/10 text-nest-primary` - Selected state
- `hover:bg-nest-surface` - Hover state (when not selected)
- `focus:ring-2 focus:ring-nest-primary/50` - Focus ring

### ListItemText
- `flex flex-col flex-1 min-w-0` - Flex container that allows truncation
- `text-nest-foreground truncate` - Primary text
- `text-sm text-nest-muted truncate` - Secondary text

### ListItemIcon
- `shrink-0 text-nest-muted` - Non-shrinking, muted color

### ListItemAvatar
- `shrink-0` - Non-shrinking container
