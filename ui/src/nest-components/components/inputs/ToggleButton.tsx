import { forwardRef, type ReactNode, type ButtonHTMLAttributes, createContext, useContext } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';

export type ToggleButtonSize = 'small' | 'medium' | 'large';
export type ToggleButtonColor = 'primary' | 'secondary' | 'accent' | 'success' | 'error';

interface ToggleButtonGroupContextValue {
  value?: string | string[];
  onChange?: (value: string | string[]) => void;
  disabled?: boolean;
  color?: ToggleButtonColor;
  size?: ToggleButtonSize;
  exclusive?: boolean;
}

const ToggleButtonGroupContext = createContext<ToggleButtonGroupContextValue | undefined>(undefined);

const COLOR_STYLES: Record<ToggleButtonColor, string> = {
  primary: 'text-nest-primary border-nest-primary bg-nest-primary/10 hover:bg-nest-primary/20',
  secondary: 'text-nest-secondary border-nest-secondary bg-nest-secondary/10 hover:bg-nest-secondary/20',
  accent: 'text-nest-accent border-nest-accent bg-nest-accent/10 hover:bg-nest-accent/20',
  success: 'text-nest-success border-nest-success bg-nest-success/10 hover:bg-nest-success/20',
  error: 'text-nest-error border-nest-error bg-nest-error/10 hover:bg-nest-error/20',
};

const SIZE_STYLES: Record<ToggleButtonSize, string> = {
  small: 'text-xs px-2 py-1',
  medium: 'text-sm px-3 py-1.5',
  large: 'text-base px-4 py-2',
};

export interface ToggleButtonGroupProps {
  /**
   * The selected value(s).
   */
  value?: string | string[];
  /**
   * Default value for uncontrolled group.
   */
  defaultValue?: string | string[];
  /**
   * Callback fired when the selected value changes.
   */
  onChange?: (value: string | string[]) => void;
  /**
   * If true, only one button can be selected at a time.
   * @default false
   */
  exclusive?: boolean;
  /**
   * If true, the buttons are disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * The color of the buttons.
   * @default 'primary'
   */
  color?: ToggleButtonColor;
  /**
   * The size of the buttons.
   * @default 'medium'
   */
  size?: ToggleButtonSize;
  /**
   * If true, the button group is displayed in a row.
   * @default false
   */
  row?: boolean;
  /**
   * The content of the group (ToggleButton components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * ToggleButtonGroup component - groups ToggleButton components.
 * Follows MUI ToggleButtonGroup API conventions.
 *
 * ToggleButtonGroup manages the selection state of its child buttons.
 *
 * @example
 * // Single selection
 * <ToggleButtonGroup value={value} onChange={(v) => setValue(v)}>
 *   <ToggleButton value="left">Left</ToggleButton>
 *   <ToggleButton value="center">Center</ToggleButton>
 * </ToggleButtonGroup>
 *
 * @example
 * // Multiple selection (exclusive=false)
 * <ToggleButtonGroup exclusive={false} defaultValue={[]}>
 *   <ToggleButton value="bold">Bold</ToggleButton>
 *   <ToggleButton value="italic">Italic</ToggleButton>
 * </ToggleButtonGroup>
 */
export const ToggleButtonGroup = forwardRef<HTMLDivElement, ToggleButtonGroupProps>(function ToggleButtonGroup(
  {
    value: valueProp,
    defaultValue,
    onChange,
    exclusive = false,
    disabled = false,
    color = 'primary',
    size = 'medium',
    row = false,
    children,
    className,
    ...props
  }: ToggleButtonGroupProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [value, setValue] = useControllableState<string | string[]>({
    value: valueProp,
    defaultValue: defaultValue ?? (exclusive ? '' : []),
    onChange,
  });

  const handleChange = (newValue: string | string[]) => {
    setValue(newValue);
  };

  const baseStyles = cn(
    'inline-flex',
    'border border-nest-border rounded-nest-md',
    'bg-nest-surface',
    row ? 'flex-row' : 'flex-col',
    className
  );

  return (
    <ToggleButtonGroupContext.Provider
      value={{
        value,
        onChange: handleChange,
        disabled,
        color,
        size,
        exclusive,
      }}
    >
      <div ref={ref as any} className={baseStyles} role="group" {...props}>
        {children}
      </div>
    </ToggleButtonGroupContext.Provider>
  );
});

export interface ToggleButtonProps extends Omit<ButtonHTMLAttributes<HTMLButtonElement>, 'value' | 'color'> {
  /**
   * The value of the button.
   */
  value?: string;
  /**
   * The label to display.
   */
  label?: ReactNode;
  /**
   * The color of the button.
   * @default inherits from group or 'primary'
   */
  color?: ToggleButtonColor;
  /**
   * The size of the button.
   * @default inherits from group or 'medium'
   */
  size?: ToggleButtonSize;
  /**
   * If true, the button is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * ToggleButton component - a selectable button.
 * Follows MUI ToggleButton API conventions.
 *
 * ToggleButton should be used within a ToggleButtonGroup.
 *
 * @example
 * <ToggleButton value="bold" aria-label="Bold">
 *   <BoldIcon />
 * </ToggleButton>
 */
export const ToggleButton = forwardRef<HTMLButtonElement, ToggleButtonProps>(function ToggleButton(
  {
    value,
    label,
    children,
    color: colorProp,
    size: sizeProp,
    disabled: disabledProp,
    className,
    ...props
  }: ToggleButtonProps,
  ref: React.Ref<HTMLButtonElement>
) {
  const context = useContext(ToggleButtonGroupContext);

  const isInGroup = context !== undefined;
  const groupValue = context?.value;
  const groupOnChange = context?.onChange;
  const groupDisabled = context?.disabled;
  const groupColor = context?.color;
  const groupSize = context?.size;
  const exclusive = context?.exclusive ?? false;

  const isSelected = isInGroup
    ? exclusive
      ? groupValue === value
      : Array.isArray(groupValue) && groupValue.includes(value ?? '')
    : false;

  const isDisabled = disabledProp ?? groupDisabled ?? false;
  const color = colorProp ?? groupColor ?? 'primary';
  const size = sizeProp ?? groupSize ?? 'medium';

  const colorStyles = COLOR_STYLES[color];
  const sizeStyles = SIZE_STYLES[size];

  const handleClick = () => {
    if (!isInGroup || !groupOnChange || value === undefined) return;

    if (exclusive) {
      // Single selection: toggle on/off or just select
      if (groupValue === value) {
        groupOnChange('');
      } else {
        groupOnChange(value);
      }
    } else {
      // Multiple selection
      const current = Array.isArray(groupValue) ? groupValue : [];
      if (current.includes(value)) {
        groupOnChange(current.filter((v) => v !== value));
      } else {
        groupOnChange([...current, value]);
      }
    }
  };

  const baseStyles = cn(
    'inline-flex items-center justify-center gap-2',
    'border-0 border-r border-nest-border',
    'bg-transparent',
    'text-nest-muted',
    'transition-colors duration-150',
    'hover:bg-nest-surface hover:text-nest-foreground',
    'focus:outline-none focus:z-10 focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-1',
    'first:rounded-l-nest-md last:rounded-r-nest-md last:border-r-0',
    sizeStyles,
    isSelected && colorStyles,
    isDisabled && 'opacity-50 cursor-not-allowed pointer-events-none',
    className
  );

  return (
    <button
      ref={ref}
      type="button"
      value={value}
      disabled={isDisabled}
      onClick={handleClick}
      className={baseStyles}
      aria-pressed={isSelected}
      {...props}
    >
      {children}
      {label && <span>{label}</span>}
    </button>
  );
});
