import { useState } from 'react';
import { Backdrop } from './Backdrop';
import { Button } from '../inputs/Button';
import { Paper } from '../surface/Paper';

export function BackdropDemos() {
  const [open, setOpen] = useState(false);
  const [invisibleOpen, setInvisibleOpen] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Backdrop</h2>
        <div className="relative h-48 border border-nest-border rounded-nest-md overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-nest-primary/20 to-nest-secondary/20" />
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-nest-foreground">Background content</p>
          </div>
          <Backdrop open={open} />
          <div className="absolute bottom-4 left-4">
            <Button onClick={() => setOpen(!open)}>
              {open ? 'Hide Backdrop' : 'Show Backdrop'}
            </Button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Invisible Backdrop</h2>
        <p className="text-sm text-nest-muted mb-4">
          Invisible backdrops are useful for capturing clicks outside a modal
          without visually obscuring the background.
        </p>
        <div className="relative h-48 border border-nest-border rounded-nest-md overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-nest-primary/20 to-nest-secondary/20" />
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-nest-foreground">Background content (visible)</p>
          </div>
          <Backdrop open={invisibleOpen} invisible onClick={() => setInvisibleOpen(false)} />
          <div className="absolute bottom-4 left-4">
            <Button onClick={() => setInvisibleOpen(!invisibleOpen)}>
              {invisibleOpen ? 'Click backdrop to hide' : 'Show Invisible Backdrop'}
            </Button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Modal Content</h2>
        <div className="relative h-64 border border-nest-border rounded-nest-md overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-nest-primary/20 to-nest-secondary/20" />
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-nest-foreground">Background content</p>
          </div>
          <Backdrop open={open} onClick={() => setOpen(false)}>
            <div className="absolute inset-0 flex items-center justify-center p-4">
              <Paper className="max-w-sm w-full p-6">
                <h3 className="text-lg font-semibold mb-2">Modal Title</h3>
                <p className="text-nest-muted mb-4">
                  This is modal content displayed on top of the backdrop.
                </p>
                <div className="flex justify-end gap-2">
                  <Button variant="outlined" onClick={() => setOpen(false)}>Cancel</Button>
                  <Button onClick={() => setOpen(false)}>Confirm</Button>
                </div>
              </Paper>
            </div>
          </Backdrop>
          <div className="absolute bottom-4 left-4">
            <Button onClick={() => setOpen(true)}>Open Modal</Button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Loading State</h2>
        <p className="text-sm text-nest-muted mb-4">
          Backdrop can be used to show a loading state over content.
        </p>
        <div className="relative h-48 border border-nest-border rounded-nest-md overflow-hidden">
          <div className="absolute inset-0 p-4">
            <div className="space-y-2">
              <div className="h-4 bg-nest-muted rounded w-3/4" />
              <div className="h-4 bg-nest-muted rounded w-1/2" />
              <div className="h-4 bg-nest-muted rounded w-5/6" />
            </div>
          </div>
          <Backdrop open>
            <div className="absolute inset-0 flex items-center justify-center">
              <div className="flex items-center gap-2 text-nest-foreground">
                <div className="w-5 h-5 border-2 border-nest-primary border-t-transparent rounded-full animate-spin" />
                <span>Loading...</span>
              </div>
            </div>
          </Backdrop>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Custom Styling</h2>
        <div className="relative h-48 border border-nest-border rounded-nest-md overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-nest-primary/20 to-nest-secondary/20" />
          <div className="absolute inset-0 flex items-center justify-center">
            <p className="text-nest-foreground">Background content</p>
          </div>
          <Backdrop
            open
            className="bg-nest-primary/30 backdrop-blur-sm"
          >
            <div className="absolute inset-0 flex items-center justify-center">
              <p className="text-white text-lg font-medium">Custom styled backdrop</p>
            </div>
          </Backdrop>
        </div>
      </section>
    </div>
  );
}
