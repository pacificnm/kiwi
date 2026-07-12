import { useState } from 'react';
import { Slider } from './Slider';
import { Typography } from '../../data-display/Typography';

export default function SliderDemo() {
  const [value, setValue] = useState<number>(30);
  const [rangeValue, setRangeValue] = useState<number[]>([20, 80]);
  const [verticalValue] = useState<number>(50);

  return (
    <div className="p-4 space-y-8">
      <div>
        <Typography variant="h5" className="mb-2">Slider</Typography>
        <Typography variant="body2" className="text-nest-muted">
          Range input for selecting values along a continuum.
        </Typography>
      </div>

      {/* Basic Slider */}
      <div className="space-y-2">
        <Typography variant="h6">Basic Slider</Typography>
        <Slider
          value={value}
          onChange={(v) => setValue(v as number)}
          min={0}
          max={100}
          step={1}
          label="Volume"
        />
        <Typography variant="body2" className="text-nest-muted">
          Current value: {value}
        </Typography>
      </div>

      {/* Range Slider */}
      <div className="space-y-2">
        <Typography variant="h6">Range Slider</Typography>
        <Slider
          value={rangeValue}
          onChange={(v) => setRangeValue(v as number[])}
          min={0}
          max={100}
          step={1}
          thumbLabels={['Min', 'Max']}
        />
        <Typography variant="body2" className="text-nest-muted">
          Range: {rangeValue[0]} - {rangeValue[1]}
        </Typography>
      </div>

      {/* Slider with Ticks */}
      <div className="space-y-2">
        <Typography variant="h6">Slider with Tick Marks</Typography>
        <Slider
          defaultValue={50}
          min={0}
          max={100}
          step={10}
          showTicks
        />
      </div>

      {/* Disabled Slider */}
      <div className="space-y-2">
        <Typography variant="h6">Disabled Slider</Typography>
        <Slider
          defaultValue={50}
          min={0}
          max={100}
          disabled
        />
      </div>

      {/* Vertical Slider */}
      <div className="space-y-2">
        <Typography variant="h6">Vertical Slider</Typography>
        <div className="h-48">
          <Slider
            value={verticalValue}
            onChange={(v) => setValue(v as number)}
            min={0}
            max={100}
            orientation="vertical"
            label="Brightness"
          />
        </div>
        <Typography variant="body2" className="text-nest-muted">
          Current value: {verticalValue}
        </Typography>
      </div>

      {/* Price Range */}
      <div className="space-y-2">
        <Typography variant="h6">Price Range</Typography>
        <Slider
          defaultValue={[50, 500]}
          min={0}
          max={1000}
          step={10}
          thumbLabels={['Min Price', 'Max Price']}
        />
        <Typography variant="body2" className="text-nest-muted">
          $50 - $500
        </Typography>
      </div>
    </div>
  );
}
