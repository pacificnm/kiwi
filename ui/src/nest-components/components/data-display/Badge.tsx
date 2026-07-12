import { forwardRef, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type BadgeVariant = 'standard' | 'dot';
export type BadgeColor = 'default' | 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info';
export type BadgeAnchorOrigin = {
  vertical?: 'top' | 'bottom';
  horizontal?: 'left' | 'right';
};

export interface BadgeProps {
  /**
   * The component to render as.
   * @default 'span'
   */
  component?: React.ElementType;
  /**
   * The content of the badge.
   * Can be a number (will be capped at max) or any React node.
   */
  badgeContent?: ReactNode;
  /**
   * The color of the badge.
   * @default 'default'
   */
  color?: BadgeColor;
  /**
   * The variant of the badge.
   * @default 'standard'
   */
  variant?: BadgeVariant;
  /**
   * Max value to show when badgeContent is a number.
   * @default 99
   */
  max?: number;
  /**
   * If true, shows the badge even when badgeContent is 0.
   * @default false
   */
  showZero?: boolean;
  /**
   * Anchor position for the badge.
   * @default { vertical: 'top', horizontal: 'right' }
   */
  anchorOrigin?: BadgeAnchorOrigin;
  /**
   * If true, the badge is invisible.
   * @default false
   */
  invisible?: boolean;
  /**
   * The element to wrap the badge around.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes for the badge.
   */
  className?: string;
  /**
   * Additional CSS classes for the badge wrapper.
   */
  wrapperClassName?: string;
  /**
   * Data-testid for the badge wrapper (for testing).
   */
  wrapperDataTestId?: string;
}

const COLOR_STYLES: Record<BadgeColor, string> = {
  default: 'bg-nest-muted text-nest-foreground',
  primary: 'bg-nest-primary text-white',
  secondary: 'bg-nest-secondary text-white',
  accent: 'bg-nest-accent text-white',
  success: 'bg-nest-success text-white',
  warning: 'bg-nest-warning text-white',
  error: 'bg-nest-error text-white',
  info: 'bg-nest-info text-white',
};

/**
 * Badge component - displays a small status or notification badge.
 * Follows MUI Badge API conventions.
 *
 * Badge wraps its children and positions a badge element at the specified anchor.
 *
 * @example
 * // Basic badge
 * <Badge badgeContent={4}>
 *   <Avatar />
 * </Badge>
 *
 * @example
 * // Dot badge
 * <Badge variant="dot" color="error">
 *   <Avatar />
 * </Badge>
 *
 * @example
 * // Badge with max
 * <Badge badgeContent={999} max={99}>
 *   <Avatar />
 * </Badge>
 */
export const Badge = forwardRef<HTMLSpanElement, BadgeProps>(function Badge(
  {
    className,
    component = 'span',
    badgeContent,
    color = 'default',
    variant = 'standard',
    max = 99,
    showZero = false,
    anchorOrigin = { vertical: 'top', horizontal: 'right' },
    invisible = false,
    children,
    wrapperClassName,
    wrapperDataTestId,
    ...props
  }: BadgeProps & React.HTMLAttributes<HTMLSpanElement>,
  ref: React.Ref<HTMLSpanElement>
) {
  const WrapperComponent = component;

  // Determine if badge should be shown
  const isNumber = typeof badgeContent === 'number';
  const shouldShow = !invisible && (
    badgeContent !== undefined &&
    badgeContent !== null &&
    badgeContent !== '' &&
    (showZero || badgeContent !== 0)
  );

  // Format badge content
  const displayContent = isNumber && badgeContent !== undefined && badgeContent !== null
    ? (badgeContent as number) > max
      ? `${max}+`
      : badgeContent
    : badgeContent;

  // Position classes
  const vertical = anchorOrigin.vertical ?? 'top';
  const horizontal = anchorOrigin.horizontal ?? 'right';

  const positionClasses = {
    top: {
      left: 'top-0 left-0 -translate-x-1/2 -translate-y-1/2',
      right: 'top-0 right-0 translate-x-1/2 -translate-y-1/2',
    },
    bottom: {
      left: 'bottom-0 left-0 -translate-x-1/2 translate-y-1/2',
      right: 'bottom-0 right-0 translate-x-1/2 translate-y-1/2',
    },
  };

  const anchorClasses = positionClasses[vertical]?.[horizontal] ?? positionClasses.top.right;

  const baseBadgeStyles = 'flex items-center justify-center rounded-full font-medium whitespace-nowrap';

  const sizeStyles = variant === 'dot'
    ? 'h-2.5 w-2.5 min-w-0 p-0'
    : 'min-w-[1.25rem] h-5 px-1.5 text-xs';

  const colorStyles = COLOR_STYLES[color];

  const invisibleStyles = invisible ? 'invisible' : '';

  return (
    <WrapperComponent
      className={cn('relative inline-flex', wrapperClassName)}
      data-badge-wrapper
      data-testid={wrapperDataTestId}
    >
      {children}
      {shouldShow && (
        <span
          ref={ref as any}
          className={cn(
            'absolute',
            baseBadgeStyles,
            sizeStyles,
            colorStyles,
            anchorClasses,
            invisibleStyles,
            className
          )}
          data-badge
          {...(props as any)}
        >
          {variant === 'dot' ? null : displayContent}
        </span>
      )}
    </WrapperComponent>
  );
});
