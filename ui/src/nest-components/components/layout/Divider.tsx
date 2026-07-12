import { forwardRef, type ElementType, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type DividerOrientation = 'horizontal' | 'vertical';

export interface DividerProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The orientation of the divider.
   * @default 'horizontal'
   */
  orientation?: DividerOrientation;
  /**
   * If true, the divider takes full width (horizontal) or full height (vertical).
   * @default true
   */
  fullWidth?: boolean;
  /**
   * If true, the divider flexes to accommodate children.
   * @default false
   */
  flexItem?: boolean;
  /**
   * Content to display within the divider (e.g., text divider).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Divider component - a visual separator with optional content.
 * Follows MUI Divider API conventions.
 *
 * Divider renders a horizontal or vertical line, optionally with text content
 * in the middle for labeled sections.
 *
 * @example
 * // Horizontal divider
 * <Divider />
 *
 * @example
 * // Vertical divider
 * <Divider orientation="vertical" />
 *
 * @example
 * // Text divider
 * <Divider>OR</Divider>
 *
 * @example
 * // With icon
 * <Divider><StarIcon /></Divider>
 */
export const Divider = forwardRef(function Divider(
  {
    className,
    component = 'div',
    orientation = 'horizontal',
    fullWidth = true,
    flexItem = false,
    children,
    ...props
  }: DividerProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component;

  const hasChildren = !!children;

  const baseStyles = 'border-nest-border';

  const horizontalStyles = hasChildren
    ? 'flex items-center w-full'
    : 'border-t';

  const verticalStyles = 'border-l h-full';

  const fullWidthStyles = orientation === 'horizontal' && fullWidth ? 'w-full' : '';

  const flexItemStyles = flexItem ? 'flex-shrink-0' : '';

  // Horizontal divider with children (text divider)
  if (orientation === 'horizontal' && hasChildren) {
    return (
      <Component
        ref={ref as any}
        className={cn(baseStyles, horizontalStyles, fullWidthStyles, flexItemStyles, className)}
        role="separator"
        {...(props as any)}
      >
        <span className="flex-1 border-t border-nest-border" />
        <span className="px-4 text-sm text-nest-muted">{children}</span>
        <span className="flex-1 border-t border-nest-border" />
      </Component>
    );
  }

  // Horizontal divider without children
  if (orientation === 'horizontal') {
    return (
      <Component
        ref={ref as any}
        className={cn(baseStyles, horizontalStyles, fullWidthStyles, flexItemStyles, className)}
        role="separator"
        {...(props as any)}
      />
    );
  }

  // Vertical divider
  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, verticalStyles, flexItemStyles, className)}
      role="separator"
      aria-orientation="vertical"
      {...(props as any)}
    />
  );
});
