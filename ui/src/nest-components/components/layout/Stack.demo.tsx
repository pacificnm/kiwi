import { Stack } from './Stack';

/**
 * Stack Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function StackDemos() {
  const demoBox = 'h-12 w-32 rounded-nest-md bg-nest-primary/20 flex items-center justify-center text-nest-primary font-medium';

  return (
    <div className="space-y-8 p-6">
      {/* Default Column Stack */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Default Column Stack</h2>
        <Stack spacing={2} className="border border-nest-border p-4">
          <div className={demoBox}>Item 1</div>
          <div className={demoBox}>Item 2</div>
          <div className={demoBox}>Item 3</div>
        </Stack>
        <p className="mt-2 text-sm text-nest-muted">Default: direction="column", spacing=2</p>
      </section>

      {/* Row Direction */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Row Direction</h2>
        <Stack direction="row" spacing={2} className="border border-nest-border p-4">
          <div className={demoBox}>Item 1</div>
          <div className={demoBox}>Item 2</div>
          <div className={demoBox}>Item 3</div>
        </Stack>
        <p className="mt-2 text-sm text-nest-muted">direction="row"</p>
      </section>

      {/* Spacing Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Spacing Variations</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={0}</p>
            <Stack direction="row" spacing={0} className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={2} (default)</p>
            <Stack direction="row" spacing={2} className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={4}</p>
            <Stack direction="row" spacing={4} className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">spacing={8}</p>
            <Stack direction="row" spacing={8} className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
        </div>
      </section>

      {/* Alignment */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Alignment (Cross Axis)</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">align="start" (default)</p>
            <Stack direction="row" spacing={2} align="start" className="border border-nest-border p-2 h-24">
              <div className={demoBox}>Short</div>
              <div className={`${demoBox} h-16`}>Medium</div>
              <div className={`${demoBox} h-20`}>Tall</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">align="center"</p>
            <Stack direction="row" spacing={2} align="center" className="border border-nest-border p-2 h-24">
              <div className={demoBox}>Short</div>
              <div className={`${demoBox} h-16`}>Medium</div>
              <div className={`${demoBox} h-20`}>Tall</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">align="end"</p>
            <Stack direction="row" spacing={2} align="end" className="border border-nest-border p-2 h-24">
              <div className={demoBox}>Short</div>
              <div className={`${demoBox} h-16`}>Medium</div>
              <div className={`${demoBox} h-20`}>Tall</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">align="stretch"</p>
            <Stack direction="row" spacing={2} align="stretch" className="border border-nest-border p-2 h-24">
              <div className={demoBox}>Short</div>
              <div className={`${demoBox} h-16`}>Medium</div>
              <div className={`${demoBox} h-20`}>Tall</div>
            </Stack>
          </div>
        </div>
      </section>

      {/* Justify Content */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Justify Content (Main Axis)</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">justify="start" (default)</p>
            <Stack direction="row" spacing={2} justify="start" className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">justify="center"</p>
            <Stack direction="row" spacing={2} justify="center" className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">justify="end"</p>
            <Stack direction="row" spacing={2} justify="end" className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">justify="between"</p>
            <Stack direction="row" spacing={2} justify="between" className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">justify="around"</p>
            <Stack direction="row" spacing={2} justify="around" className="border border-nest-border p-2">
              <div className={demoBox}>1</div>
              <div className={demoBox}>2</div>
              <div className={demoBox}>3</div>
            </Stack>
          </div>
        </div>
      </section>

      {/* Wrap */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Wrap</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">wrap={false} (default)</p>
            <Stack direction="row" spacing={2} wrap={false} className="border border-nest-border p-2 w-48">
              <div className={`${demoBox} w-16`}>1</div>
              <div className={`${demoBox} w-16`}>2</div>
              <div className={`${demoBox} w-16`}>3</div>
              <div className={`${demoBox} w-16`}>4</div>
            </Stack>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">wrap={true}</p>
            <Stack direction="row" spacing={2} wrap className="border border-nest-border p-2 w-48">
              <div className={`${demoBox} w-16`}>1</div>
              <div className={`${demoBox} w-16`}>2</div>
              <div className={`${demoBox} w-16`}>3</div>
              <div className={`${demoBox} w-16`}>4</div>
            </Stack>
          </div>
        </div>
      </section>

      {/* Custom Component */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <Stack component="nav" spacing={2} className="border border-nest-border p-4">
          <a href="#" className="text-nest-primary underline">Link 1</a>
          <a href="#" className="text-nest-primary underline">Link 2</a>
          <a href="#" className="text-nest-primary underline">Link 3</a>
        </Stack>
        <p className="mt-2 text-sm text-nest-muted">Renders as &lt;nav&gt; element</p>
      </section>

      {/* Nested Stacks */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Nested Stacks</h2>
        <Stack spacing={4} className="border border-nest-border p-4">
          <Stack direction="row" spacing={2} align="center">
            <div className={demoBox}>Header</div>
            <div className="text-nest-muted">Subtitle</div>
          </Stack>
          <div className="h-24 rounded-nest-md bg-nest-surface border border-nest-border" />
          <Stack direction="row" spacing={2} justify="end">
            <button className="rounded-nest-md border border-nest-border px-4 py-2 text-sm">Cancel</button>
            <button className="rounded-nest-md bg-nest-primary px-4 py-2 text-sm text-white">Submit</button>
          </Stack>
        </Stack>
      </section>
    </div>
  );
}
