import { useState } from 'react';
import { Popover } from './Popover';
import { Button } from '../inputs/Button';

export function PopoverDemos() {
  const [open, setOpen] = useState(false);
  const [controlledOpen, setControlledOpen] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <Popover
          trigger={<Button variant="outlined">Click to open</Button>}
          open={open}
          onOpenChange={setOpen}
        >
          <div className="space-y-2">
            <h3 className="font-semibold">Popover Title</h3>
            <p className="text-sm text-nest-muted">
              This is the popover content. It can contain any React nodes.
            </p>
            <div className="flex gap-2 pt-2">
              <Button size="small" variant="outlined" onClick={() => setOpen(false)}>
                Cancel
              </Button>
              <Button size="small" onClick={() => setOpen(false)}>
                Confirm
              </Button>
            </div>
          </div>
        </Popover>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Controlled</h2>
        <div className="flex gap-4 items-center">
          <Popover
            trigger={<Button variant="outlined">Open popover</Button>}
            open={controlledOpen}
            onOpenChange={setControlledOpen}
          >
            <p>Controlled popover content</p>
          </Popover>
          <Button
            size="small"
            variant="text"
            onClick={() => setControlledOpen(!controlledOpen)}
          >
            {controlledOpen ? 'Close' : 'Open'} programmatically
          </Button>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Placements</h2>
        <div className="grid grid-cols-3 gap-4 place-items-center">
          <div />
          <Popover
            trigger={<Button size="small" variant="outlined">Top</Button>}
            placement="top"
          >
            <p>Top placement</p>
          </Popover>
          <div />

          <Popover
            trigger={<Button size="small" variant="outlined">Left</Button>}
            placement="left"
          >
            <p>Left placement</p>
          </Popover>
          <Popover
            trigger={<Button size="small" variant="outlined">Center</Button>}
            placement="bottom"
          >
            <p>Bottom placement (default)</p>
          </Popover>
          <Popover
            trigger={<Button size="small" variant="outlined">Right</Button>}
            placement="right"
          >
            <p>Right placement</p>
          </Popover>

          <div />
          <Popover
            trigger={<Button size="small" variant="outlined">Bottom</Button>}
            placement="bottom-start"
          >
            <p>Bottom-start placement</p>
          </Popover>
          <div />
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Without Outside Click Close</h2>
        <Popover
          trigger={<Button variant="outlined">Open (click outside won't close)</Button>}
          closeOnOutsideClick={false}
        >
          <p className="text-sm">
            This popover only closes when you click the trigger again or press Escape.
          </p>
        </Popover>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Without Escape Close</h2>
        <Popover
          trigger={<Button variant="outlined">Open (Escape won't close)</Button>}
          closeOnEscape={false}
        >
          <p className="text-sm">
            This popover doesn't close on Escape key press.
          </p>
        </Popover>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Rich Content</h2>
        <Popover
          trigger={<Button variant="outlined">Open rich popover</Button>}
        >
          <div className="space-y-3 min-w-[250px]">
            <div className="flex items-center gap-3">
              <div className="w-10 h-10 rounded-full bg-nest-primary flex items-center justify-center text-white font-bold">
                JD
              </div>
              <div>
                <p className="font-medium">John Doe</p>
                <p className="text-xs text-nest-muted">john@example.com</p>
              </div>
            </div>
            <div className="border-t border-nest-border pt-2 space-y-1">
              <button className="w-full text-left text-sm hover:bg-nest-muted px-2 py-1 rounded">
                View Profile
              </button>
              <button className="w-full text-left text-sm hover:bg-nest-muted px-2 py-1 rounded">
                Settings
              </button>
              <button className="w-full text-left text-sm hover:bg-nest-error/10 text-nest-error px-2 py-1 rounded">
                Sign Out
              </button>
            </div>
          </div>
        </Popover>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Form in Popover</h2>
        <Popover
          trigger={<Button variant="outlined">Quick Form</Button>}
        >
          <form className="space-y-3" onSubmit={(e) => e.preventDefault()}>
            <div>
              <label className="block text-sm font-medium mb-1">Name</label>
              <input
                type="text"
                className="w-full border border-nest-border rounded px-2 py-1 text-sm"
                placeholder="Enter name"
              />
            </div>
            <div>
              <label className="block text-sm font-medium mb-1">Email</label>
              <input
                type="email"
                className="w-full border border-nest-border rounded px-2 py-1 text-sm"
                placeholder="Enter email"
              />
            </div>
            <Button size="small" type="submit" className="w-full">
              Submit
            </Button>
          </form>
        </Popover>
      </section>
    </div>
  );
}
