import { IconButton } from './IconButton';
import { Bell, Heart, Settings, Star, Trash2 } from 'lucide-react';

/**
 * IconButton Component Demos
 *
 * Copy these examples into your app to get started.
 */

export function IconButtonDemos() {
  return (
    <div className="space-y-8 p-6">
      {/* Colors Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Colors</h2>
        <div className="flex flex-wrap items-center gap-3">
          <IconButton aria-label="default"><Settings className="size-5" /></IconButton>
          <IconButton aria-label="primary" color="primary"><Star className="size-5" /></IconButton>
          <IconButton aria-label="secondary" color="secondary"><Star className="size-5" /></IconButton>
          <IconButton aria-label="accent" color="accent"><Heart className="size-5" /></IconButton>
          <IconButton aria-label="success" color="success"><Heart className="size-5" /></IconButton>
          <IconButton aria-label="warning" color="warning"><Bell className="size-5" /></IconButton>
          <IconButton aria-label="error" color="error"><Trash2 className="size-5" /></IconButton>
          <IconButton aria-label="info" color="info"><Bell className="size-5" /></IconButton>
        </div>
      </section>

      {/* Sizes Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="flex flex-wrap items-center gap-3">
          <IconButton aria-label="small" size="small"><Heart className="size-4" /></IconButton>
          <IconButton aria-label="medium" size="medium"><Heart className="size-5" /></IconButton>
          <IconButton aria-label="large" size="large"><Heart className="size-6" /></IconButton>
        </div>
      </section>

      {/* States Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">States</h2>
        <div className="flex flex-wrap items-center gap-3">
          <IconButton aria-label="enabled" color="primary"><Settings className="size-5" /></IconButton>
          <IconButton aria-label="disabled" color="primary" disabled><Settings className="size-5" /></IconButton>
        </div>
      </section>
    </div>
  );
}
