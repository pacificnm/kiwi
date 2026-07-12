import { forwardRef, type ReactNode, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type ButtonGroupVariant = 'text' | 'outlined' | 'contained';
export type ButtonGroupColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info' | 'inherit';
export type ButtonGroupSize = 'small' | 'medium' | 'large';
export type ButtonGroupOrientation = 'horizontal' | 'vertical';

const VARIANT_STYLES: Record<ButtonGroupVariant, string> = {
  text: '',
  outlined: 'border border-nest-border',
  contained: 'shadow-sm',
};

const COLOR_STYLES: Record<ButtonGroupColor, string> = {
  primary: '[&>button:not(:disabled)]:bg-nest-primary [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-primary/90',
  secondary: '[&>button:not(:disabled)]:bg-nest-secondary [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-secondary/90',
  accent: '[&>button:not(:disabled)]:bg-nest-accent [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-accent/90',
  success: '[&>button:not(:disabled)]:bg-nest-success [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-success/90',
  warning: '[&>button:not(:disabled)]:bg-nest-warning [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-warning/90',
  error: '[&>button:not(:disabled)]:bg-nest-error [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-error/90',
  info: '[&>button:not(:disabled)]:bg-nest-info [&>button:not(:disabled)]:text-white [&>button:not(:disabled)]:hover:bg-nest-info/90',
  inherit: '',
};

const SIZE_STYLES: Record<ButtonGroupSize, string> = {
  small: '[&>button]:text-xs [&>button]:px-2 [&>button]:py-1',
  medium: '[&>button]:text-sm [&>button]:px-3 [&>button]:py-1.5',
  large: '[&>button]:text-base [&>button]:px-4 [&>button]:py-2',
};

const ORIENTATION_STYLES: Record<ButtonGroupOrientation, string> = {
  horizontal: 'flex-row',
  vertical: 'flex-col',
};

export interface ButtonGroupProps {
  /**
   * The variant of the button group.
   * @default 'text'
   */
  variant?: ButtonGroupVariant;
  /**
   * The color of the buttons.
   * @default 'primary'
   */
  color?: ButtonGroupColor;
  /**
   * The size of the buttons.
   * @default 'medium'
   */
  size?: ButtonGroupSize;
  /**
   * The orientation of the button group.
   * @default 'horizontal'
   */
  orientation?: ButtonGroupOrientation;
  /**
   * If true, the button group will take the full width.
   * @default false
   */
  fullWidth?: boolean;
  /**
   * The content of the button group (Button components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
}

/**
 * ButtonGroup component - groups buttons together.
 * Follows MUI ButtonGroup API conventions.
 *
 * ButtonGroup displays multiple buttons as a single component,
 * with shared borders and consistent styling.
 *
 * @example
 * // Default button group
 * <ButtonGroup>
 *   <Button>One</Button>
 *   <Button>Two</Button>
 *   <Button>Three</Button>
 * </ButtonGroup>
 *
 * @example
 * // Contained variant
 * <ButtonGroup variant="contained">
 *   <Button>Left</Button>
 *   <Button>Right</Button>
 * </ButtonGroup>
 *
 * @example
 * // Vertical orientation
 * <ButtonGroup orientation="vertical">
 *   <Button>Top</Button>
 *   <Button>Middle</Button>
 *   <Button>Bottom</Button>
 * </ButtonGroup>
 */
export const ButtonGroup = forwardRef<HTMLDivElement, ButtonGroupProps>(function ButtonGroup(
  {
    variant = 'text',
    color = 'primary',
    size = 'medium',
    orientation = 'horizontal',
    fullWidth = false,
    children,
    className,
    component = 'div',
    ...props
  }: ButtonGroupProps & React.HTMLAttributes<HTMLDivElement>,
  ref: React.Ref<HTMLDivElement>
) {
  const Component = component;
  const variantStyles = VARIANT_STYLES[variant];
  const colorStyles = COLOR_STYLES[color];
  const sizeStyles = SIZE_STYLES[size];
  const orientationStyles = ORIENTATION_STYLES[orientation];

  const baseStyles = 'inline-flex [&>button]:rounded-none [&>button:first-child]:rounded-tl-nest-md [&>button:first-child]:rounded-bl-nest-md [&>button:last-child]:rounded-tr-nest-md [&>button:last-child]:rounded-br-nest-md [&>button:not(:last-child)]:border-r';

  return (
    <Component
      ref={ref as any}
      className={cn(
        baseStyles,
        variantStyles,
        colorStyles,
        sizeStyles,
        orientationStyles,
        fullWidth && 'w-full',
        className
      )}
      role="group"
      {...props}
    >
      {children}
    </Component>
  );
});
