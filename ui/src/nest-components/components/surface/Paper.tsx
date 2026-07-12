import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type PaperElevation = 0 | 1 | 2 | 3 | 4;
export type PaperVariant = 'elevation' | 'outlined';

export interface PaperProps<C extends ElementType = 'div'> {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: C;
  /**
   * Shadow depth, from 0 (no shadow) to 4 (largest shadow).
   * @default 1
   */
  elevation?: PaperElevation;
  /**
   * Variant of the paper.
   * @default 'elevation'
   */
  variant?: PaperVariant;
  /**
   * If true, removes border radius.
   * @default false
   */
  square?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: React.ReactNode;
}

const ELEVATION_STYLES: Record<PaperElevation, string> = {
  0: 'shadow-none',
  1: 'shadow-sm',
  2: 'shadow',
  3: 'shadow-md',
  4: 'shadow-lg',
};

/**
 * Paper component - a surface container with elevation.
 * Follows MUI Paper API conventions.
 *
 * Paper represents a sheet of paper on top of a surface, with optional
 * shadow (elevation) or border (outlined variant).
 *
 * @example
 * // Default paper
 * <Paper>
 *   <p>Content on a surface</p>
 * </Paper>
 *
 * @example
 * // Different elevations
 * <Paper elevation={0}>No shadow</Paper>
 * <Paper elevation={1}>Small shadow (default)</Paper>
 * <Paper elevation={4}>Large shadow</Paper>
 *
 * @example
 * // Outlined variant
 * <Paper variant="outlined">Border instead of shadow</Paper>
 *
 * @example
 * // Square corners
 * <Paper square>Square corners</Paper>
 */
export const Paper = forwardRef(function Paper<C extends ElementType = 'div'>(
  {
    className,
    component,
    elevation = 1,
    variant = 'elevation',
    square = false,
    children,
    ...props
  }: PaperProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children'>,
  ref: React.Ref<Element>
) {
  const Component = component ?? 'div';

  const baseStyles = 'bg-nest-surface text-nest-foreground';

  const elevationStyles = variant === 'elevation' ? ELEVATION_STYLES[elevation] : '';

  const outlinedStyles = variant === 'outlined' ? 'border border-nest-border' : '';

  const radiusStyles = square ? 'rounded-none' : 'rounded-nest-md';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, elevationStyles, outlinedStyles, radiusStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
}) as <C extends ElementType = 'div'>(
  props: PaperProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children'> & { ref?: React.Ref<Element> }
) => React.ReactElement | null;
