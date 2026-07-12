# Avatar

A component that represents a person or entity with an image or fallback content.

## When to Use

Use `Avatar` for:
- User profile pictures
- Contact avatars in messages/comments
- Team member displays
- Account switches
- Author attribution
- Fallback initials when no image available

## Variants

| Variant | Description |
|---------|-------------|
| `circular` (default) | Fully rounded (circle) |
| `rounded` | Rounded corners (rounded-nest-md) |
| `square` | No border radius |

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `src` | `string` | - | Image URL |
| `alt` | `string` | `''` | Alt text for image |
| `variant` | `'circular' \| 'rounded' \| 'square'` | `'circular'` | Shape variant |
| `size` | `'small' \| 'medium' \| 'large'` | `'medium'` | Avatar size |
| `children` | `ReactNode` | - | Fallback content (initials, icon) |
| `imgProps` | `ImgHTMLAttributes` | - | Props passed to img element |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Avatar with Image

```tsx
import { Avatar } from '@nest/components';

<Avatar src="/user.jpg" alt="John Doe" />
```

### Avatar with Initials

```tsx
<Avatar>JD</Avatar>
```

### Avatar with Icon

```tsx
import { User } from 'lucide-react';

<Avatar>
  <User className="h-5 w-5" />
</Avatar>
```

### Size Variants

```tsx
<Avatar size="small" src="/user.jpg" alt="Small" />
<Avatar size="medium" src="/user.jpg" alt="Medium" />
<Avatar size="large" src="/user.jpg" alt="Large" />
```

### Shape Variants

```tsx
<Avatar variant="circular" src="/user.jpg" alt="Circular" />
<Avatar variant="rounded" src="/user.jpg" alt="Rounded" />
<Avatar variant="square" src="/user.jpg" alt="Square" />
```

### Fallback on Error

```tsx
<Avatar src="/invalid.jpg" alt="User">
  Fallback
</Avatar>
```

### Avatar Group

```tsx
<div className="flex -space-x-2">
  <Avatar src="/user1.jpg" className="ring-2 ring-white" />
  <Avatar src="/user2.jpg" className="ring-2 ring-white" />
  <Avatar src="/user3.jpg" className="ring-2 ring-white" />
  <Avatar>+3</Avatar>
</div>
```

### Avatar with Status

```tsx
<div className="relative">
  <Avatar src="/user.jpg" alt="User" />
  <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-green-500 ring-2 ring-white" />
</div>
```

### Clickable Avatar

```tsx
// As button
<Avatar
  component="button"
  src="/user.jpg"
  onClick={handleClick}
  className="cursor-pointer hover:opacity-80"
/>

// As link
<Avatar
  component="a"
  href="/profile"
  src="/user.jpg"
  className="cursor-pointer hover:opacity-80"
/>
```

### In Comment

```tsx
<div className="flex gap-3">
  <Avatar src="/user.jpg" alt="User" />
  <div>
    <div className="flex items-center gap-2">
      <span className="font-medium">John Doe</span>
      <span className="text-xs text-muted">2 hours ago</span>
    </div>
    <p className="text-sm text-muted">Comment text</p>
  </div>
</div>
```

### In User Card

```tsx
<div className="flex items-center gap-3 border p-3 rounded-md">
  <Avatar src="/user.jpg" alt="User" />
  <div>
    <p className="font-medium">Jane Smith</p>
    <p className="text-xs text-muted">jane@example.com</p>
  </div>
</div>
```

## Size Guide

| Size | Dimensions | Text Size | Use Case |
|------|------------|-----------|----------|
| `small` | 32px (h-8 w-8) | xs | Compact lists, dense UIs |
| `medium` | 40px (h-10 w-10) | sm | Default, comments, nav |
| `large` | 48px (h-12 w-12) | base | Profile pages, hero sections |

## Accessibility

- Always provide `alt` text when using `src`
- Use meaningful initials or icons as fallback
- When using `component="button"` or `component="a"`, ensure proper focus states
- Status indicators should have `aria-label` or visually hidden text

## Styling

Avatar uses these base styles:
- `flex items-center justify-center` - Center content
- `overflow-hidden` - Clip image to shape
- `bg-nest-muted text-nest-foreground` - Fallback background
- `font-medium` - Emphasis for initials

Custom styling via `className`:

```tsx
<Avatar className="bg-nest-primary text-white">P</Avatar>
<Avatar className="border-2 border-nest-primary">B</Avatar>
<Avatar className="bg-gradient-to-br from-primary to-accent">G</Avatar>
```
