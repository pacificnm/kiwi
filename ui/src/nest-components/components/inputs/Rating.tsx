import { forwardRef } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';
import { Star, StarHalf } from 'lucide-react';

export type RatingSize = 'small' | 'medium' | 'large';
export type RatingColor = 'primary' | 'secondary' | 'accent' | 'warning' | 'error';

const SIZE_STYLES: Record<RatingSize, string> = {
  small: 'size-4',
  medium: 'size-6',
  large: 'size-8',
};

const COLOR_STYLES: Record<RatingColor, string> = {
  primary: 'text-nest-primary',
  secondary: 'text-nest-secondary',
  accent: 'text-nest-accent',
  warning: 'text-nest-warning',
  error: 'text-nest-error',
};

export interface RatingProps {
  /**
   * The value of the rating.
   */
  value?: number;
  /**
   * Default value for uncontrolled rating.
   */
  defaultValue?: number;
  /**
   * Callback fired when the rating changes.
   */
  onChange?: (event: React.SyntheticEvent, value: number) => void;
  /**
   * The maximum rating value.
   * @default 5
   */
  max?: number;
  /**
   * The minimum value the user can select.
   * @default 0
   */
  min?: number;
  /**
   * The step increment between values.
   * @default 1
   */
  precision?: number;
  /**
   * If true, the rating is read-only.
   * @default false
   */
  readOnly?: boolean;
  /**
   * If true, the rating is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * The size of the rating.
   * @default 'medium'
   */
  size?: RatingSize;
  /**
   * The color of the rating.
   * @default 'warning'
   */
  color?: RatingColor;
  /**
   * If true, empty stars are shown instead of just filled.
   * @default false
   */
  emptyIcon?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Rating component - a star rating input.
 * Follows MUI Rating API conventions.
 *
 * Rating allows users to rate items using stars.
 *
 * @example
 * // Basic rating
 * <Rating value={4} onChange={(e, v) => setValue(v)} />
 *
 * @example
 * // Read-only rating
 * <Rating value={4.5} readOnly />
 *
 * @example
 * // Custom max and precision
 * <Rating max={10} precision={0.5} defaultValue={7.5} />
 */
export const Rating = forwardRef<HTMLDivElement, RatingProps>(function Rating(
  {
    value: valueProp,
    defaultValue,
    onChange,
    max = 5,
    min = 0,
    precision = 1,
    readOnly = false,
    disabled = false,
    size = 'medium',
    color = 'warning',
    emptyIcon = false,
    className,
    ...props
  }: RatingProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [value, setValue] = useControllableState<number>({
    value: valueProp,
    defaultValue: defaultValue ?? 0,
    onChange: (val) => {
      const event = { target: { value: val } } as unknown as React.SyntheticEvent;
      onChange?.(event, val);
    },
  });

  const sizeStyles = SIZE_STYLES[size];
  const colorStyles = COLOR_STYLES[color];

  const handleMouseMove = () => {
    if (readOnly || disabled) return;
    // Could implement hover preview here with state
  };

  const handleClick = (newValue: number) => {
    if (readOnly || disabled) return;
    setValue(newValue);
  };

  const handleKeyDown = (event: React.KeyboardEvent, newValue: number) => {
    if (readOnly || disabled) return;
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      setValue(newValue);
    }
  };

  // Round value to precision
  const roundedValue = Math.round((value - min) / precision) * precision + min;

  const stars = [];
  for (let i = 1; i <= max; i++) {
    const filled = roundedValue >= i;
    const half = !filled && roundedValue >= i - 0.5 && precision <= 0.5;
    const isEmpty = !filled && !half && emptyIcon;

    stars.push(
      <button
        key={i}
        type="button"
        disabled={disabled || readOnly}
        onClick={() => handleClick(i)}
        onMouseMove={() => handleMouseMove()}
        onKeyDown={(e) => handleKeyDown(e, i)}
        className={cn(
          'transition-colors duration-150',
          'focus:outline-none',
          (disabled || readOnly) && 'cursor-default',
          !(disabled || readOnly) && 'cursor-pointer hover:scale-110'
        )}
        aria-label={`${i} star${i > 1 ? 's' : ''}`}
        aria-checked={roundedValue >= i}
        role="radio"
        tabIndex={(disabled || readOnly) ? -1 : 0}
      >
        {half ? (
          <StarHalf className={cn(sizeStyles, colorStyles)} />
        ) : filled || isEmpty ? (
          <Star
            className={cn(
              sizeStyles,
              filled ? colorStyles : 'text-nest-muted',
              filled && 'fill-current'
            )}
          />
        ) : (
          <Star className={cn(sizeStyles, 'text-nest-muted')} />
        )}
      </button>
    );
  }

  return (
    <div
      ref={ref as any}
      className={cn('flex items-center gap-0.5', className)}
      role="radiogroup"
      aria-label="Rating"
      {...props}
    >
      {stars}
    </div>
  );
});
