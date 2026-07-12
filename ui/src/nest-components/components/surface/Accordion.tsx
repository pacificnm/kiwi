import { forwardRef, type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';
import { ChevronDown } from 'lucide-react';
import { createContext, useContext } from 'react';

interface AccordionContextValue {
  expanded?: string | string[];
  onChange?: (value: string | string[]) => void;
  disabled?: boolean;
  exclusive?: boolean;
}

const AccordionContext = createContext<AccordionContextValue | undefined>(undefined);

export interface AccordionProps {
  /**
   * The expanded panel(s) value(s).
   */
  expanded?: string | string[];
  /**
   * Default expanded panel(s) for uncontrolled accordion.
   */
  defaultExpanded?: string | string[];
  /**
   * Callback fired when expanded state changes.
   */
  onChange?: (value: string | string[]) => void;
  /**
   * If true, the accordion is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, only one panel can be expanded at a time.
   * @default false
   */
  exclusive?: boolean;
  /**
   * The content of the accordion (AccordionItem components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Accordion component - a collection of collapsible panels.
 * Follows MUI Accordion API conventions.
 *
 * Accordion allows users to show/hide sections of content.
 *
 * @example
 * // Single expanded panel
 * <Accordion expanded="panel1">
 *   <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
 *   <AccordionItem value="panel2" summary="Panel 2">Content 2</AccordionItem>
 * </Accordion>
 *
 * @example
 * // Multiple expanded panels
 * <Accordion defaultExpanded={['panel1', 'panel2']}>
 *   <AccordionItem value="panel1" summary="Panel 1">Content 1</AccordionItem>
 *   <AccordionItem value="panel2" summary="Panel 2">Content 2</AccordionItem>
 * </Accordion>
 */
export const Accordion = forwardRef<HTMLDivElement, AccordionProps>(function Accordion(
  {
    expanded: expandedProp,
    defaultExpanded,
    onChange,
    disabled = false,
    exclusive = false,
    children,
    className,
    ...props
  }: AccordionProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [expanded, setExpanded] = useControllableState<string | string[]>({
    value: expandedProp,
    defaultValue: defaultExpanded ?? (exclusive ? '' : []),
    onChange,
  });

  const handleChange = (newValue: string | string[]) => {
    setExpanded(newValue);
  };

  const baseStyles = cn(
    'w-full',
    'border border-nest-border rounded-nest-md',
    'bg-nest-surface',
    'divide-y divide-nest-border',
    className
  );

  return (
    <AccordionContext.Provider
      value={{
        expanded,
        onChange: handleChange,
        disabled,
        exclusive,
      }}
    >
      <div ref={ref as any} className={baseStyles} {...props}>
        {children}
      </div>
    </AccordionContext.Provider>
  );
});

export interface AccordionItemProps {
  /**
   * The value of the accordion item.
   */
  value: string;
  /**
   * The summary/header content of the accordion item.
   */
  summary?: ReactNode;
  /**
   * The collapsible content of the accordion item.
   */
  children?: ReactNode;
  /**
   * If true, the item is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * AccordionItem component - a single collapsible panel within an Accordion.
 * Follows MUI Accordion API conventions.
 *
 * @example
 * <AccordionItem value="panel1" summary="Click to expand">
 *   <p>Hidden content</p>
 * </AccordionItem>
 */
export const AccordionItem = forwardRef<HTMLDivElement, AccordionItemProps>(function AccordionItem(
  {
    value,
    summary,
    children,
    disabled = false,
    className,
    ...props
  }: AccordionItemProps,
  ref: React.Ref<HTMLDivElement>
) {
  const context = useContext(AccordionContext);

  const expanded = context?.expanded;
  const onChange = context?.onChange;
  const accordionDisabled = context?.disabled;
  const exclusive = context?.exclusive ?? false;

  const isExpanded = exclusive || typeof expanded === 'string'
    ? expanded === value
    : Array.isArray(expanded) && expanded.includes(value);

  const isDisabled = disabled || accordionDisabled || false;

  const handleClick = () => {
    if (isDisabled || !onChange) return;

    const isCurrentlyExpanded = exclusive
      ? expanded === value
      : Array.isArray(expanded) && expanded.includes(value);

    if (exclusive) {
      onChange(isCurrentlyExpanded ? '' : value);
    } else {
      const current = Array.isArray(expanded) ? expanded : [];
      if (current.includes(value)) {
        onChange(current.filter((v) => v !== value));
      } else {
        onChange([...current, value]);
      }
    }
  };

  const baseStyles = cn(
    'overflow-hidden',
    className
  );

  return (
    <div ref={ref as any} className={baseStyles} {...props}>
      <AccordionSummary
        expanded={isExpanded}
        disabled={isDisabled}
        onClick={handleClick}
      >
        {summary}
      </AccordionSummary>
      <AccordionDetails expanded={isExpanded}>
        {children}
      </AccordionDetails>
    </div>
  );
});

export interface AccordionSummaryProps {
  /**
   * Whether the summary is expanded.
   */
  expanded?: boolean;
  /**
   * If true, the summary is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Click handler for the summary.
   */
  onClick?: () => void;
  /**
   * The content of the summary.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * AccordionSummary component - the clickable header of an AccordionItem.
 *
 * @example
 * <AccordionSummary expanded={expanded} onClick={handleClick}>
 *   Panel Title
 * </AccordionSummary>
 */
export const AccordionSummary = forwardRef<HTMLButtonElement, AccordionSummaryProps>(function AccordionSummary(
  {
    expanded = false,
    disabled = false,
    onClick,
    children,
    className,
    ...props
  }: AccordionSummaryProps,
  ref: React.Ref<HTMLButtonElement>
) {
  const baseStyles = cn(
    'w-full',
    'flex items-center justify-between',
    'px-4 py-3',
    'text-left',
    'text-nest-foreground font-medium',
    'bg-nest-surface',
    'transition-colors duration-200',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2',
    disabled && 'opacity-50 cursor-not-allowed',
    !disabled && 'hover:bg-nest-muted/50',
    className
  );

  return (
    <button
      ref={ref as any}
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={baseStyles}
      aria-expanded={expanded}
      {...props}
    >
      <span>{children}</span>
      <ChevronDown
        className={cn(
          'size-5 text-nest-muted transition-transform duration-200',
          expanded && 'rotate-180'
        )}
        aria-hidden="true"
      />
    </button>
  );
});

export interface AccordionDetailsProps {
  /**
   * Whether the details are expanded.
   */
  expanded?: boolean;
  /**
   * The content of the details.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * AccordionDetails component - the collapsible content area of an AccordionItem.
 *
 * @example
 * <AccordionDetails expanded={expanded}>
 *   <p>Hidden content</p>
 * </AccordionDetails>
 */
export const AccordionDetails = forwardRef<HTMLDivElement, AccordionDetailsProps>(function AccordionDetails(
  {
    expanded = false,
    children,
    className,
    ...props
  }: AccordionDetailsProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'px-4',
    'transition-all duration-200',
    expanded ? 'py-3 opacity-100' : 'py-0 opacity-0',
    'overflow-hidden',
    className
  );

  return (
    <div
      ref={ref as any}
      className={baseStyles}
      role="region"
      {...props}
    >
      {expanded && children}
    </div>
  );
});
