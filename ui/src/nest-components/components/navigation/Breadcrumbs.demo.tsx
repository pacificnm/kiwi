import { Breadcrumbs } from './Breadcrumbs';
import { Slash, Home } from 'lucide-react';

/**
 * Breadcrumbs Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function BreadcrumbsDemos() {
  const defaultItems = [
    { label: 'Home', href: '/' },
    { label: 'Products', href: '/products' },
    { label: 'Electronics', href: '/products/electronics' },
    { label: 'Cameras', current: true },
  ];

  return (
    <div className="space-y-8 p-6">
      {/* Basic Breadcrumbs */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Breadcrumbs</h2>
        <Breadcrumbs items={defaultItems} />
      </section>

      {/* Short Breadcrumbs */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Short Breadcrumbs</h2>
        <div className="space-y-4">
          <Breadcrumbs items={[{ label: 'Home', href: '/' }]} />
          <Breadcrumbs items={[
            { label: 'Home', href: '/' },
            { label: 'Current', current: true },
          ]} />
          <Breadcrumbs items={[
            { label: 'Home', href: '/' },
            { label: 'Section', href: '/section' },
            { label: 'Current', current: true },
          ]} />
        </div>
      </section>

      {/* Custom Separators */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Separators</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">Chevron (default)</p>
            <Breadcrumbs items={defaultItems.slice(0, 3)} />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">Slash</p>
            <Breadcrumbs items={defaultItems.slice(0, 3)} separator={<Slash className="h-4 w-4 text-nest-muted" />} />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">Text separator</p>
            <Breadcrumbs items={defaultItems.slice(0, 3)} separator="/" />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">Arrow</p>
            <Breadcrumbs items={defaultItems.slice(0, 3)} separator="→" />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">Dot</p>
            <Breadcrumbs items={defaultItems.slice(0, 3)} separator="•" />
          </div>
        </div>
      </section>

      {/* Max Items Collapsing */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Max Items Collapsing</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">No collapsing (default)</p>
            <Breadcrumbs items={defaultItems} />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">maxItems={3}</p>
            <Breadcrumbs items={defaultItems} maxItems={3} />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">maxItems={2}</p>
            <Breadcrumbs items={defaultItems} maxItems={2} />
          </div>
        </div>
      </section>

      {/* With Icon Labels */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Icon Labels</h2>
        <Breadcrumbs
          items={[
            { label: <Home className="h-4 w-4" />, href: '/' },
            { label: 'Products', href: '/products' },
            { label: 'Cameras', current: true },
          ]}
        />
      </section>

      {/* All Links (no current) */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">All Links (no current)</h2>
        <Breadcrumbs
          items={[
            { label: 'Home', href: '/' },
            { label: 'Products', href: '/products' },
            { label: 'Electronics', href: '/products/electronics' },
          ]}
        />
      </section>

      {/* Custom Component */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <Breadcrumbs items={defaultItems.slice(0, 3)} component="div" />
        <p className="mt-2 text-sm text-nest-muted">Renders as &lt;div&gt; instead of &lt;nav&gt;</p>
      </section>

      {/* In Different Contexts */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">In Different Contexts</h2>
        <div className="space-y-4">
          {/* In header */}
          <div className="border-b border-nest-border pb-4">
            <Breadcrumbs items={defaultItems} />
          </div>
          {/* In card */}
          <div className="rounded-nest-md border border-nest-border p-4">
            <Breadcrumbs items={defaultItems} />
          </div>
          {/* Dark background simulation */}
          <div className="rounded-nest-md bg-nest-foreground p-4">
            <Breadcrumbs items={defaultItems} />
          </div>
        </div>
      </section>

      {/* Long Breadcrumb Path */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Long Breadcrumb Path</h2>
        <Breadcrumbs
          items={[
            { label: 'Home', href: '/' },
            { label: 'Products', href: '/products' },
            { label: 'Electronics', href: '/products/electronics' },
            { label: 'Cameras', href: '/products/electronics/cameras' },
            { label: 'DSLR', href: '/products/electronics/cameras/dslr' },
            { label: 'Canon', href: '/products/electronics/cameras/dslr/canon' },
            { label: 'EOS R5', current: true },
          ]}
          maxItems={5}
        />
      </section>

      {/* Accessibility Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Accessibility</h2>
        <Breadcrumbs
          items={[
            { label: 'Home', href: '/' },
            { label: 'Products', href: '/products' },
            { label: 'Current Page', current: true },
          ]}
          ariaLabel="You are here"
        />
        <p className="mt-2 text-sm text-nest-muted">
          Custom aria-label="You are here" - tab to see focus states on links
        </p>
      </section>
    </div>
  );
}
