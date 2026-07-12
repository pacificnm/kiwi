import { forwardRef, type ReactNode, useState, type Ref, useEffect } from 'react';
import { cn } from '../../lib/cn';
import {
  useFloating,
  useClick,
  useDismiss,
  useRole,
  useInteractions,
  FloatingPortal,
  offset,
  flip,
  shift,
  useMergeRefs,
} from '@floating-ui/react';
import { ChevronDown, Check } from 'lucide-react';

export type SelectSize = 'small' | 'medium';

export interface SelectOption {
  value: string;
  label: ReactNode;
  disabled?: boolean;
}

export interface SelectProps {
  /**
   * The selected value.
   */
  value?: string;
  /**
   * Default value for uncontrolled select.
   */
  defaultValue?: string;
  /**
   * Callback fired when value changes.
   */
  onChange?: (value: string) => void;
  /**
   * Array of options to select from.
   */
  options?: SelectOption[];
  /**
   * Children (MenuItem components) - alternative to options prop.
   */
  children?: ReactNode;
  /**
   * Placeholder text when no value is selected.
   */
  placeholder?: string;
  /**
   * If true, the select is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, the select shows an error state.
   * @default false
   */
  error?: boolean;
  /**
   * The size of the select.
   * @default 'medium'
   */
  size?: SelectSize;
  /**
   * Label for the select.
   */
  label?: ReactNode;
  /**
   * Additional CSS classes for the trigger.
   */
  className?: string;
}

/**
 * Select component - a dropdown selection input.
 * Follows MUI Select API conventions.
 *
 * Select allows users to choose one option from a dropdown list.
 *
 * @example
 * // Basic select
 * <Select
 *   value={value}
 *   onChange={(v) => setValue(v)}
 *   options={[
 *     { value: 'a', label: 'Option A' },
 *     { value: 'b', label: 'Option B' },
 *   ]}
 * />
 */
export const Select = forwardRef<HTMLButtonElement, SelectProps>(function Select(
  {
    value: valueProp,
    defaultValue,
    onChange,
    options = [],
    children,
    placeholder = 'Select...',
    disabled = false,
    error = false,
    size = 'medium',
    label,
    className,
    ...props
  }: SelectProps,
  ref: React.Ref<HTMLButtonElement>
) {
  const [open, setOpen] = useState(false);
  const [value, setValue] = useState(valueProp ?? defaultValue ?? '');

  // Controlled/uncontrolled sync
  useEffect(() => {
    if (valueProp !== undefined) {
      setValue(valueProp);
    }
  }, [valueProp]);

  const { refs, floatingStyles, context } = useFloating({
    open,
    onOpenChange: setOpen,
    placement: 'bottom-start',
    middleware: [
      offset(4),
      flip(),
      shift({ padding: 8 }),
    ],
  });

  const click = useClick(context);
  const dismiss = useDismiss(context);
  const role = useRole(context, { role: 'listbox' });

  const { getReferenceProps, getFloatingProps } = useInteractions([
    click,
    dismiss,
    role,
  ]);

  const selectedOption = options.find((opt) => opt.value === value);
  const mergedRef = useMergeRefs([refs.setReference, ref as Ref<HTMLButtonElement>]);

  const sizeStyles = size === 'small' ? 'text-xs px-2 py-1.5' : 'text-sm px-3 py-2';

  const handleSelect = (optionValue: string) => {
    setValue(optionValue);
    onChange?.(optionValue);
    setOpen(false);
  };

  return (
    <>
      {label && (
        <label className="block text-sm font-medium text-nest-foreground mb-1">
          {label}
        </label>
      )}
      <button
        ref={mergedRef}
        type="button"
        disabled={disabled}
        className={cn(
          'w-full flex items-center justify-between gap-2',
          'border rounded-nest-md',
          'bg-nest-surface',
          'transition-colors duration-150',
          'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-1',
          sizeStyles,
          disabled && 'opacity-50 cursor-not-allowed',
          error && 'border-nest-error',
          !error && !disabled && 'border-nest-border hover:border-nest-primary/50',
          className
        )}
        aria-haspopup="listbox"
        aria-expanded={open}
        {...getReferenceProps()}
        {...props}
      >
        <span className={cn(!selectedOption && 'text-nest-muted')}>
          {selectedOption?.label ?? placeholder}
        </span>
        <ChevronDown className={cn('size-4 text-nest-muted transition-transform', open && 'rotate-180')} />
      </button>

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
              'py-1',
              'min-w-[--reference-width]',
              'max-h-64 overflow-y-auto'
            )}
            style={floatingStyles}
            role="listbox"
            {...getFloatingProps()}
          >
            {options.map((option) => (
              <button
                key={option.value}
                type="button"
                disabled={option.disabled}
                className={cn(
                  'w-full flex items-center justify-between gap-2',
                  'px-3 py-2 text-sm',
                  'text-left',
                  'transition-colors duration-150',
                  'focus:outline-none focus:bg-nest-primary/10',
                  option.disabled && 'opacity-50 cursor-not-allowed',
                  !option.disabled && 'hover:bg-nest-muted/50',
                  option.value === value && 'bg-nest-primary/10 text-nest-primary font-medium'
                )}
                role="option"
                aria-selected={option.value === value}
                onClick={() => !option.disabled && handleSelect(option.value)}
              >
                <span>{option.label}</span>
                {option.value === value && <Check className="size-4" />}
              </button>
            ))}
            {children}
          </div>
        )}
      </FloatingPortal>
    </>
  );
});
