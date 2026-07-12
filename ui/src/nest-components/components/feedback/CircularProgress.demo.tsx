import { CircularProgress } from './CircularProgress';

/**
 * CircularProgress Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function CircularProgressDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Indeterminate Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Indeterminate Progress</h2>
        <div className="flex items-center gap-6">
          <CircularProgress />
          <CircularProgress size="small" />
          <CircularProgress size="medium" />
          <CircularProgress size="large" />
        </div>
        <p className="mt-2 text-sm text-nest-muted">small, medium (default), large</p>
      </section>

      {/* Determinate Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Determinate Progress</h2>
        <div className="flex items-center gap-6">
          <div className="flex flex-col items-center gap-2">
            <CircularProgress variant="determinate" value={0} />
            <span className="text-xs text-nest-muted">0%</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress variant="determinate" value={25} />
            <span className="text-xs text-nest-muted">25%</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress variant="determinate" value={50} />
            <span className="text-xs text-nest-muted">50%</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress variant="determinate" value={75} />
            <span className="text-xs text-nest-muted">75%</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress variant="determinate" value={100} />
            <span className="text-xs text-nest-muted">100%</span>
          </div>
        </div>
      </section>

      {/* Color Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Color Variants</h2>
        <div className="flex flex-wrap items-center gap-6">
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="primary" />
            <span className="text-xs text-nest-muted">Primary</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="secondary" />
            <span className="text-xs text-nest-muted">Secondary</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="accent" />
            <span className="text-xs text-nest-muted">Accent</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="success" />
            <span className="text-xs text-nest-muted">Success</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="warning" />
            <span className="text-xs text-nest-muted">Warning</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="error" />
            <span className="text-xs text-nest-muted">Error</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress color="info" />
            <span className="text-xs text-nest-muted">Info</span>
          </div>
        </div>
      </section>

      {/* Size Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Size Comparison</h2>
        <div className="flex items-center gap-6">
          <div className="flex flex-col items-center gap-2">
            <CircularProgress size="small" />
            <span className="text-xs text-nest-muted">Small (16px)</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress size="medium" />
            <span className="text-xs text-nest-muted">Medium (32px)</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress size="large" />
            <span className="text-xs text-nest-muted">Large (48px)</span>
          </div>
          <div className="flex flex-col items-center gap-2">
            <CircularProgress size={64} />
            <span className="text-xs text-nest-muted">Custom (64px)</span>
          </div>
        </div>
      </section>

      {/* All Colors Determinate */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Determinate Colors</h2>
        <div className="flex flex-wrap items-center gap-6">
          <CircularProgress variant="determinate" value={75} color="primary" />
          <CircularProgress variant="determinate" value={75} color="secondary" />
          <CircularProgress variant="determinate" value={75} color="success" />
          <CircularProgress variant="determinate" value={75} color="warning" />
          <CircularProgress variant="determinate" value={75} color="error" />
        </div>
      </section>

      {/* Inherit Color */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Inherit Color</h2>
        <div className="space-y-4">
          <div className="text-nest-primary">
            <span className="mr-4">Primary text:</span>
            <CircularProgress color="inherit" size="small" />
          </div>
          <div className="text-nest-success">
            <span className="mr-4">Success text:</span>
            <CircularProgress color="inherit" size="small" />
          </div>
          <div className="text-nest-error">
            <span className="mr-4">Error text:</span>
            <CircularProgress color="inherit" size="small" />
          </div>
        </div>
      </section>

      {/* Loading States */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Loading States</h2>
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-3">
            <CircularProgress size="small" />
            <span>Loading...</span>
          </div>
          <div className="flex items-center gap-3">
            <CircularProgress size="small" color="success" />
            <span>Saving...</span>
          </div>
          <div className="flex items-center gap-3">
            <CircularProgress size="small" color="error" />
            <span>Error occurred</span>
          </div>
        </div>
      </section>

      {/* Centered Loading */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Centered Loading</h2>
        <div className="flex h-32 items-center justify-center border border-nest-border rounded-nest-md">
          <CircularProgress size="large" />
        </div>
        <p className="mt-2 text-sm text-nest-muted">Centered in a container</p>
      </section>

      {/* With Label */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Label</h2>
        <div className="flex items-center gap-6">
          <div className="relative flex items-center justify-center">
            <CircularProgress variant="determinate" value={60} size="large" />
            <span className="absolute text-xs font-medium text-nest-foreground">60%</span>
          </div>
          <div className="relative flex items-center justify-center">
            <CircularProgress variant="determinate" value={80} size="large" color="success" />
            <span className="absolute text-xs font-medium text-nest-foreground">80%</span>
          </div>
        </div>
      </section>
    </div>
  );
}
