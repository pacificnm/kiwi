import { useState } from 'react';
import { ToggleButton, ToggleButtonGroup } from './ToggleButton';

export function ToggleButtonDemos() {
  const [align, setAlign] = useState('left');
  const [formats, setFormats] = useState<string[]>(['bold']);
  const [view, setView] = useState('grid');

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Exclusive Selection (Single)</h2>
        <div className="space-y-4">
          <ToggleButtonGroup value={align} onChange={(v) => setAlign(v as string)} exclusive row>
            <ToggleButton value="left" aria-label="Left align">L</ToggleButton>
            <ToggleButton value="center" aria-label="Center align">C</ToggleButton>
            <ToggleButton value="right" aria-label="Right align">R</ToggleButton>
          </ToggleButtonGroup>
          <p className="text-sm text-nest-muted">Selected: {align}</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Multiple Selection</h2>
        <div className="space-y-4">
          <ToggleButtonGroup value={formats} onChange={(v) => setFormats(v as string[])} exclusive={false} row>
            <ToggleButton value="bold" aria-label="Bold">
              <strong>B</strong>
            </ToggleButton>
            <ToggleButton value="italic" aria-label="Italic">
              <em>I</em>
            </ToggleButton>
            <ToggleButton value="underline" aria-label="Underline">
              <u>U</u>
            </ToggleButton>
          </ToggleButtonGroup>
          <p className="text-sm text-nest-muted">Selected: {formats.join(', ') || 'none'}</p>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Labels</h2>
        <ToggleButtonGroup value={view} onChange={(v) => setView(v as string)} exclusive row>
          <ToggleButton value="list" label="List View" />
          <ToggleButton value="grid" label="Grid View" />
          <ToggleButton value="tiles" label="Tiles View" />
        </ToggleButtonGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-4">
          <ToggleButtonGroup defaultValue="primary" color="primary" exclusive row>
            <ToggleButton value="primary" label="Primary" />
          </ToggleButtonGroup>
          <ToggleButtonGroup defaultValue="secondary" color="secondary" exclusive row>
            <ToggleButton value="secondary" label="Secondary" />
          </ToggleButtonGroup>
          <ToggleButtonGroup defaultValue="success" color="success" exclusive row>
            <ToggleButton value="success" label="Success" />
          </ToggleButtonGroup>
          <ToggleButtonGroup defaultValue="error" color="error" exclusive row>
            <ToggleButton value="error" label="Error" />
          </ToggleButtonGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="space-y-4">
          <ToggleButtonGroup size="small" defaultValue="one" exclusive row>
            <ToggleButton value="one" label="Small" />
            <ToggleButton value="two" label="Small" />
          </ToggleButtonGroup>
          <ToggleButtonGroup size="medium" defaultValue="one" exclusive row>
            <ToggleButton value="one" label="Medium" />
            <ToggleButton value="two" label="Medium" />
          </ToggleButtonGroup>
          <ToggleButtonGroup size="large" defaultValue="one" exclusive row>
            <ToggleButton value="one" label="Large" />
            <ToggleButton value="two" label="Large" />
          </ToggleButtonGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Vertical Layout</h2>
        <ToggleButtonGroup defaultValue="day" exclusive>
          <ToggleButton value="day" label="Day" />
          <ToggleButton value="week" label="Week" />
          <ToggleButton value="month" label="Month" />
        </ToggleButtonGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <ToggleButtonGroup disabled defaultValue="one" exclusive row>
          <ToggleButton value="one" label="Disabled One" />
          <ToggleButton value="two" label="Disabled Two" />
          <ToggleButton value="three" label="Disabled Three" />
        </ToggleButtonGroup>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Text Formatting Toolbar</h2>
        <div className="border border-nest-border rounded-nest-md p-2 inline-flex flex-col gap-2">
          <ToggleButtonGroup defaultValue={['bold']} exclusive={false} row>
            <ToggleButton value="bold" aria-label="Bold"><strong>B</strong></ToggleButton>
            <ToggleButton value="italic" aria-label="Italic"><em>I</em></ToggleButton>
            <ToggleButton value="underline" aria-label="Underline"><u>U</u></ToggleButton>
          </ToggleButtonGroup>
          <ToggleButtonGroup defaultValue="left" exclusive row>
            <ToggleButton value="left" aria-label="Left">≡</ToggleButton>
            <ToggleButton value="center" aria-label="Center">≡</ToggleButton>
            <ToggleButton value="right" aria-label="Right">≡</ToggleButton>
          </ToggleButtonGroup>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Uncontrolled</h2>
        <ToggleButtonGroup defaultValue="first" exclusive row>
          <ToggleButton value="first" label="First" />
          <ToggleButton value="second" label="Second" />
          <ToggleButton value="third" label="Third" />
        </ToggleButtonGroup>
      </section>
    </div>
  );
}
