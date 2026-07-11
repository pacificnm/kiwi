import { forwardRef, type InputHTMLAttributes, type ReactNode, useId } from 'react';
import { cn } from '../../lib/cn';

export type TextFieldVariant = 'outlined' | 'filled' | 'standard';
export type TextFieldSize = 'small' | 'medium';

export interface TextFieldProps
  extends Omit<InputHTMLAttributes<HTMLInputElement | HTMLTextAreaElement>, 'size'> {
  /**
   * The label text displayed above/before the input.
   */
  label?: string;
  /**
   * Helper text displayed below the input.
   */
  helperText?: ReactNode;
  /**
   * Error message displayed below the input (overrides helperText).
   */
  error?: string;
  /**
   * The visual style of the input.
   * @default 'outlined'
   */
  variant?: TextFieldVariant;
  /**
   * The size of the input.
   * @default 'medium'
   */
  size?: TextFieldSize;
  /**
   * Icon to display before the input content.
   */
  startAdornment?: ReactNode;
  /**
   * Icon to display after the input content.
   */
  endAdornment?: ReactNode;
  /**
   * Renders a multiline textarea instead of a single-line input.
   * @default false
   */
  multiline?: boolean;
  /**
   * Number of rows for multiline textarea.
   * @default 3
   */
  rows?: number;
  /**
   * Full width input.
   * @default false
   */
  fullWidth?: boolean;
}

const VARIANT_STYLES: Record<TextFieldVariant, string> = {
  outlined: 'border border-nest-border bg-nest-background focus-within:border-nest-primary focus-within:ring-1 focus-within:ring-nest-primary',
  filled: 'border-0 bg-nest-surface focus-within:bg-nest-background',
  standard: 'border-b border-nest-border bg-transparent focus-within:border-nest-primary',
};

const SIZE_STYLES: Record<TextFieldSize, string> = {
  small: 'px-2 py-1.5 text-xs',
  medium: 'px-3 py-2 text-sm',
};

/**
 * TextField component for user text input.
 * Follows MUI TextField API conventions.
 *
 * @example
 * // Basic usage
 * <TextField label="Email" value={email} onChange={(e) => setEmail(e.target.value)} />
 *
 * @example
 * // With error
 * <TextField label="Email" error="Invalid email address" />
 *
 * @example
 * // Multiline
 * <TextField label="Description" multiline rows={4} />
 *
 * @example
 * // With adornment
 * <TextField
 *   label="Search"
 *   startAdornment={<SearchIcon />}
 *   endAdornment={<ClearIcon />}
 * />
 */
export const TextField = forwardRef<HTMLInputElement | HTMLTextAreaElement, TextFieldProps>(
  function TextField(
    {
      className,
      label,
      helperText,
      error,
      variant = 'outlined',
      size = 'medium',
      startAdornment,
      endAdornment,
      multiline = false,
      rows = 3,
      fullWidth = false,
      id: idProp,
      ...props
    },
    ref
  ) {
    const generatedId = useId();
    const id = idProp || generatedId;
    const hasError = !!error;

    const baseStyles =
      'rounded-nest-md transition-all duration-150 focus:outline-none text-nest-foreground placeholder:text-nest-muted';

    const variantStyles = VARIANT_STYLES[variant];

    const sizeStyles = SIZE_STYLES[size];

    const fullWidthStyles = fullWidth ? 'w-full' : '';

    const errorStyles = hasError ? 'border-nest-error focus-within:border-nest-error focus-within:ring-nest-error' : '';

    const inputStyles = cn(
      baseStyles,
      variantStyles,
      sizeStyles,
      fullWidthStyles,
      errorStyles,
      className
    );

    const adornedInputStyles = cn(
      inputStyles,
      'flex items-center gap-2',
      startAdornment || endAdornment ? 'px-2' : ''
    );

    const InputComponent = multiline ? 'textarea' : 'input';

    const inputProps = {
      id,
      className: cn('w-full bg-transparent outline-none', multiline ? `resize-y min-h-[${rows * 1.5}em]` : ''),
      rows: multiline ? rows : undefined,
      'aria-invalid': hasError || undefined,
      'aria-describedby': error ? `${id}-error` : helperText ? `${id}-helper` : undefined,
      ...props,
    };

    return (
      <div className={cn('flex flex-col gap-1', fullWidthStyles)}>
        {label && (
          <label
            htmlFor={id}
            className="text-sm font-medium text-nest-foreground"
          >
            {label}
          </label>
        )}
        <div className={cn('flex items-center', adornedInputStyles)}>
          {startAdornment && (
            <span className="flex shrink-0 items-center text-nest-muted">
              {startAdornment}
            </span>
          )}
          <InputComponent ref={ref as never} {...inputProps} />
          {endAdornment && (
            <span className="flex shrink-0 items-center text-nest-muted">
              {endAdornment}
            </span>
          )}
        </div>
        {(error || helperText) && (
          <p
            id={error ? `${id}-error` : `${id}-helper`}
            className={cn('text-xs', hasError ? 'text-nest-error' : 'text-nest-muted')}
            role={hasError ? 'alert' : undefined}
          >
            {error || helperText}
          </p>
        )}
      </div>
    );
  }
);
