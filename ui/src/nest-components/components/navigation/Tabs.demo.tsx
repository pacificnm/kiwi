import { useState } from 'react';
import { Tabs, Tab, TabPanel } from './Tabs';
import { Settings, User, Mail, Bell } from 'lucide-react';

export function TabsDemos() {
  const [value, setValue] = useState('one');
  const [verticalValue, setVerticalValue] = useState('profile');

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <Tabs value={value} onChange={(v) => setValue(v as string)}>
          <Tab value="one" label="Tab One" />
          <Tab value="two" label="Tab Two" />
          <Tab value="three" label="Tab Three" />
        </Tabs>
        <div className="mt-4 p-4 border border-nest-border rounded-nest-md">
          <p className="text-sm text-nest-muted">Selected: {value}</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Panels</h2>
        <Tabs defaultValue="overview">
          <Tab value="overview" label="Overview" />
          <Tab value="features" label="Features" />
          <Tab value="pricing" label="Pricing" />
          <TabPanel value="overview">
            <p className="text-nest-foreground">Overview content goes here. This panel is shown when the Overview tab is selected.</p>
          </TabPanel>
          <TabPanel value="features">
            <p className="text-nest-foreground">Features content goes here. List your product features in this panel.</p>
          </TabPanel>
          <TabPanel value="pricing">
            <p className="text-nest-foreground">Pricing information goes here. Display your pricing plans in this panel.</p>
          </TabPanel>
        </Tabs>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Icons</h2>
        <Tabs defaultValue="settings">
          <Tab value="settings" icon={<Settings className="size-4" />} label="Settings" />
          <Tab value="profile" icon={<User className="size-4" />} label="Profile" />
          <Tab value="notifications" icon={<Bell className="size-4" />} label="Notifications" />
          <Tab value="messages" icon={<Mail className="size-4" />} label="Messages" />
        </Tabs>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width</h2>
        <Tabs variant="fullWidth" defaultValue="first">
          <Tab value="first" label="First" />
          <Tab value="second" label="Second" />
          <Tab value="third" label="Third" />
        </Tabs>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Vertical Orientation</h2>
        <div className="flex gap-4">
          <Tabs orientation="vertical" value={verticalValue} onChange={(v) => setVerticalValue(v as string)}>
            <Tab value="profile" label="Profile" />
            <Tab value="account" label="Account" />
            <Tab value="security" label="Security" />
            <Tab value="billing" label="Billing" />
          </Tabs>
          <div className="flex-1 p-4 border border-nest-border rounded-nest-md">
            <p className="text-nest-foreground">Content for {verticalValue} panel</p>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled Tab</h2>
        <Tabs defaultValue="enabled">
          <Tab value="enabled" label="Enabled" />
          <Tab value="disabled" label="Disabled" disabled />
          <Tab value="also-enabled" label="Also Enabled" />
        </Tabs>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Settings Page Layout</h2>
        <div className="border border-nest-border rounded-nest-md overflow-hidden">
          <Tabs defaultValue="general" className="bg-nest-muted/50">
            <Tab value="general" label="General" />
            <Tab value="appearance" label="Appearance" />
            <Tab value="notifications" label="Notifications" />
            <Tab value="advanced" label="Advanced" />
          </Tabs>
          <div className="p-6">
            <TabPanel value="general">
              <h3 className="text-lg font-semibold mb-4">General Settings</h3>
              <div className="space-y-4">
                <div>
                  <label className="block text-sm font-medium mb-1">Site Name</label>
                  <input type="text" className="w-full border border-nest-border rounded-nest-md px-3 py-2" placeholder="My Site" />
                </div>
                <div>
                  <label className="block text-sm font-medium mb-1">Description</label>
                  <textarea className="w-full border border-nest-border rounded-nest-md px-3 py-2" rows={3} placeholder="Site description" />
                </div>
              </div>
            </TabPanel>
            <TabPanel value="appearance">
              <h3 className="text-lg font-semibold mb-4">Appearance Settings</h3>
              <div className="space-y-4">
                <label className="flex items-center gap-2">
                  <input type="checkbox" defaultChecked />
                  <span className="text-sm">Dark mode</span>
                </label>
                <label className="flex items-center gap-2">
                  <input type="checkbox" />
                  <span className="text-sm">Compact view</span>
                </label>
              </div>
            </TabPanel>
            <TabPanel value="notifications">
              <h3 className="text-lg font-semibold mb-4">Notification Settings</h3>
              <p className="text-nest-muted">Configure how you receive notifications.</p>
            </TabPanel>
            <TabPanel value="advanced">
              <h3 className="text-lg font-semibold mb-4">Advanced Settings</h3>
              <p className="text-nest-muted text-sm">Warning: Changes to advanced settings may affect system stability.</p>
            </TabPanel>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Uncontrolled</h2>
        <Tabs defaultValue="tab1">
          <Tab value="tab1" label="Tab 1" />
          <Tab value="tab2" label="Tab 2" />
          <Tab value="tab3" label="Tab 3" />
        </Tabs>
      </section>
    </div>
  );
}
