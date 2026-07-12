import { Grid } from './Grid';

/**
 * Grid Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function GridDemos() {
  const itemClass = 'flex items-center justify-center rounded-nest-md bg-nest-primary/20 p-4 text-nest-primary font-medium';

  return (
    <div className="space-y-8 p-6">
      {/* Basic Grid Container */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Grid Container</h2>
        <Grid container spacing={2} className="border border-nest-border p-2">
          <Grid size={12}>
            <div className={itemClass}>Full width (12/12)</div>
          </Grid>
          <Grid size={6}>
            <div className={itemClass}>Half (6/12)</div>
          </Grid>
          <Grid size={6}>
            <div className={itemClass}>Half (6/12)</div>
          </Grid>
          <Grid size={4}>
            <div className={itemClass}>Third (4/12)</div>
          </Grid>
          <Grid size={4}>
            <div className={itemClass}>Third (4/12)</div>
          </Grid>
          <Grid size={4}>
            <div className={itemClass}>Third (4/12)</div>
          </Grid>
          <Grid size={3}>
            <div className={itemClass}>Quarter (3/12)</div>
          </Grid>
          <Grid size={3}>
            <div className={itemClass}>Quarter (3/12)</div>
          </Grid>
          <Grid size={3}>
            <div className={itemClass}>Quarter (3/12)</div>
          </Grid>
          <Grid size={3}>
            <div className={itemClass}>Quarter (3/12)</div>
          </Grid>
        </Grid>
      </section>

      {/* Spacing Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Spacing Variations</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={0}</p>
            <Grid container spacing={0} className="border border-nest-border">
              <Grid size={4}><div className={itemClass}>1</div></Grid>
              <Grid size={4}><div className={itemClass}>2</div></Grid>
              <Grid size={4}><div className={itemClass}>3</div></Grid>
            </Grid>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={2}</p>
            <Grid container spacing={2} className="border border-nest-border">
              <Grid size={4}><div className={itemClass}>1</div></Grid>
              <Grid size={4}><div className={itemClass}>2</div></Grid>
              <Grid size={4}><div className={itemClass}>3</div></Grid>
            </Grid>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={4}</p>
            <Grid container spacing={4} className="border border-nest-border">
              <Grid size={4}><div className={itemClass}>1</div></Grid>
              <Grid size={4}><div className={itemClass}>2</div></Grid>
              <Grid size={4}><div className={itemClass}>3</div></Grid>
            </Grid>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={8}</p>
            <Grid container spacing={8} className="border border-nest-border">
              <Grid size={4}><div className={itemClass}>1</div></Grid>
              <Grid size={4}><div className={itemClass}>2</div></Grid>
              <Grid size={4}><div className={itemClass}>3</div></Grid>
            </Grid>
          </div>
        </div>
      </section>

      {/* Responsive Sizing */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Responsive Sizing</h2>
        <Grid container spacing={2} className="border border-nest-border p-2">
          <Grid size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <div className={itemClass}>xs=12, sm=6, md=4, lg=3</div>
          </Grid>
          <Grid size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <div className={itemClass}>xs=12, sm=6, md=4, lg=3</div>
          </Grid>
          <Grid size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <div className={itemClass}>xs=12, sm=6, md=4, lg=3</div>
          </Grid>
          <Grid size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <div className={itemClass}>xs=12, sm=6, md=4, lg=3</div>
          </Grid>
        </Grid>
        <p className="mt-2 text-sm text-nest-muted">
          Resize window to see responsive behavior: full width on mobile, 2 per row on tablet, 3 per row on desktop, 4 per row on large screens.
        </p>
      </section>

      {/* Offset */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Offset</h2>
        <Grid container spacing={2} className="border border-nest-border p-2">
          <Grid size={3}>
            <div className={itemClass}>3 cols</div>
          </Grid>
          <Grid size={6} offset={3}>
            <div className={itemClass}>6 cols with offset 3</div>
          </Grid>
        </Grid>
        <Grid container spacing={2} className="mt-2 border border-nest-border p-2">
          <Grid size={4}>
            <div className={itemClass}>4 cols</div>
          </Grid>
          <Grid size={4} offset={4}>
            <div className={itemClass}>4 cols with offset 4 (centered)</div>
          </Grid>
        </Grid>
      </section>

      {/* Auto Size */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Auto Size</h2>
        <Grid container spacing={2} className="border border-nest-border p-2">
          <Grid size="auto">
            <div className={itemClass}>Auto width</div>
          </Grid>
          <Grid size={6}>
            <div className={itemClass}>6 cols</div>
          </Grid>
          <Grid size="auto">
            <div className={itemClass}>Auto width</div>
          </Grid>
        </Grid>
      </section>

      {/* Custom Columns */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Columns</h2>
        <Grid container spacing={2} columns={6} className="border border-nest-border p-2">
          <Grid size={2}>
            <div className={itemClass}>2/6</div>
          </Grid>
          <Grid size={2}>
            <div className={itemClass}>2/6</div>
          </Grid>
          <Grid size={2}>
            <div className={itemClass}>2/6</div>
          </Grid>
        </Grid>
        <p className="mt-2 text-sm text-nest-muted">Grid with 6 columns instead of default 12</p>
      </section>

      {/* Complex Layout */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Complex Layout</h2>
        <Grid container spacing={3} className="border border-nest-border p-3">
          {/* Header */}
          <Grid size={12}>
            <div className="rounded-nest-md bg-nest-primary p-4 text-white">
              <h3 className="text-lg font-semibold">Header</h3>
            </div>
          </Grid>

          {/* Sidebar */}
          <Grid size={{ xs: 12, md: 3 }}>
            <div className="rounded-nest-md bg-nest-surface p-4 border border-nest-border">
              <h4 className="mb-2 font-medium">Sidebar</h4>
              <ul className="space-y-1 text-sm text-nest-muted">
                <li>Link 1</li>
                <li>Link 2</li>
                <li>Link 3</li>
              </ul>
            </div>
          </Grid>

          {/* Main Content */}
          <Grid size={{ xs: 12, md: 9 }}>
            <Grid container spacing={2}>
              <Grid size={{ xs: 12, lg: 6 }}>
                <div className="h-32 rounded-nest-md bg-nest-secondary/20 p-4">Card 1</div>
              </Grid>
              <Grid size={{ xs: 12, lg: 6 }}>
                <div className="h-32 rounded-nest-md bg-nest-accent/20 p-4">Card 2</div>
              </Grid>
              <Grid size={12}>
                <div className="h-32 rounded-nest-md bg-nest-success/20 p-4">Full Width Card</div>
              </Grid>
            </Grid>
          </Grid>

          {/* Footer */}
          <Grid size={12}>
            <div className="rounded-nest-md bg-nest-muted/20 p-4 text-center text-sm text-nest-muted">
              Footer
            </div>
          </Grid>
        </Grid>
      </section>
    </div>
  );
}
