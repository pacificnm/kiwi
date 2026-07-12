# Slider

A range input component for selecting values along a continuum. Supports single values, ranges, vertical orientation, and tick marks.

## When to Use

- Selecting a numeric value within a range (volume, brightness, price)
- Setting a minimum and maximum range (price filters, date ranges)
- Adjusting settings with immediate visual feedback
- Interactive data exploration

## Anatomy

The Slider consists of:
- **Track**: The line representing the full range of possible values
- **Thumb(s)**: Draggable handles for selecting values
- **Fill**: The portion of the track representing selected values
- **Tick marks**: Optional visual indicators at step intervals

## Usage

### Basic Slider

```tsx
import { Slider } from '@nest/components';

function VolumeControl() {
  const [volume, setVolume] = useState(50);
  
  return (
    <Slider
      value={volume}
      onChange={setVolume}
      min={0}
      max={100}
      step={1}
      label="Volume"
    />
  );
}
```

### Range Slider

```tsx
function PriceRange() {
  const [range, setRange] = useState([50, 500]);
  
  return (
    <Slider
      value={range}
      onChange={setRange}
      min={0}
      max={1000}
      step={10}
      thumbLabels={['Min', 'Max']}
    />
  );
}
```

### Vertical Slider

```tsx
<BrightnessSlider>
  <Slider
    value={brightness}
    onChange={setBrightness}
    min={0}
    max={100}
    orientation="vertical"
    label="Brightness"
  />
</BrightnessSlider>
```

### Slider with Tick Marks

```tsx
<Slider
  defaultValue={50}
  min={0}
  max={100}
  step={10}
  showTicks
/>
```

### Disabled Slider

```tsx
<Slider
  defaultValue={50}
  min={0}
  max={100}
  disabled
/>
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `value` | `number \| number[]` | - | Controlled value |
| `defaultValue` | `number \| number[]` | - | Uncontrolled default value |
| `onChange` | `(value: number \| number[]) => void` | - | Value change callback |
| `min` | `number` | `0` | Minimum value |
| `max` | `number` | `100` | Maximum value |
| `step` | `number` | `1` | Step increment |
| `orientation` | `'horizontal' \| 'vertical'` | `'horizontal'` | Slider orientation |
| `disabled` | `boolean` | `false` | Disable the slider |
| `label` | `string` | - | Accessibility label |
| `showTicks` | `boolean` | `false` | Show tick marks |
| `thumbLabels` | `string[]` | - | Labels for each thumb |
| `trackClassName` | `string` | - | Custom track classes |
| `thumbClassName` | `string` | - | Custom thumb classes |
| `className` | `string` | - | Custom root classes |

## Accessibility

- Built with react-aria for full keyboard support
- Arrow keys adjust values (Shift + Arrow for larger steps)
- Home/End jump to min/max values
- Screen reader announcements for value changes
- Proper ARIA attributes for sliders

## Best Practices

- **Use appropriate step values**: Match step to the precision needed
- **Show current value**: Display the selected value alongside the slider
- **Label clearly**: Use the `label` prop for accessibility
- **Consider tick marks**: Helpful for discrete values or reference points
- **Range sliders**: Use `thumbLabels` to distinguish min/max thumbs

## Related Components

- **TextField**: For precise numeric input
- **Select**: For choosing from discrete options
- **Progress**: For displaying values without interaction
