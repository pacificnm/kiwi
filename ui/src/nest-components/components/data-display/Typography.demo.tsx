import { Typography } from './Typography';

/**
 * Typography Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function TypographyDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variants</h2>
        <div className="space-y-4">
          <Typography variant="h1">Heading 1 - Display Large</Typography>
          <Typography variant="h2">Heading 2 - Display Medium</Typography>
          <Typography variant="h3">Heading 3 - Section Title</Typography>
          <Typography variant="h4">Heading 4 - Subsection Title</Typography>
          <Typography variant="h5">Heading 5 - Card Title</Typography>
          <Typography variant="h6">Heading 6 - Group Title</Typography>
          <Typography variant="subtitle1">Subtitle 1 - Secondary heading with muted color</Typography>
          <Typography variant="subtitle2">Subtitle 2 - Smaller secondary heading</Typography>
          <Typography variant="body1">
            Body 1 - Standard body text. This is the default variant used for paragraphs and general content.
          </Typography>
          <Typography variant="body2">
            Body 2 - Smaller body text for secondary content and descriptions.
          </Typography>
          <Typography variant="caption">Caption - Small text for annotations and captions.</Typography>
          <Typography variant="overline">Overline - Uppercase small text for labels</Typography>
        </div>
      </section>

      {/* Colors */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="space-y-2">
          <Typography color="foreground">Foreground - Default text color</Typography>
          <Typography color="primary">Primary - Primary brand color</Typography>
          <Typography color="secondary">Secondary - Secondary brand color</Typography>
          <Typography color="muted">Muted - Muted/subtle text</Typography>
          <Typography color="error">Error - Error state text</Typography>
          <Typography color="success">Success - Success state text</Typography>
          <Typography color="warning">Warning - Warning state text</Typography>
        </div>
      </section>

      {/* Alignment */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Alignment</h2>
        <div className="space-y-2">
          <Typography align="left">Left aligned text</Typography>
          <Typography align="center">Center aligned text</Typography>
          <Typography align="right">Right aligned text</Typography>
          <Typography align="justify">
            Justified text. This paragraph demonstrates text alignment that stretches lines to fill the container width.
          </Typography>
        </div>
      </section>

      {/* Gutter Bottom */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Gutter Bottom (Spacing)</h2>
        <Typography variant="h1" gutterBottom>Heading with bottom margin</Typography>
        <Typography variant="body1" gutterBottom>
          This paragraph has a bottom margin for spacing from the element below.
        </Typography>
        <Typography variant="body2">This paragraph has no bottom margin.</Typography>
      </section>

      {/* No Wrap (Truncation) */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">No Wrap (Truncation)</h2>
        <div className="max-w-xs border border-nest-border p-2">
          <Typography noWrap>
            This is a very long text that should truncate with an ellipsis when it overflows its container.
          </Typography>
          <Typography>Normal wrapping text for comparison.</Typography>
        </div>
      </section>

      {/* Component Override */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Component Override</h2>
        <Typography variant="h1" component="p">
          This renders as a &lt;p&gt; element but styled as h1
        </Typography>
        <Typography variant="body1" component="span" className="block">
          This renders as a &lt;span&gt; with display:block
        </Typography>
      </section>
    </div>
  );
}
