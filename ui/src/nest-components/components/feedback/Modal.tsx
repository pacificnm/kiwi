import { forwardRef, type ReactNode, useEffect, useRef } from 'react';
import { cn } from '../../lib/cn';
import { FloatingPortal } from '@floating-ui/react';

/**
 * Hook to trap focus within a container element.
 * Adapted from focus-trap principles for modal dialogs.
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
        // Shift + Tab
        if (current === first || !container.contains(current)) {
          event.preventDefault();
          last.focus();
        }
      } else {
        // Tab
        if (current === last || !container.contains(current)) {
          event.preventDefault();
          first.focus();
        }
      }
    };

    // Focus first focusable element
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

export interface ModalProps {
  /**
   * If true, the modal is open.
   */
  open?: boolean;
  /**
   * Callback fired when the modal requests to close.
   */
  onClose?: () => void;
  /**
   * The content of the modal.
   */
  children?: ReactNode;
  /**
   * If true, clicking outside closes the modal.
   * @default true
   */
  closeOnOutsideClick?: boolean;
  /**
   * If true, pressing Escape closes the modal.
   * @default true
   */
  closeOnEscape?: boolean;
  /**
   * If true, focus is trapped inside the modal.
   * @default true
   */
  trapFocus?: boolean;
  /**
   * Additional CSS classes for the backdrop.
   */
  backdropClassName?: string;
  /**
   * Additional CSS classes for the modal content.
   */
  className?: string;
}

/**
 * Modal component - a dialog overlay with focus trap.
 * Follows MUI Modal API conventions.
 *
 * Modal displays content in an overlay that traps focus and blocks interaction
 * with the rest of the page.
 *
 * @example
 * // Basic modal
 * <Modal open={open} onClose={() => setOpen(false)}>
 *   <div>Modal content</div>
 * </Modal>
 */
export const Modal = forwardRef<HTMLDivElement, ModalProps>(function Modal(
  {
    open = false,
    onClose,
    children,
    closeOnOutsideClick = true,
    closeOnEscape = true,
    trapFocus = true,
    backdropClassName,
    className,
    ...props
  }: ModalProps,
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

  // Prevent body scroll when modal is open
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

      {/* Modal content */}
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
          'left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2',
          'max-h-[90vh] overflow-auto',
          'bg-nest-surface',
          'rounded-nest-lg',
          'shadow-xl',
          className
        )}
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
