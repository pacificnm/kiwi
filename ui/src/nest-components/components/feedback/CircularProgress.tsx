import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type CircularProgressVariant = 'indeterminate' | 'determinate';
export type CircularProgressColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info' | 'inherit';
export type CircularProgressSize = 'small' | 'medium' | 'large' | number;

export interface CircularProgressProps {
  /**
   * The component to render as.
   * @default 'span'
   */
  component?: ElementType;
  /**
   * The variant of the progress indicator.
   * @default 'indeterminate'
   */
  variant?: CircularProgressVariant;
  /**
   * The color of the component.
   * @default 'primary'
   */
  color?: CircularProgressColor;
  /**
   * The size of the component.
   * @default 'medium'
   */
  size?: CircularProgressSize;
  /**
   * The value of the progress indicator for determinate variant.
   * Value is between 0-100.
   */
  value?: number;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

const COLOR_STYLES: Record<CircularProgressColor, string> = {
  primary: 'text-nest-primary',
  secondary: 'text-nest-secondary',
  accent: 'text-nest-accent',
  success: 'text-nest-success',
  warning: 'text-nest-warning',
  error: 'text-nest-error',
  info: 'text-nest-info',
  inherit: 'text-inherit',
};

const SIZE_STYLES: Record<'small' | 'medium' | 'large', string> = {
  small: 'h-4 w-4',
  medium: 'h-8 w-8',
  large: 'h-12 w-12',
};

const SIZE_VALUES: Record<'small' | 'medium' | 'large', number> = {
  small: 16,
  medium: 32,
  large: 48,
};

/**
 * CircularProgress component - a circular progress indicator.
 * Follows MUI CircularProgress API conventions.
 *
 * CircularProgress shows a spinning indicator (indeterminate) or
 * a circular progress bar (determinate).
 *
 * @example
 * // Indeterminate (default)
 * <CircularProgress />
 *
 * @example
 * // Determinate
 * <CircularProgress variant="determinate" value={50} />
 *
 * @example
 * // Different sizes
 * <CircularProgress size="small" />
 * <CircularProgress size="medium" />
 * <CircularProgress size="large" />
 */
export const CircularProgress = forwardRef<HTMLSpanElement, CircularProgressProps>(function CircularProgress(
  {
    component = 'span',
    variant = 'indeterminate',
    color = 'primary',
    size = 'medium',
    value = 0,
    className,
    ...props
  }: CircularProgressProps & React.HTMLAttributes<HTMLSpanElement>,
  ref: React.Ref<HTMLSpanElement>
) {
  const Component = component;

  const sizeValue = typeof size === 'number' ? size : SIZE_VALUES[size];
  const sizeClass = typeof size === 'string' ? SIZE_STYLES[size] : '';

  const colorStyle = COLOR_STYLES[color];

  const viewBox = `0 0 ${sizeValue} ${sizeValue}`;

  // Circle configuration
  const center = sizeValue / 2;
  const radius = sizeValue * 0.375;
  const circumference = 2 * Math.PI * radius;
  const progressOffset = circumference - (value / 100) * circumference;

  return (
    <Component
      ref={ref as any}
      className={cn('inline-block', sizeClass, colorStyle, className)}
      role="progressbar"
      aria-valuenow={variant === 'determinate' ? value : undefined}
      {...(props as any)}
    >
      <svg
        className={variant === 'indeterminate' ? 'animate-spin' : ''}
        viewBox={viewBox}
        width={sizeValue}
        height={sizeValue}
        xmlns="http://www.w3.org/2000/svg"
      >
        {/* Background circle */}
        <circle
          className="opacity-25"
          cx={center}
          cy={center}
          r={radius}
          fill="none"
          stroke="currentColor"
          strokeWidth={sizeValue * 0.125}
        />
        {/* Progress circle */}
        {variant === 'determinate' ? (
          <circle
            className="transition-all duration-300 ease-out"
            cx={center}
            cy={center}
            r={radius}
            fill="none"
            stroke="currentColor"
            strokeWidth={sizeValue * 0.125}
            strokeDasharray={circumference}
            strokeDashoffset={progressOffset}
            strokeLinecap="round"
            transform={`rotate(-90 ${center} ${center})`}
          />
        ) : (
          <circle
            className="opacity-75"
            cx={center}
            cy={center}
            r={radius}
            fill="none"
            stroke="currentColor"
            strokeWidth={sizeValue * 0.125}
            strokeLinecap="round"
            strokeDasharray={circumference}
            strokeDashoffset={circumference * 0.25}
            transform={`rotate(-90 ${center} ${center})`}
          />
        )}
      </svg>
    </Component>
  );
});
