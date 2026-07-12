import { forwardRef } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';
import { ChevronLeft, ChevronRight, ChevronsLeft, ChevronsRight } from 'lucide-react';

export type PaginationSize = 'small' | 'medium' | 'large';
export type PaginationColor = 'primary' | 'secondary' | 'accent' | 'success' | 'error';

const SIZE_STYLES: Record<PaginationSize, string> = {
  small: 'text-xs',
  medium: 'text-sm',
  large: 'text-base',
};

const COLOR_STYLES: Record<PaginationColor, string> = {
  primary: 'bg-nest-primary text-white hover:bg-nest-primary/90',
  secondary: 'bg-nest-secondary text-white hover:bg-nest-secondary/90',
  accent: 'bg-nest-accent text-white hover:bg-nest-accent/90',
  success: 'bg-nest-success text-white hover:bg-nest-success/90',
  error: 'bg-nest-error text-white hover:bg-nest-error/90',
};

export interface PaginationProps {
  /**
   * Total number of pages.
   */
  count: number;
  /**
   * Current page number (1-indexed).
   */
  page?: number;
  /**
   * Default page number for uncontrolled pagination.
   */
  defaultPage?: number;
  /**
   * Callback fired when the page changes.
   */
  onChange?: (event: React.ChangeEvent<unknown>, page: number) => void;
  /**
   * Number of sibling pages to show on each side of current page.
   * @default 1
   */
  siblingCount?: number;
  /**
   * Number of boundary pages to show at start and end.
   * @default 1
   */
  boundaryCount?: number;
  /**
   * If true, the pagination is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, hide the first and last page buttons.
   * @default false
   */
  hideFirstLast?: boolean;
  /**
   * If true, hide the previous and next page buttons.
   * @default false
   */
  hidePrevNext?: boolean;
  /**
   * The size of the pagination.
   * @default 'medium'
   */
  size?: PaginationSize;
  /**
   * The color of the pagination.
   * @default 'primary'
   */
  color?: PaginationColor;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Pagination component - navigate between pages of content.
 * Follows MUI Pagination API conventions.
 *
 * Pagination allows users to navigate through paginated data.
 *
 * @example
 * // Basic pagination
 * <Pagination count={10} page={page} onChange={(e, p) => setPage(p)} />
 *
 * @example
 * // Uncontrolled
 * <Pagination count={10} defaultPage={1} />
 *
 * @example
 * // With custom siblings
 * <Pagination count={20} siblingCount={2} boundaryCount={2} />
 */
export const Pagination = forwardRef<HTMLDivElement, PaginationProps>(function Pagination(
  {
    count,
    page: pageProp,
    defaultPage = 1,
    onChange,
    siblingCount = 1,
    boundaryCount = 1,
    disabled = false,
    hideFirstLast = false,
    hidePrevNext = false,
    size = 'medium',
    color = 'primary',
    className,
    ...props
  }: PaginationProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [page, setPage] = useControllableState<number>({
    value: pageProp,
    defaultValue: defaultPage,
    onChange: (value) => {
      onChange?.({ target: { value } } as unknown as React.ChangeEvent<unknown>, value);
    },
  });

  const sizeStyles = SIZE_STYLES[size];
  const colorStyles = COLOR_STYLES[color];

  const handleChange = (newPage: number) => {
    if (disabled || newPage < 1 || newPage > count) return;
    setPage(newPage);
  };

  // Generate page items to display
  const getPageItems = () => {
    const items = new Set<number>();

    // Left boundary pages
    for (let i = 1; i <= Math.min(boundaryCount, count); i++) {
      items.add(i);
    }

    // Siblings before current page
    for (let i = page - siblingCount; i < page; i++) {
      if (i >= 1) items.add(i);
    }

    // Current page
    items.add(page);

    // Siblings after current page
    for (let i = page + 1; i <= page + siblingCount; i++) {
      if (i <= count) items.add(i);
    }

    // Right boundary pages
    for (let i = Math.max(count - boundaryCount + 1, 1); i <= count; i++) {
      items.add(i);
    }

    // Convert to sorted array
    const sortedItems = Array.from(items).sort((a, b) => a - b);

    // Add ellipses
    const result: (number | 'ellipsis')[] = [];
    let prev: number | null = null;

    for (const item of sortedItems) {
      if (prev !== null && item - prev > 1) {
        result.push('ellipsis');
      }
      result.push(item);
      prev = item;
    }

    return result;
  };

  const pageItems = getPageItems();
  const isFirstPage = page <= 1;
  const isLastPage = page >= count;

  const baseStyles = cn(
    'inline-flex items-center gap-1',
    sizeStyles,
    disabled && 'opacity-50 pointer-events-none',
    className
  );

  const buttonBaseStyles = cn(
    'min-w-8 h-8 flex items-center justify-center rounded-nest-md',
    'border border-nest-border',
    'bg-nest-surface',
    'text-nest-foreground',
    'transition-colors duration-150',
    'hover:bg-nest-muted',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-1'
  );

  const activeButtonStyles = cn(
    buttonBaseStyles,
    colorStyles,
    'border-transparent'
  );

  return (
    <nav ref={ref as any} className={baseStyles} role="navigation" aria-label="Pagination" {...props}>
      {/* First page button */}
      {!hideFirstLast && (
        <button
          type="button"
          onClick={() => handleChange(1)}
          disabled={isFirstPage || disabled}
          className={cn(buttonBaseStyles, 'p-1', isFirstPage && 'opacity-50 cursor-not-allowed')}
          aria-label="Go to first page"
        >
          <ChevronsLeft className="size-4" />
        </button>
      )}

      {/* Previous page button */}
      {!hidePrevNext && (
        <button
          type="button"
          onClick={() => handleChange(page - 1)}
          disabled={isFirstPage || disabled}
          className={cn(buttonBaseStyles, 'p-1', isFirstPage && 'opacity-50 cursor-not-allowed')}
          aria-label="Go to previous page"
        >
          <ChevronLeft className="size-4" />
        </button>
      )}

      {/* Page numbers */}
      {pageItems.map((item, index) => (
        item === 'ellipsis' ? (
          <span
            key={`ellipsis-${index}`}
            className={cn(buttonBaseStyles, 'cursor-default')}
            aria-hidden="true"
          >
            …
          </span>
        ) : (
          <button
            key={item}
            type="button"
            onClick={() => handleChange(item)}
            disabled={disabled}
            className={cn(
              buttonBaseStyles,
              item === page && activeButtonStyles,
              item === page && 'font-medium'
            )}
            aria-label={`Page ${item}`}
            aria-current={item === page ? 'page' : undefined}
          >
            {item}
          </button>
        )
      ))}

      {/* Next page button */}
      {!hidePrevNext && (
        <button
          type="button"
          onClick={() => handleChange(page + 1)}
          disabled={isLastPage || disabled}
          className={cn(buttonBaseStyles, 'p-1', isLastPage && 'opacity-50 cursor-not-allowed')}
          aria-label="Go to next page"
        >
          <ChevronRight className="size-4" />
        </button>
      )}

      {/* Last page button */}
      {!hideFirstLast && (
        <button
          type="button"
          onClick={() => handleChange(count)}
          disabled={isLastPage || disabled}
          className={cn(buttonBaseStyles, 'p-1', isLastPage && 'opacity-50 cursor-not-allowed')}
          aria-label="Go to last page"
        >
          <ChevronsRight className="size-4" />
        </button>
      )}
    </nav>
  );
});
