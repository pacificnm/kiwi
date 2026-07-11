import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export type AppBarPosition = 'static' | 'fixed' | 'sticky';
export type AppBarColor = 'surface' | 'primary' | 'transparent';

export interface AppBarProps extends Omit<HTMLAttributes<HTMLElement>, 'color'> {
  /**
   * How the bar is positioned in its container.
   * @default 'static'
   */
  position?: AppBarPosition;
  /**
   * Background color scheme.
   * @default 'surface'
   */
  color?: AppBarColor;
  /**
   * Adds a bottom border + shadow, like a raised MUI AppBar.
   * @default true
   */
  elevation?: boolean;
  children?: ReactNode;
}

const COLOR_STYLES: Record<AppBarColor, string> = {
  surface: 'bg-nest-surface text-nest-foreground',
  primary: 'bg-nest-primary text-white',
  transparent: 'bg-transparent text-nest-foreground',
};

const POSITION_STYLES: Record<AppBarPosition, string> = {
  static: 'relative',
  fixed: 'fixed inset-x-0 top-0 z-40',
  sticky: 'sticky top-0 z-40',
};

/**
 * Top application bar — desktop-window chrome (title bar, menu bar, toolbar
 * row). Adapted from MUI's AppBar for Nest's compact desktop shells; pair
 * with {@link Toolbar} for the flex row of title/actions, or with
 * {@link MenuBar} for a File/Edit-style dropdown menu row.
 *
 * @example
 * <AppBar>
 *   <Toolbar>
 *     <span className="font-semibold">My App</span>
 *   </Toolbar>
 * </AppBar>
 */
export const AppBar = forwardRef<HTMLElement, AppBarProps>(function AppBar(
  { className, position = 'static', color = 'surface', elevation = true, children, ...props },
  ref
) {
  return (
    <header
      ref={ref}
      className={cn(
        'flex shrink-0 flex-col',
        POSITION_STYLES[position],
        COLOR_STYLES[color],
        elevation && 'border-b border-nest-border shadow-sm',
        className
      )}
      {...props}
    >
      {children}
    </header>
  );
});

export type ToolbarVariant = 'regular' | 'dense';

export interface ToolbarProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * `dense` matches typical desktop title-bar chrome (32px); `regular` matches
   * a standard toolbar row (48px).
   * @default 'regular'
   */
  variant?: ToolbarVariant;
  children?: ReactNode;
}

const TOOLBAR_VARIANT_STYLES: Record<ToolbarVariant, string> = {
  regular: 'h-12 px-4 gap-3',
  dense: 'h-8 px-2 gap-2',
};

/**
 * Flex row inside an {@link AppBar}: title, icon buttons, and actions laid
 * out left-to-right with consistent height and spacing.
 */
export const Toolbar = forwardRef<HTMLDivElement, ToolbarProps>(function Toolbar(
  { className, variant = 'regular', children, ...props },
  ref
) {
  return (
    <div
      ref={ref}
      className={cn('flex items-center', TOOLBAR_VARIANT_STYLES[variant], className)}
      {...props}
    >
      {children}
    </div>
  );
});
