import { useState } from 'react';
import { Tooltip } from './Tooltip';
import { Button } from '../inputs/Button';

export function TooltipDemos() {
  const [open, setOpen] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Usage</h2>
        <div className="flex gap-4">
          <Tooltip title="Helpful information">
            <Button variant="outlined">Hover me</Button>
          </Tooltip>
          <Tooltip title="Another tooltip">
            <Button variant="outlined">Or me</Button>
          </Tooltip>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Placements</h2>
        <div className="grid grid-cols-3 gap-4 place-items-center">
          <div />
          <Tooltip title="Top" placement="top">
            <Button variant="outlined" size="small">Top</Button>
          </Tooltip>
          <div />

          <Tooltip title="Left" placement="left">
            <Button variant="outlined" size="small">Left</Button>
          </Tooltip>
          <Tooltip title="Center" placement="top">
            <Button variant="outlined" size="small">Center</Button>
          </Tooltip>
          <Tooltip title="Right" placement="right">
            <Button variant="outlined" size="small">Right</Button>
          </Tooltip>

          <div />
          <Tooltip title="Bottom" placement="bottom">
            <Button variant="outlined" size="small">Bottom</Button>
          </Tooltip>
          <div />
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Delays</h2>
        <div className="flex gap-4">
          <Tooltip title="Opens after 1 second" enterDelay={1000}>
            <Button variant="outlined">Slow Open (1s delay)</Button>
          </Tooltip>
          <Tooltip title="Stays visible for 2 seconds" leaveDelay={2000}>
            <Button variant="outlined">Slow Close (2s delay)</Button>
          </Tooltip>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Controlled</h2>
        <div className="flex gap-4 items-center">
          <Tooltip title="Controlled tooltip" open={open} onOpenChange={setOpen}>
            <Button variant="outlined">Click to toggle</Button>
          </Tooltip>
          <Button variant="text" size="small" onClick={() => setOpen(!open)}>
            {open ? 'Close' : 'Open'} programmatically
          </Button>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Arrow</h2>
        <Tooltip title="Tooltip with arrow" arrow>
          <Button variant="outlined">Hover for arrow</Button>
        </Tooltip>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Rich Content</h2>
        <Tooltip title={<span className="font-bold">Bold text</span>}>
          <Button variant="outlined">Rich content tooltip</Button>
        </Tooltip>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Long Content</h2>
        <Tooltip title="This is a very long tooltip that should wrap to multiple lines and show how the tooltip handles longer content gracefully">
          <Button variant="outlined">Long tooltip</Button>
        </Tooltip>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Icon Button</h2>
        <Tooltip title="Settings">
          <button className="p-2 hover:bg-nest-muted rounded-nest-md" aria-label="Settings">
            ⚙️
          </button>
        </Tooltip>
        <Tooltip title="Delete">
          <button className="p-2 hover:bg-nest-error/10 rounded-nest-md text-nest-error" aria-label="Delete">
            🗑️
          </button>
        </Tooltip>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Form Field Help</h2>
        <div className="space-y-2 max-w-xs">
          <label className="text-sm font-medium">
            Email
            <Tooltip title="We'll never share your email with anyone">
              <span className="ml-2 text-nest-muted cursor-help">ℹ️</span>
            </Tooltip>
          </label>
          <input
            type="email"
            className="w-full border border-nest-border rounded-nest-md px-3 py-2"
            placeholder="Enter email"
          />
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled State</h2>
        <Tooltip title="This button is disabled">
          <span>
            <Button variant="outlined" disabled>Disabled Button</Button>
          </span>
        </Tooltip>
      </section>
    </div>
  );
}
