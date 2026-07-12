import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type StackDirection = 'row' | 'column';
export type StackSpacing = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 8;
export type StackAlign = 'start' | 'center' | 'end' | 'stretch';
export type StackJustify = 'start' | 'center' | 'end' | 'between' | 'around';

export interface StackProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The flex direction of the stack.
   * @default 'column'
   */
  direction?: StackDirection;
  /**
   * The spacing between children (maps to gap-*).
   * @default 2
   */
  spacing?: StackSpacing;
  /**
   * Alignment of children along the cross axis.
   * @default 'stretch'
   */
  align?: StackAlign;
  /**
   * Distribution of children along the main axis.
   * @default 'start'
   */
  justify?: StackJustify;
  /**
   * Whether children should wrap to the next line.
   * @default false
   */
  wrap?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: React.ReactNode;
}

const DIRECTION_STYLES: Record<StackDirection, string> = {
  row: 'flex-row',
  column: 'flex-col',
};

const SPACING_STYLES: Record<StackSpacing, string> = {
  0: 'gap-0',
  1: 'gap-1',
  2: 'gap-2',
  3: 'gap-3',
  4: 'gap-4',
  5: 'gap-5',
  6: 'gap-6',
  8: 'gap-8',
};

const ALIGN_STYLES: Record<StackAlign, string> = {
  start: 'items-start',
  center: 'items-center',
  end: 'items-end',
  stretch: 'items-stretch',
};

const JUSTIFY_STYLES: Record<StackJustify, string> = {
  start: 'justify-start',
  center: 'justify-center',
  end: 'justify-end',
  between: 'justify-between',
  around: 'justify-around',
};

/**
 * Stack component - a flexbox layout primitive for stacking children.
 * Follows MUI Stack API conventions.
 *
 * Stack provides a simpler API for common flex layouts with built-in
 * spacing control via the `spacing` prop.
 *
 * @example
 * // Default column stack
 * <Stack spacing={2}>
 *   <div>Item 1</div>
 *   <div>Item 2</div>
 * </Stack>
 *
 * @example
 * // Horizontal row
 * <Stack direction="row" spacing={4}>
 *   <div>Item 1</div>
 *   <div>Item 2</div>
 * </Stack>
 *
 * @example
 * // Centered and spaced
 * <Stack direction="row" spacing={2} align="center" justify="center">
 *   <div>Centered items</div>
 * </Stack>
 */
export const Stack = forwardRef(function Stack(
  {
    className,
    component = 'div',
    direction = 'column',
    spacing = 2,
    align = 'stretch',
    justify = 'start',
    wrap = false,
    children,
    ...props
  }: StackProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component;

  const baseStyles = 'flex';

  const directionStyles = DIRECTION_STYLES[direction];

  const spacingStyles = SPACING_STYLES[spacing];

  const alignStyles = ALIGN_STYLES[align];

  const justifyStyles = JUSTIFY_STYLES[justify];

  const wrapStyles = wrap ? 'flex-wrap' : '';

  return (
    <Component
      ref={ref as any}
      className={cn(
        baseStyles,
        directionStyles,
        spacingStyles,
        alignStyles,
        justifyStyles,
        wrapStyles,
        className
      )}
      {...(props as any)}
    >
      {children}
    </Component>
  );
}) as React.ForwardRefExoticComponent<
  StackProps & React.HTMLAttributes<HTMLElement> & React.RefAttributes<HTMLElement>
>;
