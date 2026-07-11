import { forwardRef, type HTMLAttributes, type ReactNode, useEffect, useCallback } from 'react';
import { cn } from '../../lib/cn';
import { IconButton } from '../inputs/IconButton';
import { X } from 'lucide-react';

export interface DialogProps extends Omit<HTMLAttributes<HTMLDivElement>, 'title'> {
  /**
   * The content of the dialog.
   */
  children: ReactNode;
  /**
   * Whether the dialog is open.
   */
  open: boolean;
  /**
   * Callback fired when the dialog is requested to be closed.
   */
  onClose: () => void;
  /**
   * The title of the dialog.
   */
  title?: ReactNode;
  /**
   * Action buttons displayed at the bottom of the dialog.
   */
  actions?: ReactNode;
  /**
   * Prevent closing when clicking outside the dialog.
   * @default false
   */
  disableBackdropClick?: boolean;
  /**
   * Prevent closing when pressing Escape.
   * @default false
   */
  disableEscapeKeyDown?: boolean;
}

/**
 * Dialog component for modal overlays.
 * Follows MUI Dialog API conventions.
 *
 * @example
 * // Basic usage
 * <Dialog open={open} onClose={() => setOpen(false)}>
 *   <p>Dialog content</p>
 * </Dialog>
 *
 * @example
 * // With title and actions
 * <Dialog
 *   open={open}
 *   onClose={handleClose}
 *   title="Confirm Delete"
 *   actions={
 *     <>
 *       <Button onClick={handleClose}>Cancel</Button>
 *       <Button color="error" onClick={handleDelete}>Delete</Button>
 *     </>
 *   }
 * >
 *   <p>Are you sure you want to delete this item?</p>
 * </Dialog>
 */
export const Dialog = forwardRef<HTMLDivElement, DialogProps>(function Dialog(
  {
    className,
    children,
    open,
    onClose,
    title,
    actions,
    disableBackdropClick = false,
    disableEscapeKeyDown = false,
    ...props
  },
  ref
) {
  const handleBackdropClick = useCallback(
    (event: React.MouseEvent<HTMLDivElement>) => {
      if (event.target === event.currentTarget && !disableBackdropClick) {
        onClose();
      }
    },
    [onClose, disableBackdropClick]
  );

  const handleKeyDown = useCallback(
    (event: KeyboardEvent) => {
      if (event.key === 'Escape' && !disableEscapeKeyDown) {
        onClose();
      }
    },
    [onClose, disableEscapeKeyDown]
  );

  useEffect(() => {
    if (open) {
      document.addEventListener('keydown', handleKeyDown);
      document.body.style.overflow = 'hidden';
    }
    return () => {
      document.removeEventListener('keydown', handleKeyDown);
      document.body.style.overflow = '';
    };
  }, [open, handleKeyDown]);

  if (!open) return null;

  return (
    <div
      ref={ref}
      className={cn(
        'fixed inset-0 z-50 flex items-center justify-center',
        'bg-black/50 backdrop-blur-sm',
        className
      )}
      onClick={handleBackdropClick}
      role="dialog"
      aria-modal="true"
      {...props}
    >
      <div
        className={cn(
          'relative mx-4 w-full max-w-md rounded-nest-lg bg-nest-background',
          'shadow-xl ring-1 ring-nest-border'
        )}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header — onClose is required, so the close button always renders */}
        <div className="flex items-center justify-between border-b border-nest-border px-4 py-3">
          {title && (
            <h2 className="text-lg font-semibold text-nest-foreground">{title}</h2>
          )}
          <IconButton
            aria-label="Close dialog"
            onClick={onClose}
            size="small"
            color="default"
          >
            <X className="size-4" />
          </IconButton>
        </div>

        {/* Content */}
        <div className="px-4 py-4">{children}</div>

        {/* Actions */}
        {actions && (
          <div className="flex items-center justify-end gap-2 border-t border-nest-border px-4 py-3">
            {actions}
          </div>
        )}
      </div>
    </div>
  );
});
