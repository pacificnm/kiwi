import { Avatar } from './Avatar';
import { User } from 'lucide-react';

/**
 * Avatar Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function AvatarDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Basic Avatars */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Avatars</h2>
        <div className="flex items-center gap-4">
          <Avatar src="https://i.pravatar.cc/150?img=1" alt="User 1" />
          <Avatar src="https://i.pravatar.cc/150?img=2" alt="User 2" />
          <Avatar src="https://i.pravatar.cc/150?img=3" alt="User 3" />
        </div>
      </section>

      {/* Fallback Avatars */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Fallback Avatars</h2>
        <div className="flex items-center gap-4">
          <Avatar>JD</Avatar>
          <Avatar>AB</Avatar>
          <Avatar>
            <User className="h-5 w-5" />
          </Avatar>
          <Avatar>👤</Avatar>
        </div>
        <p className="text-sm text-nest-muted">Initials, icons, or emoji as fallback</p>
      </section>

      {/* Size Variations */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Size Variations</h2>
        <div className="flex items-center gap-4">
          <Avatar size="small" src="https://i.pravatar.cc/150?img=1" alt="Small" />
          <Avatar size="medium" src="https://i.pravatar.cc/150?img=2" alt="Medium" />
          <Avatar size="large" src="https://i.pravatar.cc/150?img=3" alt="Large" />
        </div>
        <div className="flex items-center gap-4">
          <Avatar size="small">S</Avatar>
          <Avatar size="medium">M</Avatar>
          <Avatar size="large">L</Avatar>
        </div>
      </section>

      {/* Variant Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variant Comparison</h2>
        <div className="flex items-center gap-4">
          <Avatar variant="circular" src="https://i.pravatar.cc/150?img=1" alt="Circular" />
          <Avatar variant="rounded" src="https://i.pravatar.cc/150?img=2" alt="Rounded" />
          <Avatar variant="square" src="https://i.pravatar.cc/150?img=3" alt="Square" />
        </div>
        <div className="flex items-center gap-4">
          <Avatar variant="circular">C</Avatar>
          <Avatar variant="rounded">R</Avatar>
          <Avatar variant="square">S</Avatar>
        </div>
      </section>

      {/* Image Error Fallback */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Image Error Fallback</h2>
        <div className="flex items-center gap-4">
          <Avatar src="/invalid-image.jpg" alt="Invalid">
            Fallback
          </Avatar>
          <Avatar src="/also-invalid.png" alt="Also Invalid">
            <User className="h-5 w-5" />
          </Avatar>
          <Avatar src="/not-found.jpg" alt="Not Found">
            JD
          </Avatar>
        </div>
        <p className="text-sm text-nest-muted">Shows fallback when image fails to load</p>
      </section>

      {/* Avatar Group */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Avatar Group</h2>
        <div className="flex -space-x-2">
          <Avatar src="https://i.pravatar.cc/150?img=1" alt="User 1" className="ring-2 ring-nest-surface" />
          <Avatar src="https://i.pravatar.cc/150?img=2" alt="User 2" className="ring-2 ring-nest-surface" />
          <Avatar src="https://i.pravatar.cc/150?img=3" alt="User 3" className="ring-2 ring-nest-surface" />
          <Avatar src="https://i.pravatar.cc/150?img=4" alt="User 4" className="ring-2 ring-nest-surface" />
          <Avatar className="ring-2 ring-nest-surface">+3</Avatar>
        </div>
      </section>

      {/* Avatar with Status */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Avatar with Status</h2>
        <div className="flex items-center gap-6">
          <div className="relative">
            <Avatar src="https://i.pravatar.cc/150?img=1" alt="Online" />
            <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-nest-success ring-2 ring-nest-surface" />
          </div>
          <div className="relative">
            <Avatar src="https://i.pravatar.cc/150?img=2" alt="Away" />
            <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-nest-warning ring-2 ring-nest-surface" />
          </div>
          <div className="relative">
            <Avatar src="https://i.pravatar.cc/150?img=3" alt="Offline" />
            <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-nest-muted ring-2 ring-nest-surface" />
          </div>
          <div className="relative">
            <Avatar src="https://i.pravatar.cc/150?img=4" alt="Busy" />
            <span className="absolute bottom-0 right-0 h-3 w-3 rounded-full bg-nest-error ring-2 ring-nest-surface" />
          </div>
        </div>
      </section>

      {/* Clickable Avatar */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Clickable Avatar</h2>
        <div className="flex items-center gap-4">
          <Avatar
            component="button"
            src="https://i.pravatar.cc/150?img=1"
            alt="Clickable"
            className="cursor-pointer hover:opacity-80 focus:outline-none focus:ring-2 focus:ring-nest-primary/50"
            onClick={() => alert('Avatar clicked!')}
          />
          <Avatar
            component="a"
            href="/profile"
            src="https://i.pravatar.cc/150?img=2"
            alt="Link"
            className="cursor-pointer hover:opacity-80"
          />
        </div>
      </section>

      {/* Avatar in Context */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Avatar in Context</h2>
        <div className="space-y-4">
          {/* Comment */}
          <div className="flex gap-3">
            <Avatar src="https://i.pravatar.cc/150?img=5" alt="Commenter" />
            <div>
              <div className="flex items-center gap-2">
                <span className="font-medium">John Doe</span>
                <span className="text-xs text-nest-muted">2 hours ago</span>
              </div>
              <p className="text-sm text-nest-muted">This is a comment with an avatar.</p>
            </div>
          </div>
          {/* User Card */}
          <div className="flex items-center gap-3 border border-nest-border p-3 rounded-nest-md">
            <Avatar src="https://i.pravatar.cc/150?img=6" alt="User" />
            <div>
              <p className="font-medium">Jane Smith</p>
              <p className="text-xs text-nest-muted">jane@example.com</p>
            </div>
          </div>
        </div>
      </section>

      {/* Custom Styling */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Custom Styling</h2>
        <div className="flex items-center gap-4">
          <Avatar className="bg-nest-primary text-white">P</Avatar>
          <Avatar className="bg-nest-secondary text-white">S</Avatar>
          <Avatar className="bg-gradient-to-br from-nest-primary to-nest-accent text-white">
            G
          </Avatar>
          <Avatar className="border-2 border-nest-primary">B</Avatar>
        </div>
      </section>
    </div>
  );
}
