import { forwardRef, useEffect, useCallback, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import { Alert } from './Alert';
import type { AlertSeverity } from './Alert';

export interface SnackbarProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * The content of the snackbar.
   */
  children: ReactNode;
  /**
   * Whether the snackbar is open.
   */
  open: boolean;
  /**
   * Callback fired when the snackbar is requested to be closed.
   */
  onClose: () => void;
  /**
   * The severity of the snackbar.
   * @default 'info'
   */
  severity?: AlertSeverity;
  /**
   * Auto-close duration in milliseconds. Set to 0 to disable.
   * @default 5000
   */
  autoHideDuration?: number;
  /**
   * Action to display.
   */
  action?: ReactNode;
  /**
   * Position of the snackbar.
   * @default 'bottom-center'
   */
  position?: 'top-left' | 'top-center' | 'top-right' | 'bottom-left' | 'bottom-center' | 'bottom-right';
}

const POSITION_STYLES: Record<NonNullable<SnackbarProps['position']>, string> = {
  'top-left': 'top-4 left-4',
  'top-center': 'top-4 left-1/2 -translate-x-1/2',
  'top-right': 'top-4 right-4',
  'bottom-left': 'bottom-4 left-4',
  'bottom-center': 'bottom-4 left-1/2 -translate-x-1/2',
  'bottom-right': 'bottom-4 right-4',
};

/**
 * Snackbar component for brief messages.
 * Follows MUI Snackbar API conventions.
 *
 * @example
 * // Basic usage
 * <Snackbar open={open} onClose={() => setOpen(false)}>
 *   Message saved
 * </Snackbar>
 *
 * @example
 * // With action
 * <Snackbar
 *   open={open}
 *   onClose={() => setOpen(false)}
 *   action={<Button onClick={handleUndo}>Undo</Button>}
 * >
 *   Item deleted
 * </Snackbar>
 *
 * @example
 * // With severity
 * <Snackbar open={open} onClose={() => setOpen(false)} severity="error">
 *   Failed to save
 * </Snackbar>
 */
export const Snackbar = forwardRef<HTMLDivElement, SnackbarProps>(function Snackbar(
  {
    className,
    children,
    open,
    onClose,
    severity = 'info',
    autoHideDuration = 5000,
    action,
    position = 'bottom-center',
    ...props
  },
  ref
) {
  const handleClose = useCallback(() => {
    onClose();
  }, [onClose]);

  useEffect(() => {
    if (open && autoHideDuration > 0) {
      const timer = setTimeout(handleClose, autoHideDuration);
      return () => clearTimeout(timer);
    }
  }, [open, autoHideDuration, handleClose]);

  useEffect(() => {
    if (open) {
      document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
          handleClose();
        }
      });
    }
  }, [open, handleClose]);

  if (!open) return null;

  return (
    <div
      ref={ref}
      className={cn('fixed z-50', POSITION_STYLES[position], className)}
      role="status"
      aria-live="polite"
      {...props}
    >
      <Alert
        severity={severity}
        onClose={handleClose}
        action={action}
        variant="filled"
        className="shadow-lg min-w-[280px] max-w-[420px]"
      >
        {children}
      </Alert>
    </div>
  );
});
