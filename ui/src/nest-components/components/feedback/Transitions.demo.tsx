import { useState } from 'react';
import { Fade, Grow, Collapse } from './Transitions';
import { Button } from '../inputs/Button';

export function TransitionsDemos() {
  const [fadeOpen, setFadeOpen] = useState(false);
  const [growOpen, setGrowOpen] = useState(false);
  const [collapseOpen, setCollapseOpen] = useState(false);

  return (
    <div className="space-y-8 p-6">
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Fade</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setFadeOpen(!fadeOpen)}>
              {fadeOpen ? 'Hide' : 'Show'} Fade
            </Button>
          </div>
          <Fade in={fadeOpen}>
            <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
              <p className="text-nest-foreground">
                This content fades in and out. The fade transition uses opacity
                to smoothly show and hide content.
              </p>
            </div>
          </Fade>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Grow</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setGrowOpen(!growOpen)}>
              {growOpen ? 'Hide' : 'Show'} Grow
            </Button>
          </div>
          <Grow in={growOpen}>
            <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
              <p className="text-nest-foreground">
                This content grows in and out. The grow transition uses scale
                and opacity for a zoom effect.
              </p>
            </div>
          </Grow>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Collapse</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setCollapseOpen(!collapseOpen)}>
              {collapseOpen ? 'Collapse' : 'Expand'}
            </Button>
          </div>
          <Collapse in={collapseOpen}>
            <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
              <p className="text-nest-foreground mb-2">
                This content collapses vertically. The collapse transition
                animates the height (or width) of the content.
              </p>
              <p className="text-nest-foreground">
                It's useful for accordions, expandable sections, and revealing
                additional content.
              </p>
            </div>
          </Collapse>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Horizontal Collapse</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setCollapseOpen(!collapseOpen)}>
              Toggle
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outlined">Start</Button>
            <Collapse in={collapseOpen} orientation="horizontal">
              <div className="whitespace-nowrap">
                <Button variant="outlined">Middle Content</Button>
              </div>
            </Collapse>
            <Button variant="outlined">End</Button>
          </div>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Timeout</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setFadeOpen(!fadeOpen)}>
              Toggle Slow Fade (800ms)
            </Button>
          </div>
          <Fade in={fadeOpen} timeout={800}>
            <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
              <p className="text-nest-foreground">
                This fade has a custom 800ms timeout for a slower, more dramatic
                transition effect.
              </p>
            </div>
          </Fade>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Unmount on Exit</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setGrowOpen(!growOpen)}>
              {growOpen ? 'Hide' : 'Show'} (unmountOnExit)
            </Button>
          </div>
          <Grow in={growOpen} unmountOnExit>
            <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
              <p className="text-nest-foreground">
                This content is unmounted from the DOM after the exit transition
                completes. Check the DOM inspector to see it disappear.
              </p>
            </div>
          </Grow>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Chained Transitions</h2>
        <div className="space-y-4">
          <div className="flex gap-2">
            <Button onClick={() => setFadeOpen(!fadeOpen)}>
              {fadeOpen ? 'Hide' : 'Show'} Chained
            </Button>
          </div>
          <Fade in={fadeOpen}>
            <Grow in={fadeOpen}>
              <div className="p-4 bg-nest-surface border border-nest-border rounded-nest-md max-w-sm">
                <p className="text-nest-foreground mb-2">
                  This content uses both Fade and Grow transitions together
                  for a combined effect.
                </p>
                <Collapse in={fadeOpen}>
                  <p className="text-nest-foreground text-sm">
                    With an additional Collapse for extra flair!
                  </p>
                </Collapse>
              </div>
            </Grow>
          </Fade>
        </div>
      </section>

      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Expandable Content</h2>
        <div className="border border-nest-border rounded-nest-md overflow-hidden max-w-md">
          <div className="p-4 bg-nest-surface">
            <h3 className="font-semibold text-nest-foreground">Click to expand</h3>
            <p className="text-sm text-nest-muted">
              Reveals additional information below.
            </p>
          </div>
          <Collapse in={collapseOpen}>
            <div className="p-4 bg-nest-background border-t border-nest-border">
              <p className="text-nest-foreground">
                Here's the expanded content! You can put anything here - forms,
                images, lists, or more complex components.
              </p>
              <ul className="mt-2 list-disc list-inside text-nest-foreground text-sm">
                <li>Item one</li>
                <li>Item two</li>
                <li>Item three</li>
              </ul>
            </div>
          </Collapse>
          <div className="p-2 bg-nest-surface border-t border-nest-border">
            <Button
              variant="text"
              size="small"
              onClick={() => setCollapseOpen(!collapseOpen)}
            >
              {collapseOpen ? 'Show less' : 'Show more'}
            </Button>
          </div>
        </div>
      </section>
    </div>
  );
}
