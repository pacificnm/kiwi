import { Divider } from './Divider';

/**
 * Divider Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function DividerDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Horizontal Divider */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Horizontal Divider</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">Default (fullWidth)</p>
            <Divider className="my-2" />
            <p>Content below divider</p>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">fullWidth={false}</p>
            <Divider fullWidth={false} className="my-2 w-32" />
            <p>Short divider</p>
          </div>
        </div>
      </section>

      {/* Vertical Divider */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Vertical Divider</h2>
        <div className="flex items-center gap-4 border border-nest-border p-4">
          <span>Left content</span>
          <Divider orientation="vertical" className="h-12" />
          <span>Right content</span>
          <Divider orientation="vertical" flexItem className="h-12" />
          <span>Flex item</span>
        </div>
      </section>

      {/* Text Dividers */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Text Dividers</h2>
        <div className="space-y-4">
          <Divider>OR</Divider>
          <Divider>Continue with</Divider>
          <Divider>Section 1</Divider>
        </div>
      </section>

      {/* Icon Dividers */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Icon Dividers</h2>
        <div className="space-y-4">
          <Divider>★</Divider>
          <Divider>✦</Divider>
          <Divider>◆</Divider>
        </div>
      </section>

      {/* Form Separator */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Form Separator</h2>
        <div className="max-w-sm space-y-4 border border-nest-border p-4">
          <div>
            <label className="mb-1 block text-sm font-medium">Email</label>
            <input
              type="email"
              className="w-full rounded-nest-md border border-nest-border p-2 text-sm"
              placeholder="you@example.com"
            />
          </div>
          <Divider>OR</Divider>
          <div>
            <label className="mb-1 block text-sm font-medium">Phone</label>
            <input
              type="tel"
              className="w-full rounded-nest-md border border-nest-border p-2 text-sm"
              placeholder="+1 (555) 000-0000"
            />
          </div>
        </div>
      </section>

      {/* Content Sections */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Content Sections</h2>
        <div className="space-y-4">
          <div>
            <h3 className="mb-2 font-medium">Section 1</h3>
            <p className="text-sm text-nest-muted">Content for section 1</p>
          </div>
          <Divider>Next</Divider>
          <div>
            <h3 className="mb-2 font-medium">Section 2</h3>
            <p className="text-sm text-nest-muted">Content for section 2</p>
          </div>
          <Divider>Finally</Divider>
          <div>
            <h3 className="mb-2 font-medium">Section 3</h3>
            <p className="text-sm text-nest-muted">Content for section 3</p>
          </div>
        </div>
      </section>

      {/* Card Divider */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Card Divider</h2>
        <div className="max-w-md border border-nest-border rounded-nest-md">
          <div className="p-4">
            <h3 className="font-semibold">Card Header</h3>
            <p className="text-sm text-nest-muted">Some header content</p>
          </div>
          <Divider />
          <div className="p-4">
            <p className="text-sm">Card body content goes here</p>
          </div>
          <Divider />
          <div className="p-4 bg-nest-muted/10">
            <p className="text-xs text-nest-muted">Card footer</p>
          </div>
        </div>
      </section>

      {/* Toolbar Divider */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Toolbar Divider</h2>
        <div className="flex items-center gap-2 border border-nest-border p-2">
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Bold
          </button>
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Italic
          </button>
          <Divider orientation="vertical" className="h-6" />
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Underline
          </button>
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Strike
          </button>
          <Divider orientation="vertical" className="h-6" />
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Align Left
          </button>
          <button className="rounded-nest-md px-3 py-1.5 text-sm hover:bg-nest-surface">
            Align Center
          </button>
        </div>
      </section>

      {/* Custom Component */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Component</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">component="hr"</p>
            <Divider component="hr" className="my-2" />
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">component="li" (in a list)</p>
            <ul className="space-y-2">
              <li>List item 1</li>
              <Divider component="li" />
              <li>List item 2</li>
              <Divider component="li" />
              <li>List item 3</li>
            </ul>
          </div>
        </div>
      </section>

      {/* Responsive Layout */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Responsive Layout</h2>
        <div className="flex flex-col gap-4 md:flex-row md:items-center">
          <div className="flex-1 rounded-nest-md bg-nest-primary/20 p-4">
            <p className="font-medium">Column 1</p>
          </div>
          <Divider orientation="vertical" className="hidden h-16 md:block" />
          <Divider className="md:hidden" />
          <div className="flex-1 rounded-nest-md bg-nest-secondary/20 p-4">
            <p className="font-medium">Column 2</p>
          </div>
          <Divider orientation="vertical" className="hidden h-16 md:block" />
          <Divider className="md:hidden" />
          <div className="flex-1 rounded-nest-md bg-nest-accent/20 p-4">
            <p className="font-medium">Column 3</p>
          </div>
        </div>
      </section>
    </div>
  );
}
