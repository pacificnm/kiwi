import { forwardRef, type ReactNode } from 'react';
import { cn } from '../../lib/cn';

export interface BackdropProps {
  /**
   * If true, the backdrop is shown.
   */
  open?: boolean;
  /**
   * Callback fired when the backdrop is clicked.
   */
  onClick?: (event: React.MouseEvent<HTMLDivElement>) => void;
  /**
   * If true, the backdrop is invisible.
   * @default false
   */
  invisible?: boolean;
  /**
   * The content of the backdrop.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * If true, the backdrop is hidden from screen readers.
   * @default false
   */
  'aria-hidden'?: boolean;
}

/**
 * Backdrop component - a backdrop overlay for modals and drawers.
 * Follows MUI Backdrop API conventions.
 *
 * Backdrop is a semi-transparent overlay that appears behind modals,
 * drawers, and other overlay components.
 *
 * @example
 * // Basic backdrop
 * <Backdrop open={open} />
 *
 * @example
 * // Invisible backdrop (clickable area only)
 * <Backdrop open={open} invisible />
 *
 * @example
 * // With click handler
 * <Backdrop open={open} onClick={() => setOpen(false)} />
 */
export const Backdrop = forwardRef<HTMLDivElement, BackdropProps>(function Backdrop(
  {
    open = false,
    onClick,
    invisible = false,
    children,
    className,
    'aria-hidden': ariaHidden = false,
    ...props
  }: BackdropProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'fixed inset-0 z-50',
    'transition-opacity duration-300',
    invisible ? 'bg-transparent' : 'bg-black/50',
    open ? 'opacity-100' : 'opacity-0 pointer-events-none',
    className
  );

  if (!open && !invisible) {
    return null;
  }

  return (
    <div
      ref={ref as any}
      className={baseStyles}
      onClick={onClick}
      aria-hidden={ariaHidden}
      {...props}
    >
      {children}
    </div>
  );
});
