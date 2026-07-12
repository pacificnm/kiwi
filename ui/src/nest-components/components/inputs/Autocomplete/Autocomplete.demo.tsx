import { useState } from 'react';
import { Autocomplete } from './Autocomplete';

/**
 * Autocomplete Component Demos
 *
 * Copy these examples into your app to get started.
 */

const LANGUAGES = [
  { value: 'ts', label: 'TypeScript', description: 'Typed superset of JavaScript' },
  { value: 'rs', label: 'Rust', description: 'Memory-safe systems language' },
  { value: 'go', label: 'Go', description: 'Simple, concurrent language' },
  { value: 'py', label: 'Python', description: 'Batteries-included scripting' },
];

const SIMPLE = [
  { value: 'react', label: 'React' },
  { value: 'vue', label: 'Vue' },
  { value: 'svelte', label: 'Svelte' },
];

export function AutocompleteDemos() {
  const [value, setValue] = useState('');

  return (
    <div className="space-y-8 p-6">
      {/* Basic Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic</h2>
        <div className="max-w-xs">
          <Autocomplete
            label="Framework"
            options={SIMPLE}
            value={value}
            onChange={setValue}
            placeholder="Search…"
          />
        </div>
        <p className="mt-2 text-sm text-nest-muted">Selected: {value || '(none)'}</p>
      </section>

      {/* With descriptions Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">With Descriptions</h2>
        <div className="max-w-xs">
          <Autocomplete label="Language" options={LANGUAGES} placeholder="Type to filter…" />
        </div>
      </section>

      {/* Disabled Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled</h2>
        <div className="max-w-xs">
          <Autocomplete label="Framework" options={SIMPLE} placeholder="Disabled" disabled />
        </div>
      </section>
    </div>
  );
}
