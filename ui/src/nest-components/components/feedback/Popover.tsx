import { forwardRef, type ReactNode, useState, type Ref } from 'react';
import { cn } from '../../lib/cn';
import {
  useFloating,
  useDismiss,
  useRole,
  useInteractions,
  useClick,
  FloatingPortal,
  offset,
  flip,
  shift,
  useMergeRefs,
  type Placement,
} from '@floating-ui/react';

export type PopoverPlacement = Placement;

export interface PopoverProps {
  /**
   * The content of the popover.
   */
  children?: ReactNode;
  /**
   * The element that triggers the popover.
   */
  trigger?: ReactNode;
  /**
   * If true, the popover is open.
   */
  open?: boolean;
  /**
   * Callback fired when the open state changes.
   */
  onOpenChange?: (open: boolean) => void;
  /**
   * The placement of the popover.
   * @default 'bottom'
   */
  placement?: PopoverPlacement;
  /**
   * If true, the popover closes on outside click.
   * @default true
   */
  closeOnOutsideClick?: boolean;
  /**
   * If true, the popover closes on Escape key.
   * @default true
   */
  closeOnEscape?: boolean;
  /**
   * Additional CSS classes for the popover content.
   */
  className?: string;
}

/**
 * Popover component - a popup that appears relative to an anchor element.
 * Follows MUI Popover API conventions.
 *
 * Popover displays content in a popup that positions itself relative to an anchor.
 *
 * @example
 * // Basic popover
 * <Popover
 *   trigger={<button>Open</button>}
 *   open={open}
 *   onOpenChange={setOpen}
 * >
 *   <div>Popover content</div>
 * </Popover>
 */
export const Popover = forwardRef<HTMLDivElement, PopoverProps>(function Popover(
  {
    children,
    trigger,
    open: openProp,
    onOpenChange,
    placement = 'bottom',
    closeOnOutsideClick = true,
    closeOnEscape = true,
    className,
    ...props
  }: PopoverProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [controlledOpen, setControlledOpen] = useState(openProp ?? false);
  const open = openProp ?? controlledOpen;

  const { refs, floatingStyles, context } = useFloating({
    open,
    onOpenChange: (newOpen) => {
      setControlledOpen(newOpen);
      onOpenChange?.(newOpen);
    },
    placement,
    middleware: [
      offset(8),
      flip(),
      shift({ padding: 8 }),
    ],
  });

  const click = useClick(context, {
    enabled: openProp === undefined,
  });

  const dismiss = useDismiss(context, {
    outsidePress: closeOnOutsideClick,
    escapeKey: closeOnEscape,
  });

  const role = useRole(context, { role: 'dialog' });

  const { getReferenceProps, getFloatingProps } = useInteractions([
    click,
    dismiss,
    role,
  ]);

  // Merge refs for trigger element
  const mergedRef = useMergeRefs([refs.setReference, ref as Ref<unknown>]);

  return (
    <>
      {trigger && (
        <span ref={mergedRef} {...getReferenceProps()}>
          {trigger}
        </span>
      )}

      <FloatingPortal>
        {open && (
          <div
            ref={refs.setFloating}
            className={cn(
              'z-50',
              'bg-nest-surface',
              'border border-nest-border',
              'rounded-nest-md',
              'shadow-lg',
              'p-4',
              'min-w-[200px]',
              'max-w-[300px]',
              className
            )}
            style={floatingStyles}
            role="dialog"
            {...getFloatingProps()}
            {...props}
          >
            {children}
          </div>
        )}
      </FloatingPortal>
    </>
  );
});
