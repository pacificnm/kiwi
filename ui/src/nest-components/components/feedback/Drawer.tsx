import { forwardRef, type ReactNode, useEffect, useRef } from 'react';
import { cn } from '../../lib/cn';
import { FloatingPortal } from '@floating-ui/react';

export type DrawerAnchor = 'left' | 'right' | 'top' | 'bottom';

export interface DrawerProps {
  /**
   * If true, the drawer is open.
   */
  open?: boolean;
  /**
   * Callback fired when the drawer requests to close.
   */
  onClose?: () => void;
  /**
   * The content of the drawer.
   */
  children?: ReactNode;
  /**
   * The side from which the drawer slides in.
   * @default 'left'
   */
  anchor?: DrawerAnchor;
  /**
   * The width of the drawer (for left/right anchors).
   * @default 320
   */
  width?: number | string;
  /**
   * The height of the drawer (for top/bottom anchors).
   * @default 256
   */
  height?: number | string;
  /**
   * If true, clicking outside closes the drawer.
   * @default true
   */
  closeOnOutsideClick?: boolean;
  /**
   * If true, pressing Escape closes the drawer.
   * @default true
   */
  closeOnEscape?: boolean;
  /**
   * If true, focus is trapped inside the drawer.
   * @default true
   */
  trapFocus?: boolean;
  /**
   * Additional CSS classes for the backdrop.
   */
  backdropClassName?: string;
  /**
   * Additional CSS classes for the drawer content.
   */
  className?: string;
}

/**
 * Hook to trap focus within a container element.
 */
function useFocusTrap(containerRef: React.RefObject<HTMLElement | null>, active: boolean) {
  useEffect(() => {
    if (!active) return;

    const container = containerRef.current;
    if (!container) return;

    const focusableSelectors = [
      'button:not([disabled])',
      'input:not([disabled])',
      'select:not([disabled])',
      'textarea:not([disabled])',
      'a[href]',
      '[tabindex]:not([tabindex="-1"])',
    ].join(', ');

    const getFocusableElements = () => {
      return Array.from(container.querySelectorAll<HTMLElement>(focusableSelectors));
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Tab') return;

      const focusable = getFocusableElements();
      if (focusable.length === 0) return;

      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      const current = document.activeElement as HTMLElement;

      if (event.shiftKey) {
        if (current === first || !container.contains(current)) {
          event.preventDefault();
          last.focus();
        }
      } else {
        if (current === last || !container.contains(current)) {
          event.preventDefault();
          first.focus();
        }
      }
    };

    const focusable = getFocusableElements();
    if (focusable.length > 0) {
      focusable[0].focus();
    } else {
      container.focus();
    }

    document.addEventListener('keydown', handleKeyDown);
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
    };
  }, [active, containerRef]);
}

/**
 * Drawer component - a slide-out panel from the edge of the screen.
 * Follows MUI Drawer API conventions.
 *
 * Drawer slides in from one edge of the viewport and can be used for:
 * - Navigation menus
 * - Settings panels
 * - Contextual information
 *
 * @example
 * // Left drawer
 * <Drawer open={open} onClose={() => setOpen(false)}>
 *   <nav>Navigation items...</nav>
 * </Drawer>
 *
 * @example
 * // Right drawer for settings
 * <Drawer anchor="right" open={open} onClose={() => setOpen(false)}>
 *   <div>Settings panel...</div>
 * </Drawer>
 */
export const Drawer = forwardRef<HTMLDivElement, DrawerProps>(function Drawer(
  {
    open = false,
    onClose,
    children,
    anchor = 'left',
    width = 320,
    height = 256,
    closeOnOutsideClick = true,
    closeOnEscape = true,
    trapFocus = true,
    backdropClassName,
    className,
    ...props
  }: DrawerProps,
  ref: React.Ref<HTMLDivElement>
) {
  const containerRef = useRef<HTMLDivElement>(null);

  // Focus trap
  useFocusTrap(containerRef, trapFocus && open);

  // Handle escape key and outside click
  useEffect(() => {
    if (!open || !onClose) return;

    const handleKeyDown = (event: KeyboardEvent) => {
      if (closeOnEscape && event.key === 'Escape') {
        onClose();
      }
    };

    const handleMouseDown = (event: MouseEvent) => {
      if (
        closeOnOutsideClick &&
        containerRef.current &&
        !containerRef.current.contains(event.target as Node)
      ) {
        onClose();
      }
    };

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('mousedown', handleMouseDown);

    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.removeEventListener('mousedown', handleMouseDown);
    };
  }, [open, onClose, closeOnEscape, closeOnOutsideClick]);

  // Prevent body scroll when drawer is open
  useEffect(() => {
    if (open) {
      document.body.style.overflow = 'hidden';
    } else {
      document.body.style.overflow = '';
    }

    return () => {
      document.body.style.overflow = '';
    };
  }, [open]);

  if (!open) return null;

  const isVertical = anchor === 'top' || anchor === 'bottom';
  const sizeStyle = isVertical ? { height } : { width };

  const anchorStyles: React.CSSProperties = {
    ...sizeStyle,
    [anchor]: 0,
  };

  return (
    <FloatingPortal>
      {/* Backdrop */}
      <div
        className={cn(
          'fixed inset-0 z-50',
          'bg-black/50',
          'transition-opacity duration-300',
          backdropClassName
        )}
      />

      {/* Drawer content */}
      <div
        ref={(node) => {
          containerRef.current = node;
          if (typeof ref === 'function') {
            ref(node);
          } else if (ref) {
            (ref as React.MutableRefObject<HTMLDivElement | null>).current = node;
          }
        }}
        className={cn(
          'fixed z-50',
          'bg-nest-surface',
          'shadow-xl',
          'transition-transform duration-300 ease-out',
          'flex flex-col',
          className
        )}
        style={anchorStyles}
        role="dialog"
        aria-modal="true"
        tabIndex={-1}
        {...props}
      >
        {children}
      </div>
    </FloatingPortal>
  );
});
