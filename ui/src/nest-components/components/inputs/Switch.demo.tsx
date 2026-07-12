import { useState } from 'react';
import { Switch } from './Switch';
import { FormLabel } from './FormControl';

export function SwitchDemos() {
  const [checked, setChecked] = useState(true);
  const [smallChecked, setSmallChecked] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <Switch />
            <span className="text-sm text-nest-foreground">Unchecked</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked />
            <span className="text-sm text-nest-foreground">Default checked</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch checked={checked} onChange={(e) => setChecked(e.target.checked)} />
            <span className="text-sm text-nest-foreground">Controlled: {checked ? 'On' : 'Off'}</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="grid grid-cols-2 gap-4">
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="primary" />
            <span className="text-sm">Primary</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="secondary" />
            <span className="text-sm">Secondary</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="success" />
            <span className="text-sm">Success</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="error" />
            <span className="text-sm">Error</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="warning" />
            <span className="text-sm">Warning</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch defaultChecked color="info" />
            <span className="text-sm">Info</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <Switch size="small" checked={smallChecked} onChange={(e) => setSmallChecked(e.target.checked)} />
            <span className="text-sm">Small</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch size="medium" defaultChecked />
            <span className="text-sm">Medium (default)</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-3">
            <Switch disabled />
            <span className="text-sm">Disabled off</span>
          </div>
          <div className="flex items-center gap-3">
            <Switch disabled defaultChecked />
            <span className="text-sm">Disabled on</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Labels</h2>
        <div className="space-y-4 border border-nest-border rounded-nest-md p-4">
          <div className="flex items-center justify-between">
            <div>
              <FormLabel>Notifications</FormLabel>
              <p className="text-xs text-nest-muted">Receive push notifications</p>
            </div>
            <Switch defaultChecked />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <FormLabel>Email Updates</FormLabel>
              <p className="text-xs text-nest-muted">Weekly digest emails</p>
            </div>
            <Switch />
          </div>
          <div className="flex items-center justify-between">
            <div>
              <FormLabel>Dark Mode</FormLabel>
              <p className="text-xs text-nest-muted">Use dark theme</p>
            </div>
            <Switch defaultChecked />
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Settings Panel</h2>
        <div className="border border-nest-border rounded-nest-md divide-y">
          <div className="flex items-center justify-between p-4">
            <div>
              <FormLabel>Airplane Mode</FormLabel>
              <p className="text-xs text-nest-muted">Disable all connectivity</p>
            </div>
            <Switch />
          </div>
          <div className="flex items-center justify-between p-4">
            <div>
              <FormLabel>Wi-Fi</FormLabel>
              <p className="text-xs text-nest-muted">Connect to wireless networks</p>
            </div>
            <Switch defaultChecked />
          </div>
          <div className="flex items-center justify-between p-4">
            <div>
              <FormLabel>Bluetooth</FormLabel>
              <p className="text-xs text-nest-muted">Connect to nearby devices</p>
            </div>
            <Switch defaultChecked />
          </div>
          <div className="flex items-center justify-between p-4">
            <div>
              <FormLabel>Location Services</FormLabel>
              <p className="text-xs text-nest-muted">Allow apps to use your location</p>
            </div>
            <Switch />
          </div>
        </div>
      </section>
    </div>
  );
}
