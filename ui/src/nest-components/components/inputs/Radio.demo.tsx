import { useState } from 'react';
import { Radio, RadioGroup } from './Radio';

export function RadioDemos() {
  const [value, setValue] = useState('one');
  const [rowValue, setRowValue] = useState('a');

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="space-y-4">
          <RadioGroup value={value} onChange={(v) => setValue(v)}>
            <Radio value="one" label="Option One" />
            <Radio value="two" label="Option Two" />
            <Radio value="three" label="Option Three" />
          </RadioGroup>
          <p className="text-sm text-nest-muted">Selected: {value}</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Uncontrolled</h2>
        <RadioGroup defaultValue="b">
          <Radio value="a" label="Option A" />
          <Radio value="b" label="Option B" />
          <Radio value="c" label="Option C" />
        </RadioGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Row Layout</h2>
        <RadioGroup row value={rowValue} onChange={(v) => setRowValue(v)}>
          <Radio value="a" label="A" />
          <Radio value="b" label="B" />
          <Radio value="c" label="C" />
          <Radio value="d" label="D" />
        </RadioGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="grid grid-cols-2 gap-4">
          <RadioGroup row color="primary">
            <Radio value="p" label="Primary" defaultChecked />
          </RadioGroup>
          <RadioGroup row color="secondary">
            <Radio value="s" label="Secondary" defaultChecked />
          </RadioGroup>
          <RadioGroup row color="success">
            <Radio value="g" label="Success" defaultChecked />
          </RadioGroup>
          <RadioGroup row color="error">
            <Radio value="e" label="Error" defaultChecked />
          </RadioGroup>
          <RadioGroup row color="warning">
            <Radio value="w" label="Warning" defaultChecked />
          </RadioGroup>
          <RadioGroup row color="info">
            <Radio value="i" label="Info" defaultChecked />
          </RadioGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <RadioGroup row size="small">
            <Radio value="1" label="Small" defaultChecked />
            <Radio value="2" label="Small" />
          </RadioGroup>
          <RadioGroup row size="medium">
            <Radio value="3" label="Medium" defaultChecked />
            <Radio value="4" label="Medium" />
          </RadioGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <div className="space-y-4">
          <RadioGroup disabled>
            <Radio value="one" label="Disabled One" defaultChecked />
            <Radio value="two" label="Disabled Two" />
          </RadioGroup>
          <RadioGroup>
            <Radio value="one" label="Individual disabled" disabled />
            <Radio value="two" label="Enabled" />
          </RadioGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Without Labels (Icon Only)</h2>
        <RadioGroup row defaultValue="1">
          <Radio value="1" aria-label="Option 1" />
          <Radio value="2" aria-label="Option 2" />
          <Radio value="3" aria-label="Option 3" />
        </RadioGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Form Context</h2>
        <div className="border border-nest-border rounded-nest-md p-4 space-y-4">
          <p className="text-sm font-medium text-nest-foreground">Select your preferred contact method:</p>
          <RadioGroup name="contact" defaultValue="email">
            <Radio value="email" label="Email" />
            <Radio value="phone" label="Phone" />
            <Radio value="sms" label="SMS" />
          </RadioGroup>
        </div>
      </section>
    </div>
  );
}
