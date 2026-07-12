import { LinearProgress } from './LinearProgress';

/**
 * LinearProgress Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function LinearProgressDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Indeterminate Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Indeterminate Progress</h2>
        <div className="space-y-4">
          <LinearProgress />
          <p className="text-sm text-nest-muted">Default indeterminate loading</p>
        </div>
      </section>

      {/* Determinate Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Determinate Progress</h2>
        <div className="space-y-4">
          <div>
            <div className="mb-1 text-xs text-nest-muted">0%</div>
            <LinearProgress variant="determinate" value={0} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">25%</div>
            <LinearProgress variant="determinate" value={25} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">50%</div>
            <LinearProgress variant="determinate" value={50} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">75%</div>
            <LinearProgress variant="determinate" value={75} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">100%</div>
            <LinearProgress variant="determinate" value={100} />
          </div>
        </div>
      </section>

      {/* Buffer Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Buffer Progress</h2>
        <div className="space-y-4">
          <div>
            <div className="mb-1 text-xs text-nest-muted">Buffer: 50% / Value: 25%</div>
            <LinearProgress variant="buffer" value={25} bufferValue={50} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Buffer: 75% / Value: 50%</div>
            <LinearProgress variant="buffer" value={50} bufferValue={75} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Buffer: 100% / Value: 75%</div>
            <LinearProgress variant="buffer" value={75} bufferValue={100} />
          </div>
        </div>
      </section>

      {/* Color Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Color Variants</h2>
        <div className="space-y-4">
          <div>
            <div className="mb-1 text-xs text-nest-muted">Primary</div>
            <LinearProgress color="primary" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Secondary</div>
            <LinearProgress color="secondary" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Accent</div>
            <LinearProgress color="accent" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Success</div>
            <LinearProgress color="success" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Warning</div>
            <LinearProgress color="warning" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Error</div>
            <LinearProgress color="error" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Info</div>
            <LinearProgress color="info" />
          </div>
        </div>
      </section>

      {/* Determinate Colors */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Determinate Colors</h2>
        <div className="space-y-4">
          <LinearProgress variant="determinate" value={75} color="primary" />
          <LinearProgress variant="determinate" value={75} color="secondary" />
          <LinearProgress variant="determinate" value={75} color="success" />
          <LinearProgress variant="determinate" value={75} color="warning" />
          <LinearProgress variant="determinate" value={75} color="error" />
        </div>
      </section>

      {/* Inherit Color */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Inherit Color</h2>
        <div className="space-y-4">
          <div className="text-nest-primary">
            <div className="mb-1 text-xs">Primary text context:</div>
            <LinearProgress color="inherit" />
          </div>
          <div className="text-nest-success">
            <div className="mb-1 text-xs">Success text context:</div>
            <LinearProgress color="inherit" />
          </div>
          <div className="text-nest-error">
            <div className="mb-1 text-xs">Error text context:</div>
            <LinearProgress color="inherit" />
          </div>
        </div>
      </section>

      {/* File Upload Example */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">File Upload Example</h2>
        <div className="rounded-nest-md border border-nest-border p-4">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-sm font-medium">Uploading file.txt</span>
            <span className="text-sm text-nest-muted">65%</span>
          </div>
          <LinearProgress variant="determinate" value={65} />
        </div>
      </section>

      {/* Multiple Downloads */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Multiple Downloads</h2>
        <div className="space-y-3">
          <div>
            <div className="mb-1 flex justify-between text-xs">
              <span>Document.pdf</span>
              <span className="text-nest-muted">100%</span>
            </div>
            <LinearProgress variant="determinate" value={100} color="success" />
          </div>
          <div>
            <div className="mb-1 flex justify-between text-xs">
              <span>Image.png</span>
              <span className="text-nest-muted">75%</span>
            </div>
            <LinearProgress variant="determinate" value={75} />
          </div>
          <div>
            <div className="mb-1 flex justify-between text-xs">
              <span>Video.mp4</span>
              <span className="text-nest-muted">30%</span>
            </div>
            <LinearProgress variant="determinate" value={30} />
          </div>
        </div>
      </section>

      {/* Loading States */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Loading States</h2>
        <div className="space-y-4">
          <div>
            <div className="mb-1 text-xs text-nest-muted">Initial load</div>
            <LinearProgress />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Processing...</div>
            <LinearProgress variant="determinate" value={45} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Complete</div>
            <LinearProgress variant="determinate" value={100} color="success" />
          </div>
        </div>
      </section>

      {/* Stacked Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Stacked Progress</h2>
        <div className="space-y-2">
          <LinearProgress variant="determinate" value={100} color="success" />
          <LinearProgress variant="determinate" value={66} color="warning" />
          <LinearProgress variant="determinate" value={33} color="error" />
        </div>
      </section>

      {/* Thin Progress */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Height</h2>
        <div className="space-y-4">
          <div>
            <div className="mb-1 text-xs text-nest-muted">Default (h-1)</div>
            <LinearProgress variant="determinate" value={50} />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Custom (h-2)</div>
            <LinearProgress variant="determinate" value={50} className="h-2" />
          </div>
          <div>
            <div className="mb-1 text-xs text-nest-muted">Custom (h-4)</div>
            <LinearProgress variant="determinate" value={50} className="h-4" />
          </div>
        </div>
      </section>
    </div>
  );
}
