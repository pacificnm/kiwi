import { Container } from './Container';

/**
 * Container Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function ContainerDemos() {
  const content = (
    <div className="rounded-nest-md bg-nest-primary/20 p-8 text-center">
      <p className="font-medium text-nest-primary">Content</p>
      <p className="text-sm text-nest-muted">Resize window to see max-width effect</p>
    </div>
  );

  return (
    <div className="space-y-8 p-6">
      {/* Default Container */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Default Container</h2>
        <div className="border border-nest-border bg-nest-surface">
          <Container>
            {content}
          </Container>
        </div>
        <p className="mt-2 text-sm text-nest-muted">maxWidth="lg" (default), with gutters (px-4)</p>
      </section>

      {/* Max Width Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Max Width Variations</h2>
        <div className="space-y-4">
          <div className="border border-nest-border bg-nest-surface">
            <Container maxWidth="sm">
              <div className="rounded-nest-md bg-nest-primary/20 p-4 text-center text-nest-primary">
                maxWidth="sm" (640px)
              </div>
            </Container>
          </div>
          <div className="border border-nest-border bg-nest-surface">
            <Container maxWidth="md">
              <div className="rounded-nest-md bg-nest-secondary/20 p-4 text-center text-nest-secondary">
                maxWidth="md" (768px)
              </div>
            </Container>
          </div>
          <div className="border border-nest-border bg-nest-surface">
            <Container maxWidth="lg">
              <div className="rounded-nest-md bg-nest-accent/20 p-4 text-center text-nest-accent">
                maxWidth="lg" (1024px)
              </div>
            </Container>
          </div>
          <div className="border border-nest-border bg-nest-surface">
            <Container maxWidth="xl">
              <div className="rounded-nest-md bg-nest-success/20 p-4 text-center text-nest-success">
                maxWidth="xl" (1280px)
              </div>
            </Container>
          </div>
          <div className="border border-nest-border bg-nest-surface">
            <Container maxWidth="xxl">
              <div className="rounded-nest-md bg-nest-warning/20 p-4 text-center text-nest-warning">
                maxWidth="xxl" (1536px)
              </div>
            </Container>
          </div>
        </div>
      </section>

      {/* Full Width (No Max Width) */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Full Width (No Max Width)</h2>
        <div className="border border-nest-border bg-nest-surface">
          <Container maxWidth={false}>
            <div className="rounded-nest-md bg-nest-muted/20 p-4 text-center">
              <p className="font-medium text-nest-foreground">maxWidth={false}</p>
              <p className="text-sm text-nest-muted">Spans full width of parent</p>
            </div>
          </Container>
        </div>
      </section>

      {/* Disable Gutters */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disable Gutters</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">With gutters (default)</p>
            <div className="border border-nest-border bg-nest-surface">
              <Container>
                <div className="rounded-nest-md border-2 border-dashed border-nest-primary p-4 text-center">
                  Content with px-4 padding
                </div>
              </Container>
            </div>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">disableGutters=true</p>
            <div className="border border-nest-border bg-nest-surface">
              <Container disableGutters>
                <div className="rounded-nest-md border-2 border-dashed border-nest-primary p-4 text-center">
                  Content edge-to-edge (no horizontal padding)
                </div>
              </Container>
            </div>
          </div>
        </div>
      </section>

      {/* Fixed Width */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Fixed Width</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">maxWidth="md" (max-width)</p>
            <div className="border border-nest-border bg-nest-surface">
              <Container maxWidth="md">
                <div className="rounded-nest-md bg-nest-primary/20 p-4 text-center">
                  Shrinks on smaller screens
                </div>
              </Container>
            </div>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">maxWidth="md" + fixed (fixed width)</p>
            <div className="border border-nest-border bg-nest-surface">
              <Container maxWidth="md" fixed>
                <div className="rounded-nest-md bg-nest-secondary/20 p-4 text-center">
                  Stays at fixed width (may overflow on small screens)
                </div>
              </Container>
            </div>
          </div>
        </div>
      </section>

      {/* Custom Component */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <Container component="main" maxWidth="lg" className="border border-nest-border">
          <h1 className="mb-4 text-2xl font-bold">Main Content</h1>
          <p className="text-nest-muted">Renders as &lt;main&gt; element</p>
        </Container>
      </section>

      {/* Nested Containers */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Nested Containers</h2>
        <Container maxWidth="xl" className="border border-nest-border bg-nest-surface">
          <div className="py-4">
            <p className="mb-2 font-medium">Outer Container (xl)</p>
            <Container maxWidth="md" className="border border-nest-border bg-nest-background">
              <div className="py-4">
                <p className="mb-2 font-medium">Inner Container (md)</p>
                <Container maxWidth="sm" className="border border-nest-border bg-nest-surface">
                  <div className="py-4">
                    <p className="font-medium">Innermost Container (sm)</p>
                  </div>
                </Container>
              </div>
            </Container>
          </div>
        </Container>
      </section>

      {/* Page Layout Example */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Page Layout Example</h2>
        <div className="border border-nest-border">
          <Container component="header" maxWidth="lg" className="border-b border-nest-border bg-nest-background">
            <div className="flex h-16 items-center justify-between">
              <span className="font-bold text-nest-primary">Logo</span>
              <nav className="flex gap-4 text-sm text-nest-muted">
                <a href="#">Home</a>
                <a href="#">About</a>
                <a href="#">Contact</a>
              </nav>
            </div>
          </Container>

          <Container component="main" maxWidth="lg" className="min-h-[200px] py-8">
            <h2 className="mb-4 text-xl font-semibold">Page Content</h2>
            <p className="text-nest-muted">
              This is the main content area. The container constrains the width
              for better readability on large screens.
            </p>
          </Container>

          <Container component="footer" maxWidth="lg" className="border-t border-nest-border bg-nest-background">
            <div className="py-4 text-center text-sm text-nest-muted">
              © 2024 Nest. All rights reserved.
            </div>
          </Container>
        </div>
      </section>
    </div>
  );
}
