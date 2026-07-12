# Card

A container component for grouping related content with optional header, media, content, and actions sections.

## When to Use

Use `Card` for:
- Article or blog post previews
- Product cards
- User profile cards
- Dashboard widgets
- Media galleries
- Form containers
- Any grouped content that needs visual separation

## Subcomponents

| Component | Description |
|-----------|-------------|
| `Card` | Main container (extends Paper props) |
| `CardHeader` | Header with optional avatar, title, subheader, action |
| `CardContent` | Main content area with padding |
| `CardActions` | Action buttons area |
| `CardMedia` | Image or custom media section |

## Props

### Card

Extends all `Paper` props (`elevation`, `variant`, `square`, etc.).

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `elevation` | `0 \| 1 \| 2 \| 3 \| 4` | `1` | Shadow depth |
| `variant` | `'elevation' \| 'outlined'` | `'elevation'` | Visual style |
| `className` | `string` | - | Additional CSS classes |

### CardHeader

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `avatar` | `ReactNode` | - | Avatar element before title |
| `action` | `ReactNode` | - | Action element after title |
| `title` | `ReactNode` | - | Title content |
| `subheader` | `ReactNode` | - | Subtitle content |
| `className` | `string` | - | Additional CSS classes |

### CardContent

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `children` | `ReactNode` | - | Content to display |
| `className` | `string` | - | Additional CSS classes |

### CardActions

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `children` | `ReactNode` | - | Action buttons |
| `disableSpacing` | `boolean` | `false` | Remove gap between actions |
| `className` | `string` | - | Additional CSS classes |

### CardMedia

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `component` | `ElementType` | `'div'` | HTML element to render as |
| `image` | `string` | - | Image URL (renders as img) |
| `alt` | `string` | `''` | Alt text for images |
| `title` | `string` | - | Title attribute |
| `height` | `string` | `'140px'` | Media height |
| `className` | `string` | - | Additional CSS classes |

## Examples

### Basic Card

```tsx
import { Card, CardContent } from '@nest/components';

<Card>
  <CardContent>
    <p>Simple card content</p>
  </CardContent>
</Card>
```

### Card with Header and Actions

```tsx
<Card>
  <CardHeader
    title="Card Title"
    subheader="Card subtitle"
  />
  <CardContent>
    <p>Card content goes here</p>
  </CardContent>
  <CardActions>
    <Button>Learn More</Button>
    <Button variant="outlined">Cancel</Button>
  </CardActions>
</Card>
```

### Card with Avatar

```tsx
<Card>
  <CardHeader
    avatar={
      <Avatar src="/user.jpg" alt="User" />
    }
    title="John Doe"
    subheader="Software Engineer"
    action={
      <IconButton aria-label="settings">⋮</IconButton>
    }
  />
  <CardContent>
    <p>User bio or description</p>
  </CardContent>
</Card>
```

### Card with Media

```tsx
<Card>
  <CardMedia
    image="https://example.com/image.jpg"
    alt="Article thumbnail"
    height="160px"
  />
  <CardHeader title="Article Title" />
  <CardContent>
    <p>Article excerpt or description</p>
  </CardContent>
  <CardActions>
    <Button size="small">Read More</Button>
  </CardActions>
</Card>
```

### Horizontal Card

```tsx
<Card>
  <div className="flex">
    <CardMedia
      image="/thumbnail.jpg"
      className="!h-auto !w-40"
    />
    <div className="flex flex-1 flex-col">
      <CardHeader title="Horizontal Card" />
      <CardContent>
        <p>Content next to media</p>
      </CardContent>
      <CardActions>
        <Button>Action</Button>
      </CardActions>
    </div>
  </div>
</Card>
```

### Interactive Card

```tsx
<Card
  component="button"
  onClick={handleCardClick}
  className="cursor-pointer hover:shadow-md"
>
  <CardHeader title="Clickable Card" />
  <CardContent>
    <p>Click anywhere on this card</p>
  </CardContent>
</Card>
```

### Card Grid

```tsx
<div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
  {items.map(item => (
    <Card key={item.id} elevation={2}>
      <CardMedia image={item.image} height="120px" />
      <CardHeader title={item.title} />
      <CardContent>
        <p>{item.description}</p>
      </CardContent>
      <CardActions>
        <Button size="small">View</Button>
      </CardActions>
    </Card>
  ))}
</div>
```

### Outlined Card

```tsx
<Card variant="outlined">
  <CardHeader title="Outlined Card" />
  <CardContent>
    <p>Subtle border-based card</p>
  </CardContent>
</Card>
```

## Layout Patterns

### Media on Top (Default)

```tsx
<Card>
  <CardMedia image="/img.jpg" />
  <CardHeader title="Title" />
  <CardContent>Content</CardContent>
  <CardActions>Actions</CardActions>
</Card>
```

### Horizontal Media

```tsx
<Card>
  <div className="flex">
    <CardMedia image="/img.jpg" className="!h-auto !w-48" />
    <div className="flex flex-1 flex-col">
      <CardHeader title="Title" />
      <CardContent>Content</CardContent>
    </div>
  </div>
</Card>
```

### Header Only

```tsx
<Card>
  <CardHeader title="Simple Header" />
</Card>
```

### Content Only

```tsx
<Card>
  <CardContent>
    <p>Just content, no header or actions</p>
  </CardContent>
</Card>
```

## Accessibility

- Use semantic `component` values (`article`, `section`) for proper document structure
- When using `component="button"`, ensure the entire card is the clickable area
- Add `aria-label` to CardHeader actions for screen readers
- Provide meaningful `alt` text for CardMedia images
- Ensure sufficient color contrast for card content
