import { forwardRef, type ElementType, type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import { Paper } from './Paper';

export interface CardProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * Shadow depth, from 0 (no shadow) to 4 (largest shadow).
   * @default 1
   */
  elevation?: 0 | 1 | 2 | 3 | 4;
  /**
   * Variant of the paper.
   * @default 'elevation'
   */
  variant?: 'elevation' | 'outlined';
  /**
   * If true, removes border radius.
   * @default false
   */
  square?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: ReactNode;
}

export interface CardHeaderProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The avatar element to display before the title.
   */
  avatar?: ReactNode;
  /**
   * The action element to display after the title.
   */
  action?: ReactNode;
  /**
   * The title content.
   */
  title?: ReactNode;
  /**
   * The subtitle content.
   */
  subheader?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

export interface CardContentProps {
  /**
   * The component to render as.
   * @default 'div'
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

export interface CardActionsProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The content to display (typically buttons).
   */
  children?: ReactNode;
  /**
   * If true, disables spacing between actions.
   * @default false
   */
  disableSpacing?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

export interface CardMediaProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The image URL to display (for img component).
   */
  image?: string;
  /**
   * The alt text for the image.
   */
  alt?: string;
  /**
   * The title text for the image.
   */
  title?: string;
  /**
   * CSS applied to the root element.
   */
  className?: string;
  /**
   * Override the height of the media.
   * @default '140px'
   */
  height?: string;
}

/**
 * Card component - a container for related content.
 * Follows MUI Card API conventions.
 *
 * Card is a surface container that groups related content with optional
 * header, media, content, and actions sections.
 *
 * @example
 * // Basic card
 * <Card>
 *   <CardContent>
 *     <p>Card content</p>
 *   </CardContent>
 * </Card>
 *
 * @example
 * // Card with header and actions
 * <Card>
 *   <CardHeader title="Title" subheader="Subtitle" />
 *   <CardContent>
 *     <p>Card content</p>
 *   </CardContent>
 *   <CardActions>
 *     <Button>Learn more</Button>
 *   </CardActions>
 * </Card>
 */
export const Card = forwardRef<HTMLElement, CardProps & React.HTMLAttributes<HTMLElement>>(function Card(
  {
    className,
    component = 'div',
    elevation = 1,
    variant = 'elevation',
    square = false,
    children,
    ...props
  },
  ref
) {
  return (
    <Paper
      ref={ref as any}
      component={component}
      className={cn('overflow-hidden', className)}
      elevation={elevation}
      variant={variant}
      square={square}
      {...(props as any)}
    >
      {children}
    </Paper>
  );
});

/**
 * CardHeader component - the header section of a Card.
 *
 * Displays an optional avatar, title, subheader, and action in a flex row.
 */
export const CardHeader = forwardRef<HTMLElement, CardHeaderProps & Omit<React.HTMLAttributes<HTMLElement>, 'title'>>(function CardHeader(
  {
    className,
    component = 'div',
    avatar,
    action,
    title,
    subheader,
    ...props
  },
  ref
) {
  const Component = component;

  return (
    <Component
      ref={ref as any}
      className={cn('flex items-start gap-3 p-4', className)}
      {...(props as any)}
    >
      {avatar && <span className="shrink-0">{avatar}</span>}
      <div className="flex min-w-0 flex-1 flex-col">
        {title && (
          <span className="font-semibold text-nest-foreground">{title}</span>
        )}
        {subheader && (
          <span className="text-sm text-nest-muted">{subheader}</span>
        )}
      </div>
      {action && <span className="shrink-0">{action}</span>}
    </Component>
  );
});

/**
 * CardContent component - the main content area of a Card.
 *
 * Provides consistent padding for card body content.
 */
export const CardContent = forwardRef<HTMLElement, CardContentProps & React.HTMLAttributes<HTMLElement>>(function CardContent(
  {
    className,
    component = 'div',
    children,
    ...props
  },
  ref
) {
  const Component = component;

  return (
    <Component
      ref={ref as any}
      className={cn('p-4', className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * CardActions component - the action area of a Card.
 *
 * Displays buttons and other interactive elements with consistent spacing.
 */
export const CardActions = forwardRef<HTMLElement, CardActionsProps & React.HTMLAttributes<HTMLElement>>(function CardActions(
  {
    className,
    component = 'div',
    children,
    disableSpacing = false,
    ...props
  },
  ref
) {
  const Component = component;

  const spacingStyles = disableSpacing ? '' : 'gap-2';

  return (
    <Component
      ref={ref as any}
      className={cn('flex items-center p-2', spacingStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * CardMedia component - the media section of a Card.
 *
 * Displays an image or custom media element at a fixed height.
 */
export const CardMedia = forwardRef<HTMLElement, CardMediaProps & Omit<React.HTMLAttributes<HTMLElement>, 'title'>>(function CardMedia(
  {
    className,
    component,
    image,
    alt = '',
    title,
    height = '140px',
    ...props
  },
  ref
) {
  const isImage = component === 'img' || (!component && image);

  if (isImage) {
    return (
      <img
        ref={ref as any}
        src={image}
        alt={alt}
        title={title}
        className={cn('h-[--card-media-height] w-full object-cover', className)}
        style={{ '--card-media-height': height } as React.CSSProperties}
        {...(props as any)}
      />
    );
  }

  const Component = component ?? 'div';

  return (
    <Component
      ref={ref as any}
      className={cn('h-[--card-media-height] w-full bg-nest-muted/20', className)}
      style={{ '--card-media-height': height } as React.CSSProperties}
      {...(props as any)}
    />
  );
});
