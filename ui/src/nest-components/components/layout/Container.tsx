import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type ContainerMaxWidth = 'sm' | 'md' | 'lg' | 'xl' | 'xxl' | false;

export interface ContainerProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The maximum width of the container.
   * @default 'lg'
   */
  maxWidth?: ContainerMaxWidth;
  /**
   * If true, removes horizontal padding.
   * @default false
   */
  disableGutters?: boolean;
  /**
   * If true, applies a fixed width based on maxWidth instead of max-width.
   * @default false
   */
  fixed?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: React.ReactNode;
}

const MAX_WIDTH_STYLES: Record<Exclude<ContainerMaxWidth, false>, string> = {
  sm: 'max-w-screen-sm',
  md: 'max-w-screen-md',
  lg: 'max-w-screen-lg',
  xl: 'max-w-screen-xl',
  xxl: 'max-w-screen-2xl',
};

const FIXED_WIDTH_STYLES: Record<Exclude<ContainerMaxWidth, false>, string> = {
  sm: 'w-screen-sm',
  md: 'w-screen-md',
  lg: 'w-screen-lg',
  xl: 'w-screen-xl',
  xxl: 'w-screen-2xl',
};

/**
 * Container component - a centered layout wrapper with max-width.
 * Follows MUI Container API conventions.
 *
 * Container centers content horizontally and constrains its maximum width
 * for better readability on large screens.
 *
 * @example
 * // Default container
 * <Container>
 *   <p>Centered content with max-width</p>
 * </Container>
 *
 * @example
 * // Different max widths
 * <Container maxWidth="sm">Small</Container>
 * <Container maxWidth="md">Medium</Container>
 * <Container maxWidth="xl">Extra large</Container>
 *
 * @example
 * // Full width with no gutters
 * <Container maxWidth={false} disableGutters>
 *   Full width content
 * </Container>
 */
export const Container = forwardRef(function Container(
  {
    className,
    component = 'div',
    maxWidth = 'lg',
    disableGutters = false,
    fixed = false,
    children,
    ...props
  }: ContainerProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component;

  const baseStyles = 'mx-auto w-full';

  const maxWidthStyle = maxWidth !== false ? MAX_WIDTH_STYLES[maxWidth] : 'max-w-full';

  const fixedStyle = fixed && maxWidth !== false ? FIXED_WIDTH_STYLES[maxWidth] : '';

  const gutterStyles = disableGutters ? '' : 'px-4';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, maxWidthStyle, fixedStyle, gutterStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});
