import { useState } from 'react';
import { Checkbox } from './Checkbox';
import { FormControl, FormLabel } from './FormControl';

export function CheckboxDemos() {
  const [checked, setChecked] = useState(true);
  const [indeterminateChecked, setIndeterminateChecked] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Checkbox />
            <span className="text-sm text-nest-foreground">Unchecked</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked />
            <span className="text-sm text-nest-foreground">Default checked</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox checked={checked} onChange={(e) => setChecked(e.target.checked)} />
            <span className="text-sm text-nest-foreground">Controlled: {checked ? 'Checked' : 'Unchecked'}</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Indeterminate State</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Checkbox indeterminate />
            <span className="text-sm text-nest-foreground">Indeterminate</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox indeterminate checked={indeterminateChecked} onChange={(e) => setIndeterminateChecked(e.target.checked)} />
            <span className="text-sm text-nest-foreground">Indeterminate + controlled</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="grid grid-cols-2 gap-4">
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="primary" />
            <span className="text-sm">Primary</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="secondary" />
            <span className="text-sm">Secondary</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="success" />
            <span className="text-sm">Success</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="error" />
            <span className="text-sm">Error</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="warning" />
            <span className="text-sm">Warning</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox defaultChecked color="info" />
            <span className="text-sm">Info</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Checkbox size="small" defaultChecked />
            <span className="text-sm">Small</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox size="medium" defaultChecked />
            <span className="text-sm">Medium (default)</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <Checkbox disabled />
            <span className="text-sm">Disabled unchecked</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox disabled defaultChecked />
            <span className="text-sm">Disabled checked</span>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox disabled indeterminate />
            <span className="text-sm">Disabled indeterminate</span>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With FormLabel</h2>
        <div className="space-y-3">
          <FormControl>
            <div className="flex items-center gap-2">
              <Checkbox id="terms" />
              <FormLabel htmlFor="terms">I agree to the terms and conditions</FormLabel>
            </div>
          </FormControl>
          <FormControl>
            <div className="flex items-center gap-2">
              <Checkbox id="newsletter" defaultChecked />
              <FormLabel htmlFor="newsletter">Subscribe to newsletter</FormLabel>
            </div>
          </FormControl>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Checkbox Group</h2>
        <div className="space-y-3 border border-nest-border rounded-nest-md p-4">
          <div className="flex items-center gap-2">
            <Checkbox id="fruit-apple" />
            <label htmlFor="fruit-apple" className="text-sm">Apple</label>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox id="fruit-banana" defaultChecked />
            <label htmlFor="fruit-banana" className="text-sm">Banana</label>
          </div>
          <div className="flex items-center gap-2">
            <Checkbox id="fruit-cherry" />
            <label htmlFor="fruit-cherry" className="text-sm">Cherry</label>
          </div>
        </div>
      </section>
    </div>
  );
}
