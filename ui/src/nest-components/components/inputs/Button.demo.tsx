import { useState } from 'react';
import { Button } from './Button';
import { Save, Trash2, Search, Download } from 'lucide-react';

/**
 * Button Component Demos
 *
 * Copy these examples into your app to get started.
 */

export function ButtonDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Variants Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="flex flex-wrap gap-4">
          <Button variant="contained">Contained</Button>
          <Button variant="outlined">Outlined</Button>
          <Button variant="text">Text</Button>
        </div>
      </section>

      {/* Colors Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="flex flex-wrap gap-4">
          <Button variant="contained" color="primary">Primary</Button>
          <Button variant="contained" color="secondary">Secondary</Button>
          <Button variant="contained" color="accent">Accent</Button>
          <Button variant="contained" color="error">Error</Button>
          <Button variant="contained" color="success">Success</Button>
          <Button variant="contained" color="warning">Warning</Button>
        </div>
      </section>

      {/* Sizes Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="flex flex-wrap items-center gap-4">
          <Button size="small">Small</Button>
          <Button size="medium">Medium</Button>
          <Button size="large">Large</Button>
        </div>
      </section>

      {/* Icons Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Icons</h2>
        <div className="flex flex-wrap gap-4">
          <Button startIcon={<Save />}>Save</Button>
          <Button endIcon={<Download />}>Download</Button>
          <Button variant="outlined" startIcon={<Trash2 />} color="error">
            Delete
          </Button>
          <Button variant="text" startIcon={<Search />}>
            Search
          </Button>
        </div>
      </section>

      {/* States Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">States</h2>
        <div className="flex flex-wrap gap-4">
          <Button disabled>Disabled</Button>
          <Button loading>Loading</Button>
          <Button variant="outlined" disabled>
            Disabled Outlined
          </Button>
        </div>
      </section>

      {/* Full Width Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width</h2>
        <div className="space-y-4">
          <Button fullWidth>Full Width Button</Button>
          <Button fullWidth variant="outlined" color="secondary">
            Full Width Outlined
          </Button>
        </div>
      </section>

      {/* Interactive Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Interactive</h2>
        <InteractiveDemo />
      </section>
    </div>
  );
}

function InteractiveDemo() {
  const [count, setCount] = useState(0);
  const [loading, setLoading] = useState(false);

  const handleIncrement = () => setCount((c) => c + 1);
  const handleDecrement = () => setCount((c) => Math.max(0, c - 1));
  const handleAsyncAction = () => {
    setLoading(true);
    setTimeout(() => setLoading(false), 2000);
  };

  return (
    <div className="rounded-nest-md border border-nest-border bg-nest-surface p-4">
      <div className="mb-4 flex items-center gap-4">
        <Button onClick={handleDecrement} size="small">
          -
        </Button>
        <span className="min-w-[2rem] text-center text-nest-foreground">{count}</span>
        <Button onClick={handleIncrement} size="small">
          +
        </Button>
      </div>
      <Button
        variant="contained"
        color="success"
        loading={loading}
        onClick={handleAsyncAction}
        startIcon={<Download />}
      >
        {loading ? 'Processing...' : 'Simulate Async Action'}
      </Button>
    </div>
  );
}
