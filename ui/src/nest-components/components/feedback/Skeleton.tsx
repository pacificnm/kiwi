import { forwardRef, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export type SkeletonVariant = 'text' | 'circular' | 'rectangular' | 'rounded';
export type SkeletonAnimation = 'pulse' | 'wave';

const VARIANT_STYLES: Record<SkeletonVariant, string> = {
  text: 'h-4 rounded',
  circular: 'rounded-full',
  rectangular: 'rounded-none',
  rounded: 'rounded-nest-md',
};

const ANIMATION_STYLES: Record<SkeletonAnimation, string> = {
  pulse: 'animate-pulse',
  wave: 'animate-pulse',
};

export interface SkeletonProps {
  /**
   * The variant of the skeleton.
   * @default 'text'
   */
  variant?: SkeletonVariant;
  /**
   * The animation to apply.
   * @default 'pulse'
   */
  animation?: SkeletonAnimation | false;
  /**
   * The width of the skeleton.
   */
  width?: string | number;
  /**
   * The height of the skeleton.
   */
  height?: string | number;
  /**
   * The component to render as.
   * @default 'span'
   */
  component?: ElementType;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Skeleton component - a placeholder loading state.
 * Follows MUI Skeleton API conventions.
 *
 * Skeleton shows a simplified version of what will be loaded,
 * using Nest design tokens for the placeholder styling.
 *
 * @example
 * // Text skeleton (default)
 * <Skeleton />
 *
 * @example
 * // Circular avatar skeleton
 * <Skeleton variant="circular" width={40} height={40} />
 *
 * @example
 * // Rectangular image skeleton
 * <Skeleton variant="rectangular" width={200} height={100} />
 *
 * @example
 * // Rounded card skeleton
 * <Skeleton variant="rounded" width={300} height={150} />
 */
export const Skeleton = forwardRef<HTMLElement, SkeletonProps>(function Skeleton(
  {
    variant = 'text',
    animation = 'pulse',
    width,
    height,
    component = 'span',
    className,
    ...props
  }: SkeletonProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLElement>
) {
  const Component = component;
  const variantStyles = VARIANT_STYLES[variant];
  const animationStyles = animation ? ANIMATION_STYLES[animation] : '';

  const baseStyles = 'bg-nest-muted/30 block';

  const inlineStyles: React.CSSProperties = {};
  if (width !== undefined) {
    inlineStyles.width = typeof width === 'number' ? `${width}px` : width;
  }
  if (height !== undefined) {
    inlineStyles.height = typeof height === 'number' ? `${height}px` : height;
  }

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, variantStyles, animationStyles, className)}
      style={inlineStyles}
      {...(props as any)}
    />
  );
});
