import { Badge } from './Badge';
import { Avatar } from './Avatar';
import { IconButton } from '../inputs/IconButton';
import { Bell, Mail } from 'lucide-react';

/**
 * Badge Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function BadgeDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Badge */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Badge</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={4}>
            <Avatar src="https://i.pravatar.cc/150?img=1" alt="User" />
          </Badge>
          <Badge badgeContent={10}>
            <Avatar src="https://i.pravatar.cc/150?img=2" alt="User" />
          </Badge>
          <Badge badgeContent="New">
            <Avatar src="https://i.pravatar.cc/150?img=3" alt="User" />
          </Badge>
        </div>
      </section>

      {/* Color Variants */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Color Variants</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={1} color="default">
            <Avatar>Default</Avatar>
          </Badge>
          <Badge badgeContent={1} color="primary">
            <Avatar>Primary</Avatar>
          </Badge>
          <Badge badgeContent={1} color="secondary">
            <Avatar>Secondary</Avatar>
          </Badge>
          <Badge badgeContent={1} color="success">
            <Avatar>Success</Avatar>
          </Badge>
          <Badge badgeContent={1} color="warning">
            <Avatar>Warning</Avatar>
          </Badge>
          <Badge badgeContent={1} color="error">
            <Avatar>Error</Avatar>
          </Badge>
          <Badge badgeContent={1} color="info">
            <Avatar>Info</Avatar>
          </Badge>
        </div>
      </section>

      {/* Dot Variant */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Dot Variant</h2>
        <div className="flex items-center gap-4">
          <Badge variant="dot" color="default">
            <Avatar>Default</Avatar>
          </Badge>
          <Badge variant="dot" color="primary">
            <Avatar>Primary</Avatar>
          </Badge>
          <Badge variant="dot" color="success">
            <Avatar>Success</Avatar>
          </Badge>
          <Badge variant="dot" color="error">
            <Avatar>Error</Avatar>
          </Badge>
        </div>
      </section>

      {/* Max Value */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Max Value (99+)</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={5} max={99}>
            <Avatar>5</Avatar>
          </Badge>
          <Badge badgeContent={50} max={99}>
            <Avatar>50</Avatar>
          </Badge>
          <Badge badgeContent={99} max={99}>
            <Avatar>99</Avatar>
          </Badge>
          <Badge badgeContent={100} max={99}>
            <Avatar>100</Avatar>
          </Badge>
          <Badge badgeContent={999} max={99}>
            <Avatar>999</Avatar>
          </Badge>
        </div>
      </section>

      {/* Anchor Positions */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Anchor Positions</h2>
        <div className="flex items-center gap-6">
          <div>
            <p className="mb-2 text-center text-xs text-nest-muted">Top Right</p>
            <Badge badgeContent={1} anchorOrigin={{ vertical: 'top', horizontal: 'right' }}>
              <Avatar>TR</Avatar>
            </Badge>
          </div>
          <div>
            <p className="mb-2 text-center text-xs text-nest-muted">Top Left</p>
            <Badge badgeContent={1} anchorOrigin={{ vertical: 'top', horizontal: 'left' }}>
              <Avatar>TL</Avatar>
            </Badge>
          </div>
          <div>
            <p className="mb-2 text-center text-xs text-nest-muted">Bottom Right</p>
            <Badge badgeContent={1} anchorOrigin={{ vertical: 'bottom', horizontal: 'right' }}>
              <Avatar>BR</Avatar>
            </Badge>
          </div>
          <div>
            <p className="mb-2 text-center text-xs text-nest-muted">Bottom Left</p>
            <Badge badgeContent={1} anchorOrigin={{ vertical: 'bottom', horizontal: 'left' }}>
              <Avatar>BL</Avatar>
            </Badge>
          </div>
        </div>
      </section>

      {/* showZero */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">showZero</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={0}>
            <Avatar>Hidden</Avatar>
          </Badge>
          <Badge badgeContent={0} showZero>
            <Avatar>Visible</Avatar>
          </Badge>
        </div>
        <p className="text-sm text-nest-muted">Left: hidden (default), Right: visible (showZero)</p>
      </section>

      {/* Icon Buttons with Badges */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Icon Buttons with Badges</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={3} color="error">
            <IconButton aria-label="notifications">
              <Bell className="h-5 w-5" />
            </IconButton>
          </Badge>
          <Badge variant="dot" color="error">
            <IconButton aria-label="messages">
              <Mail className="h-5 w-5" />
            </IconButton>
          </Badge>
          <Badge badgeContent={99} max={99} color="primary">
            <IconButton aria-label="emails">
              <Mail className="h-5 w-5" />
            </IconButton>
          </Badge>
        </div>
      </section>

      {/* Invisible Badge */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Invisible Badge</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent={5}>
            <Avatar>Visible</Avatar>
          </Badge>
          <Badge badgeContent={5} invisible>
            <Avatar>Invisible</Avatar>
          </Badge>
        </div>
      </section>

      {/* Badge on Cards */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Badge on Cards</h2>
        <div className="flex items-center gap-4">
          <Badge badgeContent="Sale" color="error" className="!static">
            <div className="flex h-20 w-32 items-center justify-center rounded-nest-md border border-nest-border bg-nest-surface">
              Card
            </div>
          </Badge>
          <Badge badgeContent={3} color="primary" className="!static">
            <div className="flex h-20 w-32 items-center justify-center rounded-nest-md border border-nest-border bg-nest-surface">
              Card
            </div>
          </Badge>
        </div>
        <p className="text-sm text-nest-muted">Using !static to position badge relative to card</p>
      </section>

      {/* Custom Styling */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Styling</h2>
        <div className="flex items-center gap-4">
          <Badge
            badgeContent={5}
            className="bg-gradient-to-br from-nest-primary to-nest-accent !text-white"
          >
            <Avatar>Gradient</Avatar>
          </Badge>
          <Badge
            badgeContent={3}
            className="border-2 border-nest-surface"
            color="primary"
          >
            <Avatar>Bordered</Avatar>
          </Badge>
          <Badge
            badgeContent="Pro"
            className="bg-nest-foreground text-nest-background font-bold"
          >
            <Avatar>Custom</Avatar>
          </Badge>
        </div>
      </section>

      {/* Status Indicators */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Status Indicators</h2>
        <div className="space-y-4">
          <div className="flex items-center gap-4">
            <div className="relative">
              <Avatar src="https://i.pravatar.cc/150?img=1" alt="Online" />
              <Badge
                variant="dot"
                color="success"
                className="!static bottom-0 right-0 !translate-x-0 !translate-y-0"
              />
            </div>
            <span>Online</span>
          </div>
          <div className="flex items-center gap-4">
            <div className="relative">
              <Avatar src="https://i.pravatar.cc/150?img=2" alt="Away" />
              <Badge
                variant="dot"
                color="warning"
                className="!static bottom-0 right-0 !translate-x-0 !translate-y-0"
              />
            </div>
            <span>Away</span>
          </div>
          <div className="flex items-center gap-4">
            <div className="relative">
              <Avatar src="https://i.pravatar.cc/150?img=3" alt="Busy" />
              <Badge
                variant="dot"
                color="error"
                className="!static bottom-0 right-0 !translate-x-0 !translate-y-0"
              />
            </div>
            <span>Busy</span>
          </div>
          <div className="flex items-center gap-4">
            <div className="relative">
              <Avatar src="https://i.pravatar.cc/150?img=4" alt="Offline" />
              <Badge
                variant="dot"
                color="default"
                className="!static bottom-0 right-0 !translate-x-0 !translate-y-0"
              />
            </div>
            <span>Offline</span>
          </div>
        </div>
      </section>
    </div>
  );
}
