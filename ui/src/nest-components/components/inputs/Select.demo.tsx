import { useState } from 'react';
import { Select } from './Select';

/**
 * Select Component Demos
 *
 * Copy these examples into your app to get started.
 */

const OPTIONS = [
  { value: 'react', label: 'React' },
  { value: 'vue', label: 'Vue' },
  { value: 'svelte', label: 'Svelte' },
  { value: 'angular', label: 'Angular', disabled: true },
];

export function SelectDemos() {
  const [value, setValue] = useState('react');

  return (
    <div className="space-y-8 p-6">
      {/* Basic Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic</h2>
        <div className="max-w-xs">
          <Select label="Framework" options={OPTIONS} value={value} onChange={setValue} />
        </div>
        <p className="mt-2 text-sm text-nest-muted">Selected: {value}</p>
      </section>

      {/* Sizes Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Sizes</h2>
        <div className="flex flex-wrap gap-4">
          <div className="w-48">
            <Select size="small" placeholder="Small" options={OPTIONS} />
          </div>
          <div className="w-48">
            <Select size="medium" placeholder="Medium" options={OPTIONS} />
          </div>
        </div>
      </section>

      {/* States Demo */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">States</h2>
        <div className="flex flex-wrap gap-4">
          <div className="w-48">
            <Select error placeholder="Error" options={OPTIONS} />
          </div>
          <div className="w-48">
            <Select disabled placeholder="Disabled" options={OPTIONS} />
          </div>
        </div>
      </section>
    </div>
  );
}
