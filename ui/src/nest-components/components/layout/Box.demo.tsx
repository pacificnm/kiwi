import { Box } from './Box';

/**
 * Box Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function BoxDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Default Usage */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Default Usage</h2>
        <Box className="border border-nest-border p-4">
          Basic box with Tailwind classes for border and padding
        </Box>
      </section>

      {/* Custom Elements */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Elements</h2>
        <div className="space-y-2">
          <Box component="section" className="border border-nest-border p-2">
            Renders as &lt;section&gt;
          </Box>
          <Box component="article" className="border border-nest-border p-2">
            Renders as &lt;article&gt;
          </Box>
          <Box component="aside" className="border border-nest-border p-2">
            Renders as &lt;aside&gt;
          </Box>
          <Box component="nav" className="border border-nest-border p-2">
            Renders as &lt;nav&gt;
          </Box>
          <Box component="header" className="border border-nest-border p-2">
            Renders as &lt;header&gt;
          </Box>
          <Box component="footer" className="border border-nest-border p-2">
            Renders as &lt;footer&gt;
          </Box>
          <Box component="span" className="text-nest-primary">
            Renders as inline &lt;span&gt;
          </Box>
        </div>
      </section>

      {/* Layout Examples */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Layout Patterns</h2>

        {/* Flex Container */}
        <div className="mb-4">
          <h3 className="mb-2 text-sm font-medium text-nest-muted">Flex Container</h3>
          <Box className="flex items-center gap-4 border border-nest-border p-4">
            <Box className="h-10 w-10 bg-nest-primary rounded-nest-sm" />
            <Box className="h-10 w-10 bg-nest-secondary rounded-nest-sm" />
            <Box className="h-10 w-10 bg-nest-accent rounded-nest-sm" />
          </Box>
        </div>

        {/* Grid Container */}
        <div className="mb-4">
          <h3 className="mb-2 text-sm font-medium text-nest-muted">Grid Container</h3>
          <Box className="grid grid-cols-3 gap-4 border border-nest-border p-4">
            <Box className="h-16 bg-nest-primary/20 rounded-nest-sm" />
            <Box className="h-16 bg-nest-secondary/20 rounded-nest-sm" />
            <Box className="h-16 bg-nest-accent/20 rounded-nest-sm" />
          </Box>
        </div>

        {/* Stack Pattern */}
        <Box className="flex flex-col gap-2 border border-nest-border p-4">
          <Box className="h-8 bg-nest-primary/20 rounded-nest-sm" />
          <Box className="h-8 bg-nest-secondary/20 rounded-nest-sm" />
          <Box className="h-8 bg-nest-accent/20 rounded-nest-sm" />
        </Box>
      </section>

      {/* With Accessibility Props */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Accessibility Props</h2>
        <Box
          component="button"
          className="rounded-nest-md bg-nest-primary px-4 py-2 text-white hover:bg-nest-primary/90"
          onClick={() => alert('Box button clicked!')}
        >
          Box as Button
        </Box>
        <Box
          component="a"
          href="https://example.com"
          className="text-nest-primary underline"
          target="_blank"
          rel="noopener noreferrer"
        >
          Box as Link
        </Box>
      </section>

      {/* Nested Boxes */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Nested Boxes</h2>
        <Box className="border border-nest-border p-4">
          <Box className="border border-nest-border p-4">
            <Box className="border border-nest-border p-4">
              <Box className="border border-nest-border p-4">
                Deeply nested content
              </Box>
            </Box>
          </Box>
        </Box>
      </section>

      {/* Escape Hatch Pattern */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Escape Hatch Pattern</h2>
        <p className="mb-2 text-sm text-nest-muted">
          Use Box when you need a custom element that other components don't support:
        </p>
        <Box
          component="time"
          dateTime="2024-01-15"
          className="text-xs text-nest-muted"
        >
          January 15, 2024
        </Box>
      </section>
    </div>
  );
}
