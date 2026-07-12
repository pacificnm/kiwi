import { forwardRef, type InputHTMLAttributes } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';

export type SwitchColor = 'primary' | 'secondary' | 'accent' | 'success' | 'warning' | 'error' | 'info';
export type SwitchSize = 'small' | 'medium';

const COLOR_STYLES: Record<SwitchColor, string> = {
  primary: 'checked:bg-nest-primary',
  secondary: 'checked:bg-nest-secondary',
  accent: 'checked:bg-nest-accent',
  success: 'checked:bg-nest-success',
  warning: 'checked:bg-nest-warning',
  error: 'checked:bg-nest-error',
  info: 'checked:bg-nest-info',
};

const SIZE_STYLES: Record<SwitchSize, string> = {
  small: 'h-4 w-7',
  medium: 'h-6 w-11',
};

const THUMB_STYLES: Record<SwitchSize, string> = {
  small: 'size-3 checked:translate-x-3',
  medium: 'size-5 checked:translate-x-5',
};

export interface SwitchProps extends Omit<InputHTMLAttributes<HTMLInputElement>, 'size'> {
  /**
   * Whether the switch is checked.
   */
  checked?: boolean;
  /**
   * Default checked state for uncontrolled switch.
   */
  defaultChecked?: boolean;
  /**
   * Callback fired when the checked state changes.
   */
  onChange?: (event: React.ChangeEvent<HTMLInputElement>) => void;
  /**
   * The color of the switch.
   * @default 'primary'
   */
  color?: SwitchColor;
  /**
   * The size of the switch.
   * @default 'medium'
   */
  size?: SwitchSize;
  /**
   * If true, the switch is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Switch component - a toggle input.
 * Follows MUI Switch API conventions.
 *
 * Switch represents a binary on/off state, commonly used for settings.
 *
 * @example
 * // Uncontrolled switch
 * <Switch defaultChecked />
 *
 * @example
 * // Controlled switch
 * <Switch checked={checked} onChange={(e) => setChecked(e.target.checked)} />
 *
 * @example
 * // Different colors
 * <Switch color="secondary" />
 * <Switch color="success" />
 *
 * @example
 * // Different sizes
 * <Switch size="small" />
 * <Switch size="medium" />
 */
export const Switch = forwardRef<HTMLInputElement, SwitchProps>(function Switch(
  {
    checked: checkedProp,
    defaultChecked,
    onChange,
    color = 'primary',
    size = 'medium',
    disabled = false,
    className,
    ...props
  }: SwitchProps,
  ref: React.Ref<HTMLInputElement>
) {
  const [checked, setChecked] = useControllableState<boolean>({
    value: checkedProp,
    defaultValue: defaultChecked ?? false,
    onChange: (value) => {
      if (onChange) {
        const event = { target: { checked: value } } as React.ChangeEvent<HTMLInputElement>;
        onChange(event);
      }
    },
  });

  const sizeStyles = SIZE_STYLES[size];
  const thumbStyles = THUMB_STYLES[size];
  const colorStyles = COLOR_STYLES[color];

  const baseStyles = cn(
    'relative inline-flex items-center',
    'cursor-pointer',
    'appearance-none',
    'rounded-full',
    'bg-nest-muted',
    'transition-colors duration-200',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2',
    'disabled:opacity-50 disabled:cursor-not-allowed disabled:pointer-events-none',
    sizeStyles,
    colorStyles,
    className
  );

  const thumbBaseStyles = cn(
    'absolute left-0.5 top-0.5',
    'rounded-full',
    'bg-white',
    'shadow-sm',
    'transition-transform duration-200',
    thumbStyles
  );

  const handleChange = (event: React.ChangeEvent<HTMLInputElement>) => {
    setChecked(event.target.checked);
  };

  return (
    <div className="inline-flex items-center">
      <input
        ref={ref}
        type="checkbox"
        checked={checked}
        onChange={handleChange}
        disabled={disabled}
        className={baseStyles}
        role="switch"
        {...props}
      />
      <span className={thumbBaseStyles} />
    </div>
  );
});
