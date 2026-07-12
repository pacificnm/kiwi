import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

/**
 * Box component props.
 * Extends HTMLAttributes of the rendered component type.
 */
export interface BoxProps<C extends ElementType = 'div'> {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: C;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: React.ReactNode;
}

/**
 * Box component - a flexible layout primitive.
 * Follows MUI Box API conventions (simplified — no `sx` prop).
 *
 * Box is an escape-hatch component that renders a configurable HTML element
 * with className merging support. Use it when you need a custom element
 * or when other layout components don't fit your needs.
 *
 * @example
 * // Basic usage
 * <Box>Content</Box>
 *
 * @example
 * // Custom element
 * <Box component="section">Section content</Box>
 *
 * @example
 * // With custom styling
 * <Box className="flex items-center gap-2">Flex content</Box>
 */
export const Box = forwardRef(function Box<C extends ElementType = 'div'>(
  {
    className,
    component,
    children,
    ...props
  }: BoxProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children'>,
  ref: React.Ref<Element>
) {
  const Component = component ?? 'div';

  return (
    <Component
      ref={ref as any}
      className={cn(className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
}) as <C extends ElementType = 'div'>(
  props: BoxProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children'> & { ref?: React.Ref<Element> }
) => React.ReactElement | null;
