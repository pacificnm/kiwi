import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type LinearProgressVariant = 'indeterminate' | 'determinate' | 'buffer';
export type LinearProgressColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info' | 'inherit';

export interface LinearProgressProps {
  /**
   * The component to render as.
   * @default 'span'
   */
  component?: ElementType;
  /**
   * The variant of the progress indicator.
   * @default 'indeterminate'
   */
  variant?: LinearProgressVariant;
  /**
   * The color of the component.
   * @default 'primary'
   */
  color?: LinearProgressColor;
  /**
   * The value of the progress indicator for determinate variant.
   * Value is between 0-100.
   */
  value?: number;
  /**
   * The value of the buffer for buffer variant.
   * Value is between 0-100.
   */
  bufferValue?: number;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

const COLOR_STYLES: Record<LinearProgressColor, string> = {
  primary: 'bg-nest-primary',
  secondary: 'bg-nest-secondary',
  accent: 'bg-nest-accent',
  success: 'bg-nest-success',
  warning: 'bg-nest-warning',
  error: 'bg-nest-error',
  info: 'bg-nest-info',
  inherit: 'bg-inherit',
};

/**
 * LinearProgress component - a linear progress indicator.
 * Follows MUI LinearProgress API conventions.
 *
 * LinearProgress shows a horizontal progress bar (indeterminate) or
 * a linear progress bar (determinate).
 *
 * @example
 * // Indeterminate (default)
 * <LinearProgress />
 *
 * @example
 * // Determinate
 * <LinearProgress variant="determinate" value={50} />
 *
 * @example
 * // Buffer
 * <LinearProgress variant="buffer" value={50} bufferValue={75} />
 */
export const LinearProgress = forwardRef<HTMLSpanElement, LinearProgressProps>(function LinearProgress(
  {
    component = 'span',
    variant = 'indeterminate',
    color = 'primary',
    value = 0,
    bufferValue = 0,
    className,
    ...props
  }: LinearProgressProps & React.HTMLAttributes<HTMLSpanElement>,
  ref: React.Ref<HTMLSpanElement>
) {
  const Component = component;

  const colorStyle = COLOR_STYLES[color];

  const baseStyles = 'relative h-1 w-full rounded-nest-full bg-nest-surface overflow-hidden';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, className)}
      role="progressbar"
      aria-valuenow={variant === 'determinate' || variant === 'buffer' ? value : undefined}
      {...(props as any)}
    >
      {/* Buffer variant: shows both buffer (background) and progress (foreground) */}
      {variant === 'buffer' && (
        <>
          {/* Buffer background */}
          <div
            className={cn('absolute inset-y-0 left-0 rounded-nest-full transition-all duration-300 opacity-30', colorStyle)}
            style={{ width: `${bufferValue}%` }}
            aria-hidden="true"
          />
          {/* Progress foreground */}
          <div
            className={cn('absolute inset-y-0 left-0 rounded-nest-full transition-all duration-300 ease-out', colorStyle)}
            style={{ width: `${value}%` }}
            aria-hidden="true"
          />
        </>
      )}

      {/* Determinate progress bar */}
      {variant === 'determinate' && (
        <div
          className={cn('absolute inset-y-0 left-0 rounded-nest-full transition-all duration-300 ease-out', colorStyle)}
          style={{ width: `${value}%` }}
          aria-hidden="true"
        />
      )}

      {/* Indeterminate animation */}
      {variant === 'indeterminate' && (
        <>
          <div
            className={cn('absolute inset-y-0 left-0 rounded-nest-full animate-[linear-progress-indeterminate1_2.1s_cubic-bezier(0.65,0,0.35,1)_infinite]', colorStyle)}
            aria-hidden="true"
          />
          <div
            className={cn('absolute inset-y-0 left-0 rounded-nest-full animate-[linear-progress-indeterminate2_2.1s_cubic-bezier(0.165,0.84,0.44,1)_infinite_delay-[1.15s]]', colorStyle)}
            aria-hidden="true"
          />
        </>
      )}
    </Component>
  );
});
