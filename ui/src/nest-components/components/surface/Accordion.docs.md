# Accordion, AccordionItem, AccordionSummary, AccordionDetails

A collection of collapsible panels that allow users to show/hide sections of content.

## When to Use

Use Accordion when:
- You have too much content to display at once
- Users need to reference multiple sections while working
- Organizing FAQs, settings, or step-by-step instructions
- You want to reduce visual clutter while keeping content accessible

## Props

### Accordion

| Prop            | Type                     | Default      | Description                                      |
|-----------------|--------------------------|--------------|--------------------------------------------------|
| `expanded`      | `string \| string[]`     | -            | Expanded panel value(s) (controlled)             |
| `defaultExpanded`| `string \| string[]`    | -            | Default expanded panel(s) (uncontrolled)         |
| `onChange`      | `(value) => void`        | -            | Callback when expanded state changes             |
| `disabled`      | `boolean`                | `false`      | If true, all panels are disabled                 |
| `exclusive`     | `boolean`                | `false`      | If true, only one panel can be expanded          |
| `className`     | `string`                 | -            | Additional CSS classes                           |

### AccordionItem

| Prop       | Type          | Default | Description                       |
|------------|---------------|---------|-----------------------------------|
| `value`    | `string`      | -       | The unique value of the item      |
| `summary`  | `ReactNode`   | -       | The header/summary content        |
| `disabled` | `boolean`     | `false` | If true, the item is disabled     |
| `className`| `string`      | -       | Additional CSS classes            |

### AccordionSummary

| Prop       | Type          | Default | Description                       |
|------------|---------------|---------|-----------------------------------|
| `expanded` | `boolean`     | -       | Whether the summary is expanded   |
| `disabled` | `boolean`     | `false` | If true, the summary is disabled  |
| `onClick`  | `() => void`  | -       | Click handler                     |
| `className`| `string`      | -       | Additional CSS classes            |

### AccordionDetails

| Prop       | Type          | Default | Description                       |
|------------|---------------|---------|-----------------------------------|
| `expanded` | `boolean`     | -       | Whether the details are expanded  |
| `className`| `string`      | -       | Additional CSS classes            |

## Examples

### Basic Usage

```tsx
import { Accordion, AccordionItem } from '@nest/components';

<Accordion defaultExpanded="panel1">
  <AccordionItem value="panel1" summary="Panel 1">
    <p>Content for panel 1</p>
  </AccordionItem>
  <AccordionItem value="panel2" summary="Panel 2">
    <p>Content for panel 2</p>
  </AccordionItem>
</Accordion>
```

### Exclusive (Single Expanded)

```tsx
import { useState } from 'react';
import { Accordion, AccordionItem } from '@nest/components';

const [expanded, setExpanded] = useState('panel1');

<Accordion expanded={expanded} onChange={(v) => setExpanded(v)} exclusive>
  <AccordionItem value="panel1" summary="Panel 1">
    <p>Only one panel can be open at a time</p>
  </AccordionItem>
  <AccordionItem value="panel2" summary="Panel 2">
    <p>Opening this closes the other</p>
  </AccordionItem>
</Accordion>
```

### Multiple Expanded Panels

```tsx
import { useState } from 'react';
import { Accordion, AccordionItem } from '@nest/components';

const [expanded, setExpanded] = useState<string[]>(['panel1']);

<Accordion expanded={expanded} onChange={(v) => setExpanded(v as string[])} exclusive={false}>
  <AccordionItem value="panel1" summary="Panel 1">
    <p>Multiple panels can be open</p>
  </AccordionItem>
  <AccordionItem value="panel2" summary="Panel 2">
    <p>at the same time</p>
  </AccordionItem>
</Accordion>
```

### FAQ Section

```tsx
import { Accordion, AccordionItem } from '@nest/components';

<Accordion defaultExpanded="q1">
  <AccordionItem value="q1" summary="What is your return policy?">
    <p>We offer a 30-day return policy for all unused items.</p>
  </AccordionItem>
  <AccordionItem value="q2" summary="How long does shipping take?">
    <p>Standard shipping takes 5-7 business days. Express shipping is 2-3 days.</p>
  </AccordionItem>
  <AccordionItem value="q3" summary="Do you ship internationally?">
    <p>Yes, we ship to over 50 countries worldwide.</p>
  </AccordionItem>
</Accordion>
```

### With Rich Summary Content

```tsx
import { Accordion, AccordionItem } from '@nest/components';

<Accordion>
  <AccordionItem
    value="step1"
    summary={
      <div className="flex items-center gap-3">
        <span className="h-8 w-8 rounded-full bg-primary text-white flex items-center justify-center">1</span>
        <span>Step One: Planning</span>
      </div>
    }
  >
    <div className="pl-11">
      <p>Detailed planning content goes here...</p>
    </div>
  </AccordionItem>
</Accordion>
```

### Settings Panel

```tsx
import { Accordion, AccordionItem } from '@nest/components';

function SettingsPanel() {
  return (
    <Accordion defaultExpanded="account">
      <AccordionItem value="account" summary="Account">
        <div className="space-y-3">
          <label className="flex items-center justify-between">
            <span>Email Notifications</span>
            <input type="checkbox" defaultChecked />
          </label>
          <label className="flex items-center justify-between">
            <span>Two-Factor Auth</span>
            <input type="checkbox" />
          </label>
        </div>
      </AccordionItem>
      <AccordionItem value="privacy" summary="Privacy">
        <div className="space-y-3">
          <label className="flex items-center justify-between">
            <span>Profile Visibility</span>
            <select>
              <option>Public</option>
              <option>Friends</option>
              <option>Private</option>
            </select>
          </label>
        </div>
      </AccordionItem>
    </Accordion>
  );
}
```

### Disabled State

```tsx
import { Accordion, AccordionItem } from '@nest/components';

// Entire accordion disabled
<Accordion disabled>
  <AccordionItem value="panel1" summary="Disabled Panel">
    <p>Cannot be expanded</p>
  </AccordionItem>
</Accordion>

// Individual panel disabled
<Accordion>
  <AccordionItem value="panel1" summary="Enabled">
    <p>Can be toggled</p>
  </AccordionItem>
  <AccordionItem value="panel2" summary="Disabled" disabled>
    <p>Cannot be toggled</p>
  </AccordionItem>
</Accordion>
```

## Accessibility

- Accordion uses proper ARIA attributes (`aria-expanded`) for screen readers
- Each summary button is keyboard accessible (Tab to focus, Enter/Space to toggle)
- Details have `role="region"` for semantic navigation
- Use meaningful summary text that describes the hidden content
- For complex summaries, ensure interactive elements within are properly labeled

## Keyboard Navigation

| Key       | Action                          |
|-----------|---------------------------------|
| Tab       | Move focus to next accordion    |
| Enter     | Expand/collapse focused panel   |
| Space     | Expand/collapse focused panel   |
| Arrow Down| Move to next panel header       |
| Arrow Up  | Move to previous panel header   |
| Home      | Move to first panel header      |
| End       | Move to last panel header       |
