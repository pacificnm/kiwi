import { forwardRef, type InputHTMLAttributes, useEffect, useRef } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';
import { Check, Minus } from 'lucide-react';

export type CheckboxColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info';
export type CheckboxSize = 'small' | 'medium';

const COLOR_STYLES: Record<CheckboxColor, string> = {
  primary: 'checked:bg-nest-primary checked:border-nest-primary checked:hover:bg-nest-primary/90',
  secondary: 'checked:bg-nest-secondary checked:border-nest-secondary checked:hover:bg-nest-secondary/90',
  accent: 'checked:bg-nest-accent checked:border-nest-accent checked:hover:bg-nest-accent/90',
  success: 'checked:bg-nest-success checked:border-nest-success checked:hover:bg-nest-success/90',
  warning: 'checked:bg-nest-warning checked:border-nest-warning checked:hover:bg-nest-warning/90',
  error: 'checked:bg-nest-error checked:border-nest-error checked:hover:bg-nest-error/90',
  info: 'checked:bg-nest-info checked:border-nest-info checked:hover:bg-nest-info/90',
};

const SIZE_STYLES: Record<CheckboxSize, string> = {
  small: 'size-4',
  medium: 'size-5',
};

export interface CheckboxProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /**
   * Whether the checkbox is checked.
   */
  checked?: boolean;
  /**
   * Default checked state for uncontrolled checkbox.
   */
  defaultChecked?: boolean;
  /**
   * Callback fired when the checked state changes.
   */
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  /**
   * If true, the checkbox is in an indeterminate state.
   * @default false
   */
  indeterminate?: boolean;
  /**
   * The color of the checkbox.
   * @default 'primary'
   */
  color?: CheckboxColor;
  /**
   * The size of the checkbox.
   * @default 'medium'
   */
  size?: CheckboxSize;
  /**
   * If true, the checkbox is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Checkbox component - a selectable input.
 * Follows MUI Checkbox API conventions.
 *
 * Checkbox allows users to select one or more items from a list,
 * or to toggle a single option on/off.
 *
 * @example
 * // Uncontrolled checkbox
 * <Checkbox defaultChecked />
 *
 * @example
 * // Controlled checkbox
 * <Checkbox checked={checked} onChange={(e) => setChecked(e.target.checked)} />
 *
 * @example
 * // Indeterminate state
 * <Checkbox indeterminate checked={someItemsChecked} />
 *
 * @example
 * // Different colors
 * <Checkbox color="secondary" />
 * <Checkbox color="error" />
 */
export const Checkbox = forwardRef<HTMLInputElement, CheckboxProps>(function Checkbox(
  {
    checked: checkedProp,
    defaultChecked,
    onChange,
    indeterminate = false,
    color = 'primary',
    size = 'medium',
    disabled = false,
    className,
    ...props
  }: CheckboxProps,
  ref: React.Ref<HTMLInputElement>
) {
  const [checked, setChecked] = useControllableState<boolean>({
    value: checkedProp,
    defaultValue: defaultChecked ?? false,
    onChange: (value) => {
      // Create a synthetic event for the onChange callback
      if (onChange) {
        const event = { target: { checked: value } } as React.ChangeEvent<HTMLInputElement>;
        onChange(event);
      }
    },
  });

  const sizeStyles = SIZE_STYLES[size];
  const colorStyles = COLOR_STYLES[color];

  const baseStyles = cn(
    'relative inline-flex items-center justify-center',
    'cursor-pointer',
    'appearance-none',
    'border-2 border-nest-border rounded-nest-sm',
    'bg-nest-surface',
    'transition-colors duration-200',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none',
    sizeStyles,
    colorStyles,
    className
  );

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setChecked(event.target.checked);
  };

  // Handle indeterminate state via ref
  const inputRef = useRef<HTMLInputElement>(null);
  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.indeterminate = indeterminate;
    }
  }, [indeterminate]);

  // Combine refs
  const combinedRef = (el: HTMLInputElement | null) => {
    inputRef.current = el;
    if (ref) {
      if (typeof ref === 'function') {
        ref(el);
      } else {
        (ref as React.MutableRefObject<HTMLInputElement | null>).current = el;
      }
    }
  };

  return (
    <div className="inline-flex items-center gap-2">
      <input
        ref={combinedRef}
        type="checkbox"
        checked={checked}
        onChange={handleChange}
        disabled={disabled}
        className={baseStyles}
        {...props}
      />
      {checked && !indeterminate && (
        <Check className={cn('absolute pointer-events-none', size === 'small' ? 'size-3' : 'size-3.5')} />
      )}
      {indeterminate && (
        <Minus className={cn('absolute pointer-events-none', size === 'small' ? 'size-3' : 'size-3.5')} />
      )}
    </div>
  );
});
