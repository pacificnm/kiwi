import { forwardRef, type InputHTMLAttributes, type ReactNode, createContext, useContext } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';

export type RadioColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info';
export type RadioSize = 'small' | 'medium';

interface RadioGroupContextValue {
  value?: string;
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  name?: string;
  disabled?: boolean;
  color?: RadioColor;
  size?: RadioSize;
}

const RadioGroupContext = createContext<RadioGroupContextValue | undefined>(undefined);

const COLOR_STYLES: Record<RadioColor, string> = {
  primary: 'checked:bg-nest-primary checked:border-nest-primary checked:hover:bg-nest-primary/90',
  secondary: 'checked:bg-nest-secondary checked:border-nest-secondary checked:hover:bg-nest-secondary/90',
  accent: 'checked:bg-nest-accent checked:border-nest-accent checked:hover:bg-nest-accent/90',
  success: 'checked:bg-nest-success checked:border-nest-success checked:hover:bg-nest-success/90',
  warning: 'checked:bg-nest-warning checked:border-nest-warning checked:hover:bg-nest-warning/90',
  error: 'checked:bg-nest-error checked:border-nest-error checked:hover:bg-nest-error/90',
  info: 'checked:bg-nest-info checked:border-nest-info checked:hover:bg-nest-info/90',
};

const SIZE_STYLES: Record<RadioSize, string> = {
  small: 'size-4',
  medium: 'size-5',
};

export interface RadioGroupProps {
  /**
   * The value of the selected radio.
   */
  value?: string;
  /**
   * Default value for uncontrolled group.
   */
  defaultValue?: string;
  /**
   * Callback fired when the selected value changes.
   */
  onChange?: (value: string, event: React.ChangeEvent<HTMLInputElement>) => void;
  /**
   * The name attribute of the radio inputs.
   */
  name?: string;
  /**
   * If true, all radios in the group are disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * The color of the radios.
   * @default 'primary'
   */
  color?: RadioColor;
  /**
   * The size of the radios.
   * @default 'medium'
   */
  size?: RadioSize;
  /**
   * If true, the radios are displayed in a row.
   * @default false
   */
  row?: boolean;
  /**
   * The content of the radio group (Radio components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * RadioGroup component - groups Radio buttons together.
 * Follows MUI RadioGroup API conventions.
 *
 * RadioGroup manages the selection state of its child Radio buttons,
 * ensuring only one option can be selected at a time.
 *
 * @example
 * // Controlled radio group
 * <RadioGroup value={value} onChange={(v) => setValue(v)}>
 *   <Radio value="one" label="Option One" />
 *   <Radio value="two" label="Option Two" />
 * </RadioGroup>
 *
 * @example
 * // Row layout
 * <RadioGroup row defaultValue="a">
 *   <Radio value="a" label="A" />
 *   <Radio value="b" label="B" />
 * </RadioGroup>
 */
export const RadioGroup = forwardRef<HTMLDivElement, RadioGroupProps>(function RadioGroup(
  {
    value: valueProp,
    defaultValue,
    onChange,
    name,
    disabled = false,
    color = 'primary',
    size = 'medium',
    row = false,
    children,
    className,
    ...props
  }: RadioGroupProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [value, setValue] = useControllableState<string>({
    value: valueProp,
    defaultValue,
  });

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    onChange?.(event.target.value, event);
    setValue(event.target.value);
  };

  const baseStyles = cn(
    'flex gap-4',
    row && 'flex-row',
    !row && 'flex-col',
    className
  );

  return (
    <RadioGroupContext.Provider
      value={{
        value,
        onChange: handleChange,
        name,
        disabled,
        color,
        size,
      }}
    >
      <div ref={ref as any} className={baseStyles} role="radiogroup" {...props}>
        {children}
      </div>
    </RadioGroupContext.Provider>
  );
});

export interface RadioProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /**
   * The value of the radio.
   */
  value?: string;
  /**
   * Whether the radio is checked.
   */
  checked?: boolean;
  /**
   * Default checked state for uncontrolled radio.
   */
  defaultChecked?: boolean;
  /**
   * Callback fired when the radio is checked.
   */
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  /**
   * The label to display next to the radio.
   */
  label?: ReactNode;
  /**
   * The color of the radio.
   * @default inherits from RadioGroup or 'primary'
   */
  color?: RadioColor;
  /**
   * The size of the radio.
   * @default inherits from RadioGroup or 'medium'
   */
  size?: RadioSize;
  /**
   * If true, the radio is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Radio component - a single selectable option.
 * Follows MUI Radio API conventions.
 *
 * Radio should be used within a RadioGroup for proper selection behavior.
 *
 * @example
 * // Basic radio
 * <Radio value="option" label="Option Label" />
 *
 * @example
 * // Standalone radio (without group)
 * <Radio name="standalone" checked={checked} onChange={(e) => setChecked(e.target.checked)} />
 */
export const Radio = forwardRef<HTMLInputElement, RadioProps>(function Radio(
  {
    value,
    checked: checkedProp,
    defaultChecked,
    onChange,
    label,
    color: colorProp,
    size: sizeProp,
    disabled: disabledProp,
    className,
    ...props
  }: RadioProps,
  ref: React.Ref<HTMLInputElement>
) {
  const context = useContext(RadioGroupContext);

  const isInGroup = context !== undefined;
  const groupValue = context?.value;
  const groupOnChange = context?.onChange;
  const groupName = context?.name;
  const groupDisabled = context?.disabled;
  const groupColor = context?.color;
  const groupSize = context?.size;

  const isChecked = isInGroup ? groupValue === value : (checkedProp ?? defaultChecked ?? false);
  const isDisabled = disabledProp ?? groupDisabled ?? false;
  const color = colorProp ?? groupColor ?? 'primary';
  const size = sizeProp ?? groupSize ?? 'medium';

  const colorStyles = COLOR_STYLES[color];
  const sizeStyles = SIZE_STYLES[size];

  const baseStyles = cn(
    'relative inline-flex items-center justify-center',
    'cursor-pointer',
    'appearance-none',
    'border-2 border-nest-border rounded-full',
    'bg-nest-surface',
    'transition-colors duration-200',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none',
    sizeStyles,
    colorStyles,
    className
  );

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    if (isInGroup && groupOnChange) {
      groupOnChange(event);
    }
    onChange?.(event);
  };

  return (
    <label className={cn('inline-flex items-center gap-2 cursor-pointer', isDisabled && 'cursor-not-allowed opacity-60')}>
      <input
        ref={ref}
        type="radio"
        name={isInGroup ? groupName : props.name}
        value={value}
        checked={isChecked}
        onChange={handleChange}
        disabled={isDisabled}
        className={baseStyles}
        {...props}
      />
      {label && <span className="text-sm text-nest-foreground">{label}</span>}
    </label>
  );
});
