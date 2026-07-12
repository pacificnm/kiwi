import { forwardRef, type ReactNode, useState, type ImgHTMLAttributes } from 'react';
import { cn } from '../../lib/cn';

export type AvatarVariant = 'circular' | 'rounded' | 'square';
export type AvatarSize = 'small' | 'medium' | 'large';

export interface AvatarProps<C extends React.ElementType = 'div'> {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: C;
  /**
   * The image URL for the avatar.
   * If not provided or if the image fails to load, children will be rendered as fallback.
   */
  src?: string;
  /**
   * Alt text for the image.
   */
  alt?: string;
  /**
   * The variant of the avatar.
   * @default 'circular'
   */
  variant?: AvatarVariant;
  /**
   * The size of the avatar.
   * @default 'medium'
   */
  size?: AvatarSize;
  /**
   * Children to render as fallback when no src or image fails to load.
   * Typically initials or an icon.
   */
  children?: ReactNode;
  /**
   * Image props (when src is provided).
   */
  imgProps?: ImgHTMLAttributes<HTMLImageElement>;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

const SIZE_STYLES: Record<AvatarSize, string> = {
  small: 'h-8 w-8 text-xs',
  medium: 'h-10 w-10 text-sm',
  large: 'h-12 w-12 text-base',
};

const VARIANT_STYLES: Record<AvatarVariant, string> = {
  circular: 'rounded-full',
  rounded: 'rounded-nest-md',
  square: 'rounded-none',
};

/**
 * Avatar component - represents a person or entity with an image or fallback.
 * Follows MUI Avatar API conventions.
 *
 * Avatar displays an image if provided, otherwise renders children (initials or icon)
 * as a fallback on a muted background.
 *
 * @example
 * // Avatar with image
 * <Avatar src="/user.jpg" alt="User" />
 *
 * @example
 * // Avatar with initials
 * <Avatar>JD</Avatar>
 *
 * @example
 * // Avatar with icon
 * <Avatar><UserIcon /></Avatar>
 *
 * @example
 * // Different sizes
 * <Avatar size="small">S</Avatar>
 * <Avatar size="medium">M</Avatar>
 * <Avatar size="large">L</Avatar>
 */
export const Avatar = forwardRef(function Avatar<C extends React.ElementType = 'div'>(
  {
    className,
    component,
    src,
    alt = '',
    variant = 'circular',
    size = 'medium',
    children,
    imgProps,
    ...props
  }: AvatarProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children' | 'src' | 'alt'>,
  ref: React.Ref<Element>
) {
  const Component = component ?? 'div';
  const [imageError, setImageError] = useState(false);

  const baseStyles = 'flex shrink-0 items-center justify-center overflow-hidden bg-nest-muted text-nest-foreground font-medium';

  const sizeStyles = SIZE_STYLES[size];

  const variantStyles = VARIANT_STYLES[variant];

  const showImage = src && !imageError;

  const handleImageError = () => {
    setImageError(true);
    imgProps?.onError?.({} as React.SyntheticEvent<HTMLImageElement>);
  };

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, sizeStyles, variantStyles, className)}
      {...(props as any)}
    >
      {showImage ? (
        <img
          src={src}
          alt={alt}
          className="h-full w-full object-cover"
          onError={handleImageError}
          {...imgProps}
        />
      ) : (
        children
      )}
    </Component>
  );
}) as <C extends React.ElementType = 'div'>(
  props: AvatarProps<C> & Omit<React.ComponentPropsWithoutRef<C>, 'className' | 'children' | 'src' | 'alt'> & { ref?: React.Ref<Element> }
) => React.ReactElement | null;
