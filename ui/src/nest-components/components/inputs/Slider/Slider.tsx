import { useRef, useRef as useReactRef } from 'react';
import { cn } from '../../../lib/cn';
import { useSlider, useSliderThumb } from '@react-aria/slider';
import { useFocusRing } from '@react-aria/focus';
import { VisuallyHidden } from '@react-aria/visually-hidden';
import { mergeProps } from '@react-aria/utils';
import { useNumberFormatter } from '@react-aria/i18n';
import { useSliderState } from '@react-stately/slider';
import type { AriaSliderProps } from '@react-types/slider';

export interface SliderProps extends Omit<AriaSliderProps, 'onChange' | 'maxValue' | 'minValue'> {
  /**
   * The orientation of the slider.
   * @default 'horizontal'
   */
  orientation?: 'horizontal' | 'vertical';
  /**
   * If true, the slider will be disabled.
   */
  disabled?: boolean;
  /**
   * Callback fired when the slider value changes.
   */
  onChange?: (value: number | number[]) => void;
  /**
   * Additional CSS classes for the slider track.
   */
  trackClassName?: string;
  /**
   * Additional CSS classes for the slider thumb.
   */
  thumbClassName?: string;
  /**
   * Additional CSS classes for the root element.
   */
  className?: string;
  /**
   * If true, shows tick marks at the step intervals.
   */
  showTicks?: boolean;
  /**
   * Labels for each thumb (for multi-thumb sliders).
   */
  thumbLabels?: string[];
  /**
   * The maximum value of the slider.
   * @default 100
   */
  max?: number;
  /**
   * The minimum value of the slider.
   * @default 0
   */
  min?: number;
}

/**
 * Slider component - a range input for selecting values along a continuum.
 * Follows MUI Slider API conventions with react-aria for accessibility.
 *
 * @example
 * // Basic slider
 * <Slider
 *   defaultValue={30}
 *   min={0}
 *   max={100}
 *   step={1}
 * />
 *
 * @example
 * // Range slider
 * <Slider
 *   defaultValue={[20, 80]}
 *   min={0}
 *   max={100}
 *   step={1}
 * />
 */
export function Slider({
  orientation = 'horizontal',
  disabled = false,
  onChange,
  trackClassName,
  thumbClassName,
  className,
  showTicks = false,
  thumbLabels,
  max = 100,
  min = 0,
  ...props
}: SliderProps) {
  const containerRef = useReactRef<HTMLDivElement>(null);

  // useSliderState requires a number formatter for getThumbValueLabel; without it
  // the screen-reader output crashes ("Cannot read properties of undefined (reading 'format')").
  const numberFormatter = useNumberFormatter();

  const state = useSliderState({
    ...props,
    maxValue: max,
    minValue: min,
    numberFormatter,
    onChange: (value: number[]) => {
      onChange?.(value.length === 1 ? value[0] : value as number | number[]);
    },
  } as any);

  const { groupProps, trackProps, labelProps, outputProps } = useSlider(
    {
      ...props,
      orientation,
      isDisabled: disabled,
      maxValue: max,
      minValue: min,
    } as any,
    state,
    containerRef
  );

  const isVertical = orientation === 'vertical';
  const tickCount = Math.floor((max - min) / (props.step || 1)) + 1;

  return (
    <div
      {...groupProps}
      ref={containerRef}
      className={cn(
        'relative',
        isVertical ? 'h-48 w-12' : 'w-full h-12',
        'flex items-center justify-center',
        disabled && 'opacity-50 cursor-not-allowed',
        className
      )}
    >
      {props.label && (
        <label
          {...labelProps}
          className={cn(
            'absolute text-sm font-medium text-nest-foreground',
            isVertical ? '-top-6 left-1/2 -translate-x-1/2' : '-top-5 left-0'
          )}
        >
          {props.label}
        </label>
      )}

      {/* Track */}
      <div
        {...trackProps}
        className={cn(
          'relative bg-nest-border rounded-full',
          isVertical ? 'w-1.5 h-full' : 'h-1.5 w-full',
          trackClassName
        )}
      >
        {/* Fill track - handle single and multi-thumb */}
        {state.values.length === 1 ? (
          <div
            className={cn(
              'absolute bg-nest-primary rounded-full',
              isVertical ? 'w-full bottom-0 left-0' : 'h-full left-0 top-0'
            )}
            style={
              isVertical
                ? { height: `${state.getThumbPercent(0) * 100}%` }
                : { width: `${state.getThumbPercent(0) * 100}%` }
            }
          />
        ) : (
          <div
            className={cn(
              'absolute bg-nest-primary rounded-full',
              isVertical ? 'w-full left-0' : 'h-full top-0'
            )}
            style={
              isVertical
                ? {
                    bottom: `${state.getThumbPercent(0) * 100}%`,
                    height: `${(state.getThumbPercent(1) - state.getThumbPercent(0)) * 100}%`,
                  }
                : {
                    left: `${state.getThumbPercent(0) * 100}%`,
                    width: `${(state.getThumbPercent(1) - state.getThumbPercent(0)) * 100}%`,
                  }
            }
          />
        )}

        {/* Tick marks */}
        {showTicks && (
          <div className={cn('absolute inset-0', isVertical ? 'flex flex-col' : 'flex')}>
            {Array.from({ length: tickCount }).map((_, i) => (
              <div
                key={i}
                className={cn(
                  'absolute w-1 h-1 bg-nest-foreground rounded-full',
                  isVertical ? 'left-1/2 -translate-x-1/2' : 'top-1/2 -translate-y-1/2'
                )}
                style={
                  isVertical
                    ? { bottom: `${(i / (tickCount - 1)) * 100}%` }
                    : { left: `${(i / (tickCount - 1)) * 100}%` }
                }
              />
            ))}
          </div>
        )}
      </div>

      {/* Thumbs */}
      {state.values.map((_, index) => (
        <SliderThumb
          key={index}
          index={index}
          state={state}
          orientation={orientation}
          disabled={disabled}
          className={thumbClassName}
          label={thumbLabels?.[index]}
        />
      ))}

      {/* Output for screen readers */}
      <VisuallyHidden>
        <output {...outputProps}>
          {state.values.map((_, i) => state.getThumbValueLabel(i)).join(' - ')}
        </output>
      </VisuallyHidden>
    </div>
  );
}

interface SliderThumbProps {
  index: number;
  state: ReturnType<typeof useSliderState>;
  orientation: 'horizontal' | 'vertical';
  disabled: boolean;
  className?: string;
  label?: string;
}

function SliderThumb({ index, state, orientation, disabled, className, label }: SliderThumbProps) {
  const inputRef = useRef<HTMLInputElement>(null);
  const trackRef = useRef<HTMLDivElement>(null);

  const { thumbProps, inputProps } = useSliderThumb(
    {
      index,
      inputRef,
      trackRef,
      isDisabled: disabled,
    },
    state
  );

  const { focusProps, isFocusVisible } = useFocusRing();

  const isVertical = orientation === 'vertical';

  return (
    <div
      ref={trackRef}
      className={cn(
        'absolute',
        'w-5 h-5',
        'flex items-center justify-center',
        isVertical ? 'left-1/2 -translate-x-1/2' : 'top-1/2 -translate-y-1/2'
      )}
      style={
        isVertical
          ? { bottom: `calc(${state.getThumbPercent(index) * 100}% - 10px)` }
          : { left: `calc(${state.getThumbPercent(index) * 100}% - 10px)` }
      }
    >
      <input
        {...inputProps}
        ref={inputRef}
        className="sr-only"
        aria-label={label || `Thumb ${index + 1}`}
      />
      <div
        {...mergeProps(thumbProps, focusProps)}
        className={cn(
          'w-5 h-5',
          'bg-nest-surface',
          'border-2 border-nest-primary',
          'rounded-full',
          'shadow-md',
          'cursor-pointer',
          'transition-transform duration-150',
          !disabled && 'hover:scale-110',
          isFocusVisible && 'ring-2 ring-nest-primary ring-offset-2',
          className
        )}
        tabIndex={disabled ? -1 : 0}
      />
    </div>
  );
}
