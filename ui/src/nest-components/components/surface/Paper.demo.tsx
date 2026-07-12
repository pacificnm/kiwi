import { Paper } from './Paper';

/**
 * Paper Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function PaperDemos() {
  const content = (
    <div className="p-4">
      <h3 className="mb-2 font-semibold">Paper Content</h3>
      <p className="text-sm text-nest-muted">
        Paper is a surface container that can have elevation (shadow) or a border.
      </p>
    </div>
  );

  return (
    <div className="space-y-8 p-6">
      {/* Default Paper */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Default Paper</h2>
        <Paper>{content}</Paper>
        <p className="mt-2 text-sm text-nest-muted">elevation={1} (default), variant="elevation"</p>
      </section>

      {/* Elevation Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Elevation Variations</h2>
        <div className="grid grid-cols-2 gap-4 md:grid-cols-5">
          <Paper elevation={0} className="p-4 text-center">
            <p className="font-medium">elevation={0}</p>
            <p className="text-xs text-nest-muted">No shadow</p>
          </Paper>
          <Paper elevation={1} className="p-4 text-center">
            <p className="font-medium">elevation={1}</p>
            <p className="text-xs text-nest-muted">Default</p>
          </Paper>
          <Paper elevation={2} className="p-4 text-center">
            <p className="font-medium">elevation={2}</p>
            <p className="text-xs text-nest-muted">Small</p>
          </Paper>
          <Paper elevation={3} className="p-4 text-center">
            <p className="font-medium">elevation={3}</p>
            <p className="text-xs text-nest-muted">Medium</p>
          </Paper>
          <Paper elevation={4} className="p-4 text-center">
            <p className="font-medium">elevation={4}</p>
            <p className="text-xs text-nest-muted">Large</p>
          </Paper>
        </div>
      </section>

      {/* Variant Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variant Comparison</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">variant="elevation" (default)</p>
            <Paper elevation={2} className="p-4">
              <p>Shadow-based elevation</p>
            </Paper>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">variant="outlined"</p>
            <Paper variant="outlined" className="p-4">
              <p>Border-based outline</p>
            </Paper>
          </div>
        </div>
      </section>

      {/* Square Corners */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Square Corners</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">square={false} (default)</p>
            <Paper elevation={2} className="p-4">
              <p>Rounded corners (rounded-nest-md)</p>
            </Paper>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">square={true}</p>
            <Paper square elevation={2} className="p-4">
              <p>Square corners (rounded-none)</p>
            </Paper>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">square={true} + variant="outlined"</p>
            <Paper square variant="outlined" className="p-4">
              <p>Square outlined corners</p>
            </Paper>
          </div>
        </div>
      </section>

      {/* Custom Component */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <div className="space-y-4">
          <Paper component="article" elevation={2} className="p-4">
            <h3 className="mb-2 font-semibold">Article Paper</h3>
            <p className="text-sm text-nest-muted">Renders as &lt;article&gt;</p>
          </Paper>
          <Paper component="section" elevation={2} className="p-4">
            <h3 className="mb-2 font-semibold">Section Paper</h3>
            <p className="text-sm text-nest-muted">Renders as &lt;section&gt;</p>
          </Paper>
          <Paper component="aside" elevation={2} className="p-4">
            <h3 className="mb-2 font-semibold">Aside Paper</h3>
            <p className="text-sm text-nest-muted">Renders as &lt;aside&gt;</p>
          </Paper>
        </div>
      </section>

      {/* Interactive Paper */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Interactive Paper</h2>
        <div className="space-y-4">
          <Paper
            component="button"
            elevation={2}
            className="w-full cursor-pointer p-4 text-left hover:bg-nest-primary/5 transition-colors"
            onClick={() => alert('Paper clicked!')}
          >
            <h3 className="font-semibold">Clickable Paper</h3>
            <p className="text-sm text-nest-muted">Paper as a button</p>
          </Paper>
          <Paper
            component="a"
            href="#"
            elevation={2}
            className="block p-4 hover:bg-nest-primary/5 transition-colors"
          >
            <h3 className="font-semibold text-nest-primary">Link Paper</h3>
            <p className="text-sm text-nest-muted">Paper as an anchor link</p>
          </Paper>
        </div>
      </section>

      {/* Nested Papers */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Nested Papers</h2>
        <Paper elevation={2} className="p-6">
          <h3 className="mb-4 font-semibold">Outer Paper</h3>
          <Paper elevation={1} className="p-4">
            <h4 className="mb-2 font-medium">Inner Paper</h4>
            <p className="text-sm text-nest-muted">
              Nested paper with lower elevation for visual hierarchy
            </p>
          </Paper>
        </Paper>
      </section>

      {/* Card-like Pattern */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card-like Pattern</h2>
        <div className="grid gap-4 md:grid-cols-3">
          <Paper elevation={2}>
            <div className="border-b border-nest-border bg-nest-muted/10 p-4">
              <h3 className="font-semibold">Card Header</h3>
            </div>
            <div className="p-4">
              <p className="text-sm text-nest-muted">Card body content</p>
            </div>
            <div className="border-t border-nest-border bg-nest-muted/10 p-4">
              <p className="text-xs text-nest-muted">Card footer</p>
            </div>
          </Paper>
          <Paper elevation={2}>
            <div className="border-b border-nest-border bg-nest-muted/10 p-4">
              <h3 className="font-semibold">Card Header</h3>
            </div>
            <div className="p-4">
              <p className="text-sm text-nest-muted">Card body content</p>
            </div>
            <div className="border-t border-nest-border bg-nest-muted/10 p-4">
              <p className="text-xs text-nest-muted">Card footer</p>
            </div>
          </Paper>
          <Paper elevation={2}>
            <div className="border-b border-nest-border bg-nest-muted/10 p-4">
              <h3 className="font-semibold">Card Header</h3>
            </div>
            <div className="p-4">
              <p className="text-sm text-nest-muted">Card body content</p>
            </div>
            <div className="border-t border-nest-border bg-nest-muted/10 p-4">
              <p className="text-xs text-nest-muted">Card footer</p>
            </div>
          </Paper>
        </div>
      </section>

      {/* Form Container */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Form Container</h2>
        <Paper elevation={2} className="mx-auto max-w-md p-6">
          <h3 className="mb-4 text-lg font-semibold">Login Form</h3>
          <div className="space-y-4">
            <div>
              <label className="mb-1 block text-sm font-medium">Email</label>
              <input
                type="email"
                className="w-full rounded-nest-md border border-nest-border bg-nest-background p-2 text-sm"
                placeholder="you@example.com"
              />
            </div>
            <div>
              <label className="mb-1 block text-sm font-medium">Password</label>
              <input
                type="password"
                className="w-full rounded-nest-md border border-nest-border bg-nest-background p-2 text-sm"
                placeholder="••••••••"
              />
            </div>
            <button className="w-full rounded-nest-md bg-nest-primary px-4 py-2 text-sm font-medium text-white hover:bg-nest-primary/90">
              Sign In
            </button>
          </div>
        </Paper>
      </section>
    </div>
  );
}
