import { Link } from './Link';

/**
 * Link Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function LinkDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Links */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Links</h2>
        <div className="flex flex-col gap-2">
          <Link href="#basic">Default Link (hover underline)</Link>
          <Link href="#basic" underline="none">No Underline Link</Link>
          <Link href="#basic" underline="always">Always Underlined Link</Link>
        </div>
      </section>

      {/* Color Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Color Variants</h2>
        <div className="flex flex-col gap-2">
          <Link href="#color" color="primary">Primary Color Link</Link>
          <Link href="#color" color="inherit">Inherit Color Link</Link>
          <p className="text-nest-muted">
            <Link href="#color" color="inherit">Inherited link in muted text</Link>
          </p>
        </div>
      </section>

      {/* Underline Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Underline Comparison</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-4">
            <span className="w-24 text-sm text-nest-muted">none:</span>
            <Link href="#underline" underline="none">No underline</Link>
          </div>
          <div className="flex items-center gap-4">
            <span className="w-24 text-sm text-nest-muted">hover:</span>
            <Link href="#underline" underline="hover">Hover to underline</Link>
          </div>
          <div className="flex items-center gap-4">
            <span className="w-24 text-sm text-nest-muted">always:</span>
            <Link href="#underline" underline="always">Always underlined</Link>
          </div>
        </div>
      </section>

      {/* External Links */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">External Links</h2>
        <div className="flex flex-col gap-2">
          <Link href="https://example.com" external>
            External Link (opens in new tab)
          </Link>
          <Link href="https://github.com" external>
            GitHub (opens in new tab)
          </Link>
          <Link href="/internal">
            Internal Link (same tab)
          </Link>
        </div>
      </section>

      {/* Links in Text */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Links in Text</h2>
        <p className="text-nest-foreground">
          This is a paragraph with an{' '}
          <Link href="#inline" color="primary">inline link</Link>{' '}
          in the middle of text. You can also have{' '}
          <Link href="#inline" underline="always">always underlined links</Link>{' '}
          for emphasis.
        </p>
        <p className="text-nest-muted">
          Muted text with{' '}
          <Link href="#inline" color="inherit">inherited color link</Link>.
        </p>
      </section>

      {/* Link with Icons */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Links with Icons</h2>
        <div className="flex flex-col gap-2">
          <Link href="#icon" className="inline-flex items-center gap-1">
            <span>📄</span>
            <span>Document Link</span>
          </Link>
          <Link href="#icon" className="inline-flex items-center gap-1">
            <span>🔗</span>
            <span>External Resource</span>
          </Link>
          <Link href="#icon" external className="inline-flex items-center gap-1">
            <span>Link with arrow</span>
            <span>→</span>
          </Link>
        </div>
      </section>

      {/* Custom Component Link */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <div className="flex flex-col gap-2">
          <Link component="button" onClick={() => alert('Button link clicked!')}>
            Button as Link
          </Link>
          <Link component="span" className="cursor-pointer">
            Span as Link (no href)
          </Link>
        </div>
      </section>

      {/* Navigation Links */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Navigation Links</h2>
        <nav className="flex gap-4">
          <Link href="#nav" underline="none">Home</Link>
          <Link href="#nav" underline="none">About</Link>
          <Link href="#nav" underline="none">Services</Link>
          <Link href="#nav" underline="none">Contact</Link>
        </nav>
        <p className="mt-2 text-sm text-nest-muted">Navigation with underline="none"</p>
      </section>

      {/* Breadcrumb-style Links */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Breadcrumb Links</h2>
        <div className="flex items-center gap-2 text-sm">
          <Link href="#breadcrumb" color="primary">Home</Link>
          <span className="text-nest-muted">/</span>
          <Link href="#breadcrumb" color="primary">Products</Link>
          <span className="text-nest-muted">/</span>
          <Link href="#breadcrumb" color="primary">Electronics</Link>
          <span className="text-nest-muted">/</span>
          <span className="text-nest-foreground">Cameras</span>
        </div>
      </section>

      {/* Focus States */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Focus States</h2>
        <p className="text-sm text-nest-muted">
          Tab to see focus ring on links:
        </p>
        <div className="flex gap-4">
          <Link href="#focus">Link 1</Link>
          <Link href="#focus">Link 2</Link>
          <Link href="#focus">Link 3</Link>
        </div>
      </section>

      {/* All Combinations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">All Combinations</h2>
        <div className="grid grid-cols-2 gap-4">
          <Link href="#combo" color="primary" underline="hover">
            Primary + Hover
          </Link>
          <Link href="#combo" color="primary" underline="none">
            Primary + None
          </Link>
          <Link href="#combo" color="primary" underline="always">
            Primary + Always
          </Link>
          <Link href="#combo" color="inherit" underline="hover">
            Inherit + Hover
          </Link>
          <Link href="#combo" color="inherit" underline="none">
            Inherit + None
          </Link>
          <Link href="#combo" color="inherit" underline="always">
            Inherit + Always
          </Link>
        </div>
      </section>
    </div>
  );
}
