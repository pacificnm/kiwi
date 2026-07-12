import { forwardRef, type ElementType, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type TypographyVariant =
  | 'h1'
  | 'h2'
  | 'h3'
  | 'h4'
  | 'h5'
  | 'h6'
  | 'subtitle1'
  | 'subtitle2'
  | 'body1'
  | 'body2'
  | 'caption'
  | 'overline';

export type TypographyAlign = 'inherit' | 'left' | 'center' | 'right' | 'justify';

export type TypographyColor =
  | 'primary'
  | 'secondary'
  | 'foreground'
  | 'muted'
  | 'error'
  | 'success'
  | 'warning';

export interface TypographyProps {
  /**
   * The variant of the typography.
   * @default 'body1'
   */
  variant?: TypographyVariant;
  /**
   * The text alignment.
   * @default 'inherit'
   */
  align?: TypographyAlign;
  /**
   * The color of the text.
   * When not specified, some variants (subtitle1, subtitle2, caption) use muted by default.
   * @default 'foreground' (or 'muted' for subtitle1, subtitle2, caption)
   */
  color?: TypographyColor;
  /**
   * Adds a bottom margin for spacing.
   * @default false
   */
  gutterBottom?: boolean;
  /**
   * Truncates text with ellipsis when it overflows.
   * @default false
   */
  noWrap?: boolean;
  /**
   * The component to render as. Defaults to a semantic tag based on variant.
   */
  component?: ElementType;
  /**
   * The content to display.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

const VARIANT_STYLES: Record<TypographyVariant, string> = {
  h1: 'text-4xl font-bold',
  h2: 'text-3xl font-bold',
  h3: 'text-2xl font-semibold',
  h4: 'text-xl font-semibold',
  h5: 'text-lg font-medium',
  h6: 'text-base font-medium',
  subtitle1: 'text-lg text-nest-muted',
  subtitle2: 'text-base text-nest-muted',
  body1: 'text-sm',
  body2: 'text-xs',
  caption: 'text-xs text-nest-muted',
  overline: 'text-xs uppercase tracking-wide',
};

const COLOR_STYLES: Record<TypographyColor, string> = {
  primary: 'text-nest-primary',
  secondary: 'text-nest-secondary',
  foreground: 'text-nest-foreground',
  muted: 'text-nest-muted',
  error: 'text-nest-error',
  success: 'text-nest-success',
  warning: 'text-nest-warning',
};

const ALIGN_STYLES: Record<TypographyAlign, string> = {
  inherit: '',
  left: 'text-left',
  center: 'text-center',
  right: 'text-right',
  justify: 'text-justify',
};

const DEFAULT_COMPONENT: Record<TypographyVariant, ElementType> = {
  h1: 'h1',
  h2: 'h2',
  h3: 'h3',
  h4: 'h4',
  h5: 'h5',
  h6: 'h6',
  subtitle1: 'h6',
  subtitle2: 'h6',
  body1: 'p',
  body2: 'p',
  caption: 'span',
  overline: 'span',
};

/**
 * Typography component for displaying text with consistent styling.
 * Follows MUI Typography API conventions.
 *
 * @example
 * // Basic usage
 * <Typography>Default body text</Typography>
 *
 * @example
 * // Headings
 * <Typography variant="h1">Heading 1</Typography>
 * <Typography variant="h2">Heading 2</Typography>
 *
 * @example
 * // Colors
 * <Typography color="primary">Primary text</Typography>
 * <Typography color="error">Error text</Typography>
 *
 * @example
 * // Truncation and spacing
 * <Typography noWrap>Long text that truncates</Typography>
 * <Typography gutterBottom>Text with bottom margin</Typography>
 */
export const Typography = forwardRef<HTMLElement, TypographyProps>(function Typography(
  {
    className,
    variant = 'body1',
    align = 'inherit',
    color,
    gutterBottom = false,
    noWrap = false,
    component,
    children,
    ...props
  },
  ref
) {
  const Component = component ?? DEFAULT_COMPONENT[variant];

  const baseStyles = 'font-body';

  const variantStyles = VARIANT_STYLES[variant];

  // Only apply explicit color if provided; variants like subtitle1/subtitle2/caption
  // already include text-nest-muted in their variant styles
  const colorStyles = color ? COLOR_STYLES[color] : '';

  const alignStyles = ALIGN_STYLES[align];

  const gutterBottomStyles = gutterBottom ? 'mb-2' : '';

  const noWrapStyles = noWrap ? 'truncate' : '';

  return (
    <Component
      ref={ref as any}
      className={cn(
        baseStyles,
        variantStyles,
        colorStyles,
        alignStyles,
        gutterBottomStyles,
        noWrapStyles,
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
});
