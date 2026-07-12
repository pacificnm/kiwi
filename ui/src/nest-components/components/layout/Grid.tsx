import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type GridSpacing = 0 | 1 | 2 | 3 | 4 | 5 | 6 | 8;

export interface GridSize {
  xs?: number | 'auto';
  sm?: number | 'auto';
  md?: number | 'auto';
  lg?: number | 'auto';
}

export interface GridProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * If true, the component becomes a grid container.
   * @default false
   */
  container?: boolean;
  /**
   * Number of columns in the grid (only used when container=true).
   * @default 12
   */
  columns?: number;
  /**
   * Spacing between grid items (maps to gap-*).
   */
  spacing?: GridSpacing;
  /**
   * Size of the grid item (number of columns to span).
   * Can be a number for all breakpoints or an object for responsive sizing.
   */
  size?: number | 'auto' | GridSize;
  /**
   * Offset from the left side (number of empty columns).
   */
  offset?: number | GridSize;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: React.ReactNode;
}

const SPACING_STYLES: Record<GridSpacing, string> = {
  0: 'gap-0',
  1: 'gap-1',
  2: 'gap-2',
  3: 'gap-3',
  4: 'gap-4',
  5: 'gap-5',
  6: 'gap-6',
  8: 'gap-8',
};

// Static class strings for column spans (Tailwind can't parse dynamic classes)
const COL_SPAN_CLASSES: Record<number, string> = {
  1: 'col-span-1',
  2: 'col-span-2',
  3: 'col-span-3',
  4: 'col-span-4',
  5: 'col-span-5',
  6: 'col-span-6',
  7: 'col-span-7',
  8: 'col-span-8',
  9: 'col-span-9',
  10: 'col-span-10',
  11: 'col-span-11',
  12: 'col-span-12',
};

const COL_START_CLASSES: Record<number, string> = {
  1: 'col-start-1',
  2: 'col-start-2',
  3: 'col-start-3',
  4: 'col-start-4',
  5: 'col-start-5',
  6: 'col-start-6',
  7: 'col-start-7',
  8: 'col-start-8',
  9: 'col-start-9',
  10: 'col-start-10',
  11: 'col-start-11',
  12: 'col-start-12',
  13: 'col-start-13',
};

/**
 * Grid component - a CSS grid layout component.
 * Follows MUI Grid v2 API conventions.
 *
 * Grid can be used as a container (grid wrapper) or as an item (grid cell).
 * When container=true, it creates a 12-column grid. When size is specified,
 * the component becomes a grid item spanning the specified columns.
 *
 * @example
 * // Grid container with items
 * <Grid container spacing={2}>
 *   <Grid size={6}>Half width</Grid>
 *   <Grid size={6}>Half width</Grid>
 * </Grid>
 *
 * @example
 * // Responsive sizing
 * <Grid container spacing={2}>
 *   <Grid size={{ xs: 12, md: 6 }}>Full on mobile, half on tablet+</Grid>
 * </Grid>
 *
 * @example
 * // With offset
 * <Grid container spacing={2}>
 *   <Grid size={3}>Quarter</Grid>
 *   <Grid size={6} offset={3}>Three quarters with offset</Grid>
 * </Grid>
 */
export const Grid = forwardRef(function Grid(
  {
    className,
    component,
    container = false,
    columns = 12,
    spacing = 0,
    size,
    offset,
    children,
    ...props
  }: GridProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component ?? (container ? 'div' : 'div');

  const baseStyles = container ? 'grid' : '';

  // Container styles
  const columnsStyles = container ? `grid-cols-${columns}` : '';
  const spacingStyles = container && spacing !== undefined ? SPACING_STYLES[spacing] : '';

  // Item styles based on size
  let sizeStyles = '';
  let offsetStyles = '';

  if (size !== undefined) {
    if (size === 'auto') {
      sizeStyles = 'col-auto';
    } else if (typeof size === 'number') {
      sizeStyles = COL_SPAN_CLASSES[size] || '';
    } else if (typeof size === 'object') {
      // Responsive sizing
      const sizeClasses: string[] = [];
      if (size.xs !== undefined) {
        sizeClasses.push(size.xs === 'auto' ? 'col-auto' : COL_SPAN_CLASSES[size.xs as number] || '');
      }
      if (size.sm !== undefined) {
        sizeClasses.push(size.sm === 'auto' ? 'sm:col-auto' : `sm:${COL_SPAN_CLASSES[size.sm as number] || ''}`);
      }
      if (size.md !== undefined) {
        sizeClasses.push(size.md === 'auto' ? 'md:col-auto' : `md:${COL_SPAN_CLASSES[size.md as number] || ''}`);
      }
      if (size.lg !== undefined) {
        sizeClasses.push(size.lg === 'auto' ? 'lg:col-auto' : `lg:${COL_SPAN_CLASSES[size.lg as number] || ''}`);
      }
      sizeStyles = sizeClasses.join(' ');
    }
  }

  // Offset styles
  if (offset !== undefined) {
    if (typeof offset === 'number') {
      offsetStyles = COL_START_CLASSES[offset + 1] || '';
    } else if (typeof offset === 'object') {
      const offsetClasses: string[] = [];
      if (offset.xs !== undefined) {
        offsetClasses.push(COL_START_CLASSES[(offset.xs as number) + 1] || '');
      }
      if (offset.sm !== undefined) {
        offsetClasses.push(`sm:${COL_START_CLASSES[(offset.sm as number) + 1] || ''}`);
      }
      if (offset.md !== undefined) {
        offsetClasses.push(`md:${COL_START_CLASSES[(offset.md as number) + 1] || ''}`);
      }
      if (offset.lg !== undefined) {
        offsetClasses.push(`lg:${COL_START_CLASSES[(offset.lg as number) + 1] || ''}`);
      }
      offsetStyles = offsetClasses.join(' ');
    }
  }

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, columnsStyles, spacingStyles, sizeStyles, offsetStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});
