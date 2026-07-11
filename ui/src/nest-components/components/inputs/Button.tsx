import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type ButtonVariant = 'text' | 'outlined' | 'contained';
export type ButtonColor = 'primary' | 'secondary' | 'accent' | 'error' | 'success' | 'warning';
export type ButtonSize = 'small' | 'medium' | 'large';

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /**
   * The visual style of the button.
   * @default 'contained'
   */
  variant?: ButtonVariant;
  /**
   * The color scheme of the button.
   * @default 'primary'
   */
  color?: ButtonColor;
  /**
   * The size of the button.
   * @default 'medium'
   */
  size?: ButtonSize;
  /**
   * Icon to display before the label.
   */
  startIcon?: ReactNode;
  /**
   * Icon to display after the label.
   */
  endIcon?: ReactNode;
  /**
   * Shows a loading spinner and disables the button.
   * @default false
   */
  loading?: boolean;
  /**
   * Full width button.
   * @default false
   */
  fullWidth?: boolean;
}

const VARIANT_STYLES: Record<ButtonVariant, Record<ButtonColor, string>> = {
  contained: {
    primary: 'bg-nest-primary text-white hover:bg-nest-primary/90',
    secondary: 'bg-nest-secondary text-white hover:bg-nest-secondary/90',
    accent: 'bg-nest-accent text-white hover:bg-nest-accent/90',
    error: 'bg-nest-error text-white hover:bg-nest-error/90',
    success: 'bg-nest-success text-white hover:bg-nest-success/90',
    warning: 'bg-nest-warning text-white hover:bg-nest-warning/90',
  },
  outlined: {
    primary: 'border border-nest-primary text-nest-primary hover:bg-nest-primary/10',
    secondary: 'border border-nest-secondary text-nest-secondary hover:bg-nest-secondary/10',
    accent: 'border border-nest-accent text-nest-accent hover:bg-nest-accent/10',
    error: 'border border-nest-error text-nest-error hover:bg-nest-error/10',
    success: 'border border-nest-success text-nest-success hover:bg-nest-success/10',
    warning: 'border border-nest-warning text-nest-warning hover:bg-nest-warning/10',
  },
  text: {
    primary: 'text-nest-primary hover:bg-nest-primary/10',
    secondary: 'text-nest-secondary hover:bg-nest-secondary/10',
    accent: 'text-nest-accent hover:bg-nest-accent/10',
    error: 'text-nest-error hover:bg-nest-error/10',
    success: 'text-nest-success hover:bg-nest-success/10',
    warning: 'text-nest-warning hover:bg-nest-warning/10',
  },
};

const SIZE_STYLES: Record<ButtonSize, string> = {
  small: 'h-8 px-3 text-xs gap-1',
  medium: 'h-10 px-4 text-sm gap-1.5',
  large: 'h-12 px-6 text-base gap-2',
};

const LOADING_SPINNER_SIZES: Record<ButtonSize, string> = {
  small: 'size-3',
  medium: 'size-4',
  large: 'size-5',
};

/**
 * Button component for user actions.
 * Follows MUI Button API conventions.
 *
 * @example
 * // Basic usage
 * <Button>Click me</Button>
 *
 * @example
 * // Variants
 * <Button variant="contained">Contained</Button>
 * <Button variant="outlined">Outlined</Button>
 * <Button variant="text">Text</Button>
 *
 * @example
 * // With icon
 * <Button startIcon={<SaveIcon />}>Save</Button>
 *
 * @example
 * // Loading state
 * <Button loading>Loading...</Button>
 */
export const Button = forwardRef<HTMLButtonElement, ButtonProps>(function Button(
  {
    className,
    variant = 'contained',
    color = 'primary',
    size = 'medium',
    startIcon,
    endIcon,
    loading = false,
    fullWidth = false,
    disabled,
    children,
    ...props
  },
  ref
) {
  const baseStyles =
    'inline-flex items-center justify-center font-nest-body rounded-nest-md transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2';

  const variantStyles = VARIANT_STYLES[variant][color];

  const disabledStyles = 'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none';

  const fullWidthStyles = fullWidth ? 'w-full' : '';

  const sizeStyles = SIZE_STYLES[size];

  return (
    <button
      ref={ref}
      className={cn(baseStyles, variantStyles, disabledStyles, fullWidthStyles, sizeStyles, className)}
      disabled={disabled || loading}
      {...props}
    >
      {loading ? (
        <svg
          className={cn('animate-spin', LOADING_SPINNER_SIZES[size])}
          xmlns="http://www.w3.org/2000/svg"
          fill="none"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <circle
            className="opacity-25"
            cx="12"
            cy="12"
            r="10"
            stroke="currentColor"
            strokeWidth="4"
          />
          <path
            className="opacity-75"
            fill="currentColor"
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
      ) : (
        startIcon && (
          <span className="flex shrink-0 items-center justify-center">{startIcon}</span>
        )
      )}
      {children}
      {endIcon && (
        <span className="flex shrink-0 items-center justify-center">{endIcon}</span>
      )}
    </button>
  );
});
