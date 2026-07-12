import { useState } from 'react';
import { Chip } from './Chip';
import { Star, Mail, Phone } from 'lucide-react';

/**
 * Chip Demos
 *
 * Run this component in a gallery/demo viewer to see all variants.
 */
export function ChipDemos() {
  const [chips, setChips] = useState(['Chip 1', 'Chip 2', 'Chip 3']);

  const removeChip = (chip: string) => {
    setChips(chips.filter((c) => c !== chip));
  };

  const addChip = () => {
    const newChip = `Chip ${chips.length + 1}`;
    setChips([...chips, newChip]);
  };

  return (
    <div className="space-y-8 p-6">
      {/* Basic Chips */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Basic Chips</h2>
        <div className="flex flex-wrap gap-2">
          <Chip label="Default Chip" />
          <Chip label="Primary Chip" color="primary" />
          <Chip label="Secondary Chip" color="secondary" />
          <Chip label="Accent Chip" color="accent" />
          <Chip label="Success Chip" color="success" />
          <Chip label="Warning Chip" color="warning" />
          <Chip label="Error Chip" color="error" />
          <Chip label="Info Chip" color="info" />
        </div>
      </section>

      {/* Variant Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Variant Comparison</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">variant="filled"</p>
            <div className="flex flex-wrap gap-2">
              <Chip label="Default" variant="filled" color="default" />
              <Chip label="Primary" variant="filled" color="primary" />
              <Chip label="Error" variant="filled" color="error" />
            </div>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">variant="outlined"</p>
            <div className="flex flex-wrap gap-2">
              <Chip label="Default" variant="outlined" color="default" />
              <Chip label="Primary" variant="outlined" color="primary" />
              <Chip label="Error" variant="outlined" color="error" />
            </div>
          </div>
        </div>
      </section>

      {/* Size Comparison */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Size Comparison</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">size="small"</p>
            <div className="flex flex-wrap gap-2">
              <Chip label="Small Chip" size="small" />
              <Chip label="Small Primary" size="small" color="primary" />
            </div>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">size="medium" (default)</p>
            <div className="flex flex-wrap gap-2">
              <Chip label="Medium Chip" size="medium" />
              <Chip label="Medium Primary" size="medium" color="primary" />
            </div>
          </div>
        </div>
      </section>

      {/* Chips with Icons */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Chips with Icons</h2>
        <div className="flex flex-wrap gap-2">
          <Chip label="Star Chip" icon={<Star className="h-4 w-4" />} />
          <Chip label="Email Chip" icon={<Mail className="h-4 w-4" />} color="primary" />
          <Chip label="Phone Chip" icon={<Phone className="h-4 w-4" />} color="success" />
          <Chip label="Filled Star" icon={<Star className="h-4 w-4" />} variant="filled" color="accent" />
        </div>
      </section>

      {/* Deletable Chips */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Deletable Chips</h2>
        <div className="flex flex-wrap gap-2">
          {chips.map((chip) => (
            <Chip
              key={chip}
              label={chip}
              onDelete={() => removeChip(chip)}
              color="primary"
            />
          ))}
          <Chip label="Add Chip" onClick={addChip} clickable color="success" />
        </div>
        <p className="mt-2 text-sm text-nest-muted">Click the X to delete, click "Add Chip" to add</p>
      </section>

      {/* Clickable Chips */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Clickable Chips</h2>
        <div className="flex flex-wrap gap-2">
          <Chip
            label="Clickable Default"
            clickable
            onClick={() => alert('Clicked!')}
          />
          <Chip
            label="Clickable Primary"
            clickable
            color="primary"
            onClick={() => alert('Clicked Primary!')}
          />
          <Chip
            label="Clickable Outlined"
            clickable
            variant="outlined"
            color="secondary"
            onClick={() => alert('Clicked Outlined!')}
          />
        </div>
      </section>

      {/* Disabled Chips */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Disabled Chips</h2>
        <div className="flex flex-wrap gap-2">
          <Chip label="Disabled" disabled />
          <Chip label="Disabled Primary" color="primary" disabled />
          <Chip label="Disabled Outlined" variant="outlined" color="error" disabled />
          <Chip
            label="Disabled with Delete"
            color="success"
            disabled
            onDelete={() => {}}
          />
        </div>
      </section>

      {/* Chip as Selection */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Chip as Selection</h2>
        <SelectionDemo />
      </section>

      {/* Chip with Long Text */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">Chip with Long Text</h2>
        <div className="max-w-xs space-y-2">
          <Chip label="Short" />
          <Chip label="Medium length chip" />
          <Chip label="This is a very long chip label that should truncate with ellipsis" />
          <Chip
            label="Long deletable chip with very long text"
            onDelete={() => {}}
          />
        </div>
      </section>

      {/* Chip Colors Reference */}
      <section>
        <h2 className="mb-4 text-lg font-semibold text-nest-foreground">All Colors Reference</h2>
        <div className="space-y-4">
          <div>
            <p className="mb-2 text-sm text-nest-muted">Filled</p>
            <div className="flex flex-wrap gap-2">
              {(['default', 'primary', 'secondary', 'accent', 'success', 'warning', 'error', 'info'] as const).map((color) => (
                <Chip key={color} label={color} color={color} variant="filled" />
              ))}
            </div>
          </div>
          <div>
            <p className="mb-2 text-sm text-nest-muted">Outlined</p>
            <div className="flex flex-wrap gap-2">
              {(['default', 'primary', 'secondary', 'accent', 'success', 'warning', 'error', 'info'] as const).map((color) => (
                <Chip key={color} label={color} color={color} variant="outlined" />
              ))}
            </div>
          </div>
        </div>
      </section>
    </div>
  );
}

function SelectionDemo() {
  const [selected, setSelected] = useState<string>('option-2');

  const options = [
    { value: 'option-1', label: 'Option 1' },
    { value: 'option-2', label: 'Option 2' },
    { value: 'option-3', label: 'Option 3' },
    { value: 'option-4', label: 'Option 4' },
  ];

  return (
    <div className="flex flex-wrap gap-2">
      {options.map((option) => (
        <Chip
          key={option.value}
          label={option.label}
          variant={selected === option.value ? 'filled' : 'outlined'}
          color={selected === option.value ? 'primary' : 'default'}
          clickable
          onClick={() => setSelected(option.value)}
        />
      ))}
    </div>
  );
}
