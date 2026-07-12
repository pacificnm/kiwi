# Tabs, Tab, TabPanel

A container for organizing content into multiple sections that can be switched between.

## When to Use

Use Tabs when:
- You have related content that should be organized into separate views
- Users need to switch between views frequently
- You want to reduce scrolling by grouping content logically
- Building settings pages, dashboards, or data views

## Props

### Tabs

| Prop          | Type                     | Default       | Description                                      |
|---------------|--------------------------|---------------|--------------------------------------------------|
| `value`       | `string \| number`       | -             | Selected tab value (controlled)                  |
| `defaultValue`| `string \| number`       | -             | Default selected tab (uncontrolled)              |
| `onChange`    | `(value) => void`        | -             | Callback when tab changes                        |
| `variant`     | `'standard' \| 'fullWidth'` | `'standard'` | The variant of the tabs                          |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` | The orientation of the tabs                    |
| `className`   | `string`                 | -             | Additional CSS classes                           |

### Tab

| Prop       | Type            | Default | Description                       |
|------------|-----------------|---------|-----------------------------------|
| `value`    | `string \| number` | -    | The value of the tab              |
| `label`    | `ReactNode`     | -       | The label displayed on the tab    |
| `icon`     | `ReactNode`     | -       | Icon displayed before the label   |
| `disabled` | `boolean`       | `false` | If true, the tab is disabled      |
| `className`| `string`        | -       | Additional CSS classes            |

### TabPanel

| Prop       | Type            | Default | Description                       |
|------------|-----------------|---------|-----------------------------------|
| `value`    | `string \| number` | -    | The tab value this panel corresponds to |
| `className`| `string`        | -       | Additional CSS classes            |

## Examples

### Basic Usage

```tsx
import { Tabs, Tab } from '@nest/components';

<Tabs defaultValue="one">
  <Tab value="one" label="Tab One" />
  <Tab value="two" label="Tab Two" />
  <Tab value="three" label="Tab Three" />
</Tabs>
```

### With Panels

```tsx
import { Tabs, Tab, TabPanel } from '@nest/components';

<Tabs defaultValue="overview">
  <Tab value="overview" label="Overview" />
  <Tab value="features" label="Features" />
  <Tab value="pricing" label="Pricing" />
  
  <TabPanel value="overview">
    <p>Overview content</p>
  </TabPanel>
  <TabPanel value="features">
    <p>Features content</p>
  </TabPanel>
  <TabPanel value="pricing">
    <p>Pricing content</p>
  </TabPanel>
</Tabs>
```

### Controlled

```tsx
import { useState } from 'react';
import { Tabs, Tab, TabPanel } from '@nest/components';

const [value, setValue] = useState('one');

<Tabs value={value} onChange={(v) => setValue(v)}>
  <Tab value="one" label="First" />
  <Tab value="two" label="Second" />
  <TabPanel value="one">First panel</TabPanel>
  <TabPanel value="two">Second panel</TabPanel>
</Tabs>
```

### With Icons

```tsx
import { Tabs, Tab } from '@nest/components';
import { Settings, User, Bell } from 'lucide-react';

<Tabs defaultValue="settings">
  <Tab value="settings" icon={<Settings className="size-4" />} label="Settings" />
  <Tab value="profile" icon={<User className="size-4" />} label="Profile" />
  <Tab value="notifications" icon={<Bell className="size-4" />} label="Notifications" />
</Tabs>
```

### Full Width

```tsx
import { Tabs, Tab } from '@nest/components';

<Tabs variant="fullWidth">
  <Tab value="first" label="First" />
  <Tab value="second" label="Second" />
  <Tab value="third" label="Third" />
</Tabs>
```

### Vertical Orientation

```tsx
import { Tabs, Tab, TabPanel } from '@nest/components';

<div className="flex">
  <Tabs orientation="vertical" defaultValue="profile">
    <Tab value="profile" label="Profile" />
    <Tab value="account" label="Account" />
    <Tab value="security" label="Security" />
  </Tabs>
  <div className="flex-1 p-4">
    <TabPanel value="profile">Profile content</TabPanel>
    <TabPanel value="account">Account content</TabPanel>
    <TabPanel value="security">Security content</TabPanel>
  </div>
</div>
```

### Disabled Tab

```tsx
import { Tabs, Tab } from '@nest/components';

<Tabs defaultValue="enabled">
  <Tab value="enabled" label="Enabled" />
  <Tab value="disabled" label="Disabled" disabled />
  <Tab value="active" label="Active" />
</Tabs>
```

### Settings Page

```tsx
import { Tabs, Tab, TabPanel } from '@nest/components';

function SettingsPage() {
  return (
    <div className="border rounded-lg overflow-hidden">
      <Tabs defaultValue="general" className="bg-nest-muted/50">
        <Tab value="general" label="General" />
        <Tab value="appearance" label="Appearance" />
        <Tab value="notifications" label="Notifications" />
      </Tabs>
      <div className="p-6">
        <TabPanel value="general">
          <h3>General Settings</h3>
          <input type="text" placeholder="Site name" />
        </TabPanel>
        <TabPanel value="appearance">
          <h3>Appearance Settings</h3>
          <label>
            <input type="checkbox" /> Dark mode
          </label>
        </TabPanel>
        <TabPanel value="notifications">
          <h3>Notification Settings</h3>
          <p>Configure notifications...</p>
        </TabPanel>
      </div>
    </div>
  );
}
```

## Accessibility

- Tabs container uses `role="tablist"`
- Each Tab uses `role="tab"` with `aria-selected` to indicate selection
- TabPanel uses `role="tabpanel"` with `hidden` when not selected
- Tabs support keyboard navigation:
  - Arrow Left/Right (or Up/Down for vertical) to move between tabs
  - Enter/Space to activate a tab
  - Home/End to go to first/last tab
- Always provide meaningful labels for tabs
- When using icons, ensure they have appropriate alt text or are marked decorative

## Keyboard Navigation

| Key          | Action                              |
|--------------|-------------------------------------|
| Arrow Left   | Move to previous tab (horizontal)   |
| Arrow Right  | Move to next tab (horizontal)       |
| Arrow Up     | Move to previous tab (vertical)     |
| Arrow Down   | Move to next tab (vertical)         |
| Home         | Move to first tab                   |
| End          | Move to last tab                    |
| Enter/Space  | Activate focused tab                |
