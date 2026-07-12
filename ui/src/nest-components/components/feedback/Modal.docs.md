# Modal

A dialog overlay component that displays content in a layer that traps focus and blocks interaction with the rest of the page.

## When to Use

- Confirming destructive actions (delete, discard changes)
- Collecting focused user input (forms, selections)
- Displaying important information requiring user acknowledgment
- Multi-step workflows that need to maintain context

## Anatomy

The Modal consists of:
- **Backdrop**: Semi-transparent overlay covering the entire viewport
- **Dialog surface**: The content container, centered on screen
- **Focus trap**: Keeps keyboard navigation within the modal

## Usage

### Basic Modal

```tsx
import { Modal } from '@nest/components';

function Example() {
  const [open, setOpen] = useState(false);
  
  return (
    <>
      <Button onClick={() => setOpen(true)}>Open Modal</Button>
      <Modal open={open} onClose={() => setOpen(false)}>
        <div className="p-6">
          <h2>Modal Title</h2>
          <p>Modal content goes here...</p>
        </div>
      </Modal>
    </>
  );
}
```

### Modal with Actions

```tsx
<Modal open={open} onClose={handleClose}>
  <div className="p-6">
    <Typography variant="h6" className="mb-2">
      Confirm Delete
    </Typography>
    <Typography variant="body2" className="text-nest-muted mb-4">
      Are you sure you want to delete this item?
    </Typography>
    <Stack direction="row" gap={2} className="justify-end">
      <Button variant="outlined" onClick={handleCancel}>
        Cancel
      </Button>
      <Button variant="contained" color="error" onClick={handleDelete}>
        Delete
      </Button>
    </Stack>
  </div>
</Modal>
```

### Prevent Outside Close

```tsx
<Modal 
  open={open} 
  onClose={handleClose}
  closeOnOutsideClick={false}
  closeOnEscape={false}
>
  <div className="p-6">
    <Typography variant="h6">
      Required Action
    </Typography>
    <Typography variant="body2">
      You must complete this form before closing.
    </Typography>
    {/* Form fields */}
  </div>
</Modal>
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `open` | `boolean` | `false` | Controls modal visibility |
| `onClose` | `() => void` | - | Callback when modal requests to close |
| `closeOnOutsideClick` | `boolean` | `true` | Clicking backdrop closes modal |
| `closeOnEscape` | `boolean` | `true` | Pressing Escape closes modal |
| `trapFocus` | `boolean` | `true` | Traps keyboard focus inside modal |
| `className` | `string` | - | Additional classes for dialog surface |
| `backdropClassName` | `string` | - | Additional classes for backdrop |

## Accessibility

- Modal uses `role="dialog"` and `aria-modal="true"`
- Focus is automatically moved to the first focusable element
- Tab key is trapped within the modal
- Escape key closes the modal (unless disabled)
- Backdrop click closes the modal (unless disabled)

## Best Practices

- **Keep it focused**: Modals should have a single clear purpose
- **Provide clear exit**: Always give users a way to close (button, Escape, or outside click)
- **Size appropriately**: Content should fit without excessive scrolling
- **Use for important interactions**: Don't overuse modals for routine content
- **Consider alternatives**: For non-blocking content, consider Popover or inline expansion

## Related Components

- **Dialog**: Pre-styled modal with header/content/footer structure
- **Popover**: Lighter overlay for contextual content
- **Drawer**: Slide-in panel alternative for complex content
