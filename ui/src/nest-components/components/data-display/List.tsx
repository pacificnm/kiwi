import { forwardRef, type ReactNode, type ElementType } from 'react';
import { cn } from '../../lib/cn';

// List Props
export interface ListProps {
  /**
   * The component to render as.
   * @default 'ul'
   */
  component?: ElementType;
  /**
   * If true, compact vertical padding is applied.
   * @default false
   */
  dense?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: ReactNode;
}

// ListItem Props
export interface ListItemProps {
  /**
   * The component to render as.
   * @default 'li'
   */
  component?: ElementType;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: ReactNode;
}

// ListItemButton Props
export interface ListItemButtonProps {
  /**
   * The component to render as.
   * @default 'button'
   */
  component?: ElementType;
  /**
   * If true, the button is selected.
   * @default false
   */
  selected?: boolean;
  /**
   * If true, the button is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: ReactNode;
  /**
   * Click handler.
   */
  onClick?: () => void;
}

// ListItemText Props
export interface ListItemTextProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * The primary content.
   */
  primary?: ReactNode;
  /**
   * The secondary content.
   */
  secondary?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

// ListItemIcon Props
export interface ListItemIconProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display.
   */
  children?: ReactNode;
}

// ListItemAvatar Props
export interface ListItemAvatarProps {
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The content to display (typically an Avatar).
   */
  children?: ReactNode;
}

const DENSE_STYLES = 'py-1';
const NORMAL_STYLES = 'py-2';

/**
 * List component - a container for list items.
 * Follows MUI List API conventions.
 *
 * @example
 * // Basic list
 * <List>
 *   <ListItem>Item 1</ListItem>
 *   <ListItem>Item 2</ListItem>
 * </List>
 *
 * @example
 * // Dense list
 * <List dense>
 *   <ListItem>Dense item 1</ListItem>
 *   <ListItem>Dense item 2</ListItem>
 * </List>
 */
export const List = forwardRef<HTMLUListElement, ListProps>(function List(
  {
    component = 'ul',
    dense = false,
    className,
    children,
    ...props
  }: ListProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLUListElement>
) {
  const Component = component;

  const baseStyles = 'flex flex-col';
  const denseStyles = dense ? DENSE_STYLES : NORMAL_STYLES;

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, denseStyles, className)}
      role="list"
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * ListItem component - a list item container.
 *
 * @example
 * <ListItem>
 *   <ListItemIcon><StarIcon /></ListItemIcon>
 *   <ListItemText primary="Item title" secondary="Item description" />
 * </ListItem>
 */
export const ListItem = forwardRef<HTMLLIElement, ListItemProps>(function ListItem(
  {
    component = 'li',
    className,
    children,
    ...props
  }: ListItemProps & React.HTMLAttributes<HTMLElement>,
  ref: React.Ref<HTMLLIElement>
) {
  const Component = component;

  const baseStyles = 'flex items-center gap-3 px-3';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, className)}
      role="listitem"
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * ListItemButton component - an interactive list item.
 *
 * @example
 * <ListItemButton selected={selectedIndex === 0} onClick={() => setSelectedIndex(0)}>
 *   <ListItemText primary="Inbox" />
 * </ListItemButton>
 */
export const ListItemButton = forwardRef<HTMLButtonElement, ListItemButtonProps>(function ListItemButton(
  {
    component = 'button',
    selected = false,
    disabled = false,
    className,
    children,
    onClick,
    ...props
  }: ListItemButtonProps & React.ButtonHTMLAttributes<HTMLButtonElement>,
  ref: React.Ref<HTMLButtonElement>
) {
  const Component = component;

  const baseStyles = 'flex items-center gap-3 px-3 w-full text-left rounded-nest-sm transition-colors duration-150 focus:outline-none focus:ring-2 focus:ring-nest-primary/50';
  const selectedStyles = selected ? 'bg-nest-primary/10 text-nest-primary' : '';
  const hoverStyles = selected ? '' : 'hover:bg-nest-surface';
  const disabledStyles = disabled ? 'opacity-50 cursor-not-allowed pointer-events-none' : '';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, selectedStyles, hoverStyles, disabledStyles, className)}
      disabled={disabled}
      onClick={onClick}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * ListItemText component - text content for a list item.
 *
 * @example
 * <ListItemText
 *   primary="Primary text"
 *   secondary="Secondary text"
 * />
 */
export const ListItemText = forwardRef<HTMLDivElement, ListItemTextProps>(function ListItemText(
  {
    component = 'div',
    primary,
    secondary,
    className,
    ...props
  }: ListItemTextProps & React.HTMLAttributes<HTMLDivElement>,
  ref: React.Ref<HTMLDivElement>
) {
  const Component = component;

  return (
    <Component
      ref={ref as any}
      className={cn('flex flex-col flex-1 min-w-0', className)}
      {...(props as any)}
    >
      {primary && (
        <span className="text-nest-foreground truncate">{primary}</span>
      )}
      {secondary && (
        <span className="text-sm text-nest-muted truncate">{secondary}</span>
      )}
    </Component>
  );
});

/**
 * ListItemIcon component - icon container for a list item.
 *
 * @example
 * <ListItemIcon>
 *   <StarIcon />
 * </ListItemIcon>
 */
export const ListItemIcon = forwardRef<HTMLDivElement, ListItemIconProps>(function ListItemIcon(
  {
    component = 'div',
    className,
    children,
    ...props
  }: ListItemIconProps & React.HTMLAttributes<HTMLDivElement>,
  ref: React.Ref<HTMLDivElement>
) {
  const Component = component;

  const baseStyles = 'shrink-0 text-nest-muted';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});

/**
 * ListItemAvatar component - avatar container for a list item.
 *
 * @example
 * <ListItemAvatar>
 *   <Avatar src="/user.jpg" alt="User" />
 * </ListItemAvatar>
 */
export const ListItemAvatar = forwardRef<HTMLDivElement, ListItemAvatarProps>(function ListItemAvatar(
  {
    component = 'div',
    className,
    children,
    ...props
  }: ListItemAvatarProps & React.HTMLAttributes<HTMLDivElement>,
  ref: React.Ref<HTMLDivElement>
) {
  const Component = component;

  const baseStyles = 'shrink-0';

  return (
    <Component
      ref={ref as any}
      className={cn(baseStyles, className)}
      {...(props as any)}
    >
      {children}
    </Component>
  );
});
