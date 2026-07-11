import { forwardRef, type ButtonHTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type IconButtonSize = 'small' | 'medium' | 'large';
export type IconButtonColor =
  | 'default'
  | 'primary'
  | 'secondary'
  | 'accent'
  | 'error'
  | 'success'
  | 'warning'
  | 'info';

export interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  /**
   * The icon element to render.
   */
  children: ReactNode;
  /**
   * The size of the button.
   * @default 'medium'
   */
  size?: IconButtonSize;
  /**
   * The color scheme of the button.
   * @default 'default'
   */
  color?: IconButtonColor;
  /**
   * Full width button.
   * @default false
   */
  fullWidth?: boolean;
  /**
   * Tooltip text (for accessibility when icon-only).
   */
  'aria-label'?: string;
}

const SIZE_STYLES: Record<IconButtonSize, string> = {
  small: 'size-8',
  medium: 'size-10',
  large: 'size-12',
};

const COLOR_STYLES: Record<IconButtonColor, string> = {
  default: 'text-nest-foreground hover:bg-nest-muted/20',
  primary: 'text-nest-primary hover:bg-nest-primary/10',
  secondary: 'text-nest-secondary hover:bg-nest-secondary/10',
  accent: 'text-nest-accent hover:bg-nest-accent/10',
  error: 'text-nest-error hover:bg-nest-error/10',
  success: 'text-nest-success hover:bg-nest-success/10',
  warning: 'text-nest-warning hover:bg-nest-warning/10',
  info: 'text-nest-info hover:bg-nest-info/10',
};

/**
 * Icon button component for icon-only actions.
 * Follows MUI IconButton API conventions.
 *
 * @example
 * // Basic usage
 * <IconButton aria-label="delete">
 *   <TrashIcon />
 * </IconButton>
 *
 * @example
 * // With color
 * <IconButton color="error" aria-label="delete">
 *   <TrashIcon />
 * </IconButton>
 */
export const IconButton = forwardRef<HTMLButtonElement, IconButtonProps>(function IconButton(
  {
    className,
    size = 'medium',
    color = 'default',
    fullWidth = false,
    children,
    ...props
  },
  ref
) {
  const baseStyles =
    'inline-flex items-center justify-center rounded-nest-md transition-all duration-150 focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2';

  const colorStyles = COLOR_STYLES[color];

  const sizeStyles = SIZE_STYLES[size];

  const fullWidthStyles = fullWidth ? 'w-full' : '';

  const disabledStyles = 'disabled:opacity-50 disabled:cursor-not-allowed';

  return (
    <button
      ref={ref}
      className={cn(
        baseStyles,
        colorStyles,
        sizeStyles,
        fullWidthStyles,
        disabledStyles,
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
});
