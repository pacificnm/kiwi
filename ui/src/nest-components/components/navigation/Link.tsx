import { forwardRef } from 'react';
import { cn } from '../../lib/cn';

export type LinkUnderline = 'none' | 'hover' | 'always';
export type LinkColor = 'primary' | 'inherit';

export interface LinkProps extends React.AnchorHTMLAttributes<HTMLAnchorElement> {
  /**
   * The component to render as.
   * @default 'a'
   */
  component?: React.ElementType;
  /**
   * When true, the link will have an underline.
   * @default 'hover'
   */
  underline?: LinkUnderline;
  /**
   * The color of the link.
   * @default 'primary'
   */
  color?: LinkColor;
  /**
   * If true, the link will open in a new tab.
   * Sets target="_blank" and rel="noopener noreferrer".
   * @default false
   */
  external?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

const UNDERLINE_STYLES: Record<LinkUnderline, string> = {
  none: 'no-underline',
  hover: 'underline hover:underline',
  always: 'underline',
};

const COLOR_STYLES: Record<LinkColor, string> = {
  primary: 'text-nest-primary hover:text-nest-primary/80',
  inherit: 'text-inherit hover:text-inherit/80',
};

/**
 * Link component - a styled anchor element.
 * Follows MUI Link API conventions.
 *
 * Link provides a consistent styling for anchor elements with optional
 * underline and color variants.
 *
 * @example
 * // Basic link
 * <Link href="/page">Click me</Link>
 *
 * @example
 * // No underline
 * <Link href="/page" underline="none">No underline</Link>
 *
 * @example
 * // External link
 * <Link href="https://example.com" external>External</Link>
 */
export const Link = forwardRef<HTMLAnchorElement, LinkProps>(function Link(
  {
    className,
    component,
    underline = 'hover',
    color = 'primary',
    external = false,
    href,
    children,
    ...props
  }: LinkProps,
  ref: React.Ref<HTMLAnchorElement>
) {
  const Component = component ?? 'a';

  const baseStyles = 'cursor-pointer font-body transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2 rounded-nest-sm';

  const underlineStyles = UNDERLINE_STYLES[underline];

  const colorStyles = COLOR_STYLES[color];

  const externalProps = external
    ? {
        target: '_blank',
        rel: 'noopener noreferrer',
        ...props,
      }
    : props;

  return (
    <Component
      ref={ref as any}
      href={href}
      className={cn(baseStyles, underlineStyles, colorStyles, className)}
      {...(externalProps as any)}
    >
      {children}
    </Component>
  );
});
