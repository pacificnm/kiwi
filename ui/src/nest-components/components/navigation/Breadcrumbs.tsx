import { forwardRef, type ReactNode, type ElementType } from 'react';
import { cn } from '../../lib/cn';
import { ChevronRight } from 'lucide-react';

export interface BreadcrumbsProps {
  /**
   * The component to render as for the root element.
   * @default 'nav'
   */
  component?: ElementType;
  /**
   * The separator to display between breadcrumb items.
   * @default <ChevronRight />
   */
  separator?: ReactNode;
  /**
   * The maximum number of breadcrumb items to display.
   * If exceeded, items are collapsed with a more indicator.
   * @default 0 (no collapsing)
   */
  maxItems?: number;
  /**
   * The items to display in the breadcrumb.
   */
  items: BreadcrumbItem[];
  /**
   * Aria-label for the nav element.
   * @default 'breadcrumb'
   */
  ariaLabel?: string;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

export interface BreadcrumbItem {
  /**
   * The label to display for the breadcrumb item.
   */
  label: ReactNode;
  /**
   * The href for the breadcrumb item.
   * If not provided, the item is rendered as plain text.
   */
  href?: string;
  /**
   * If true, the item is the current page and is not clickable.
   */
  current?: boolean;
}

/**
 * Breadcrumbs component - displays a navigation path.
 * Follows MUI Breadcrumbs API conventions.
 *
 * Breadcrumbs show the navigation path to the current page,
 * allowing users to navigate back to parent pages.
 *
 * @example
 * // Basic breadcrumbs
 * <Breadcrumbs items={[
 *   { label: 'Home', href: '/' },
 *   { label: 'Products', href: '/products' },
 *   { label: 'Electronics', current: true }
 * ]} />
 *
 * @example
 * // Custom separator
 * <Breadcrumbs items={items} separator="/" />
 */
export const Breadcrumbs = forwardRef<HTMLElement, BreadcrumbsProps>(function Breadcrumbs(
  {
    component = 'nav',
    separator = <ChevronRight className="h-4 w-4 text-nest-muted" data-testid="separator-icon" />,
    maxItems = 0,
    items,
    ariaLabel = 'breadcrumb',
    className,
    ...props
  }: BreadcrumbsProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component;

  const baseStyles = 'flex items-center gap-1 text-sm';

  // Handle maxItems collapsing
  let displayItems = items;
  if (maxItems > 0 && items.length > maxItems) {
    const itemsToShow = items.slice(-maxItems + 1);
    const collapsedCount = items.length - maxItems + 1;
    displayItems = [{ label: `+${collapsedCount}`, href: undefined, current: false }, ...itemsToShow];
  }

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, className)}
      aria-label={ariaLabel}
      {...(props as any)}
    >
      <ol className="flex items-center gap-1">
        {displayItems.map((item, index) => (
          <li key={index} className="flex items-center gap-1">
            {index > 0 && (
              <span className="text-nest-muted" aria-hidden="true">
                {separator}
              </span>
            )}
            {item.href && !item.current ? (
              <a
                href={item.href}
                className="text-nest-primary hover:text-nest-primary/80 hover:underline focus:outline-none focus:ring-2 focus:ring-nest-primary/50 rounded-nest-sm"
              >
                {item.label}
              </a>
            ) : (
              <span
                className={item.current ? 'font-medium text-nest-foreground' : 'text-nest-muted'}
                aria-current={item.current ? 'page' : undefined}
              >
                {item.label}
              </span>
            )}
          </li>
        ))}
      </ol>
    </Component>
  );
});
