import { forwardRef, type ReactNode, useState, useRef, cloneElement, isValidElement, type Ref } from 'react';
import { cn } from '../../lib/cn';
import {
  useFloating,
  useHover,
  useFocus,
  useDismiss,
  useRole,
  useInteractions,
  FloatingPortal,
  offset,
  flip,
  shift,
  useMergeRefs,
} from '@floating-ui/react';

export type TooltipPlacement = 'top' | 'bottom' | 'left' | 'right' | 'top-start' | 'top-end' | 'bottom-start' | 'bottom-end' | 'left-start' | 'left-end' | 'right-start' | 'right-end';

export interface TooltipProps {
  /**
   * The content of the tooltip.
   */
  title: ReactNode;
  /**
   * The element to attach the tooltip to.
   */
  children: ReactNode;
  /**
   * The placement of the tooltip relative to the anchor.
   * @default 'top'
   */
  placement?: TooltipPlacement;
  /**
   * If true, the tooltip is always open.
   * @default false
   */
  open?: boolean;
  /**
   * Callback fired when the open state changes.
   */
  onOpenChange?: (open: boolean) => void;
  /**
   * Delay in milliseconds before the tooltip opens.
   * @default 0
   */
  enterDelay?: number;
  /**
   * Delay in milliseconds before the tooltip closes.
   * @default 0
   */
  leaveDelay?: number;
  /**
   * If true, show an arrow on the tooltip.
   * @default false
   */
  arrow?: boolean;
  /**
   * Additional CSS classes for the tooltip.
   */
  className?: string;
}

/**
 * Tooltip component - a popup that displays information on hover or focus.
 * Follows MUI Tooltip API conventions.
 *
 * Tooltip shows additional information when users hover or focus an element.
 *
 * @example
 * // Basic tooltip
 * <Tooltip title="Helpful information">
 *   <button>Hover me</button>
 * </Tooltip>
 *
 * @example
 * // Different placement
 * <Tooltip title="Info" placement="right">
 *   <button>Hover me</button>
 * </Tooltip>
 */
export const Tooltip = forwardRef<HTMLDivElement, TooltipProps>(function Tooltip(
  {
    title,
    children,
    placement = 'top',
    open: openProp,
    onOpenChange,
    enterDelay = 0,
    leaveDelay = 0,
    arrow = false,
    className,
    ...props
  }: TooltipProps,
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

  const hover = useHover(context, {
    delay: { open: enterDelay, close: leaveDelay },
    enabled: openProp === undefined,
  });

  const focus = useFocus(context, {
    enabled: openProp === undefined,
  });

  const dismiss = useDismiss(context);
  const role = useRole(context, { role: 'tooltip' });

  const { getReferenceProps, getFloatingProps } = useInteractions([
    hover,
    focus,
    dismiss,
    role,
  ]);

  const arrowRef = useRef<HTMLDivElement>(null);

  // Merge refs for the child element
  const mergedRef = useMergeRefs([refs.setReference, ref as Ref<unknown>]);

  // Render children with reference props
  const renderedChildren = isValidElement(children)
    ? cloneElement(children, {
        ...getReferenceProps(),
        ref: mergedRef,
      } as any)
    : children;

  return (
    <>
      {renderedChildren}

      <FloatingPortal>
        {open && (
          <div
            ref={refs.setFloating}
            role="tooltip"
            className={cn(
              'z-50',
              'bg-nest-foreground text-nest-background',
              'text-xs font-medium',
              'px-2 py-1',
              'rounded-nest-sm',
              'shadow-lg',
              'max-w-xs',
              'text-center',
              className
            )}
            style={floatingStyles}
            {...getFloatingProps()}
            {...props}
          >
            {title}
            {arrow && (
              <div
                ref={arrowRef}
                className="absolute w-2 h-2 bg-nest-foreground rotate-45"
              />
            )}
          </div>
        )}
      </FloatingPortal>
    </>
  );
});
