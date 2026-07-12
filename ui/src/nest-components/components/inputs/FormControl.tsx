import { forwardRef, type ReactNode, type ElementType } from 'react';
import { cn } from '../../lib/cn';

export interface FormControlProps {
  /**
   * The content of the form control.
   */
  children?: ReactNode;
  /**
   * If true, the form control is in an error state.
   * @default false
   */
  error?: boolean;
  /**
   * If true, the form control is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, the form control is required.
   * @default false
   */
  required?: boolean;
  /**
   * If true, the label is displayed in a focused state.
   * @default false
   */
  focused?: boolean;
  /**
   * If true, the form control takes full width.
   * @default false
   */
  fullWidth?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The component to render as.
   * @default 'div'
   */
  component?: ElementType;
}

/**
 * FormControl component - provides context for form fields.
 * Follows MUI FormControl API conventions.
 *
 * FormControl wraps form inputs and their labels/helper text,
 * providing consistent spacing and state styling.
 *
 * @example
 * // Basic form control
 * <FormControl>
 *   <FormLabel>Name</FormLabel>
 *   <TextField placeholder="Enter name" />
 *   <FormHelperText>Optional helper text</FormHelperText>
 * </FormControl>
 *
 * @example
 * // Error state
 * <FormControl error>
 *   <FormLabel>Email</FormLabel>
 *   <TextField placeholder="Enter email" />
 *   <FormHelperText>Invalid email format</FormHelperText>
 * </FormControl>
 */
export const FormControl = forwardRef<HTMLDivElement, FormControlProps>(function FormControl(
  {
    children,
    error = false,
    disabled = false,
    required = false,
    focused = false,
    fullWidth = false,
    className,
    component = 'div',
    ...props
  }: FormControlProps & React.HTMLAttributes<HTMLDivElement>,
  ref: React.Ref<HTMLDivElement>
) {
  const Component = component;

  const baseStyles = 'flex flex-col gap-1';

  return (
    <Component
      ref={ref as any}
      className={cn(
        baseStyles,
        disabled && 'opacity-60 pointer-events-none',
        fullWidth && 'w-full',
        className
      )}
      {...props}
    >
      {children}
    </Component>
  );
});

export interface FormLabelProps {
  /**
   * The content of the label.
   */
  children?: ReactNode;
  /**
   * If true, the label is in an error state.
   * @default false
   */
  error?: boolean;
  /**
   * If true, the label is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, the label is focused.
   * @default false
   */
  focused?: boolean;
  /**
   * If true, the label is required (shows asterisk).
   * @default false
   */
  required?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The HTML for attribute.
   */
  htmlFor?: string;
}

/**
 * FormLabel component - a label for form inputs.
 * Follows MUI FormLabel API conventions.
 *
 * FormLabel displays text that identifies what the form field is for.
 *
 * @example
 * // Basic label
 * <FormLabel htmlFor="email">Email</FormLabel>
 *
 * @example
 * // Required label
 * <FormLabel required htmlFor="email">Email</FormLabel>
 *
 * @example
 * // Error state
 * <FormLabel error>Email (invalid)</FormLabel>
 */
export const FormLabel = forwardRef<HTMLLabelElement, FormLabelProps>(function FormLabel(
  {
    children,
    error = false,
    disabled = false,
    focused = false,
    required = false,
    className,
    htmlFor,
    ...props
  }: FormLabelProps & React.LabelHTMLAttributes<HTMLLabelElement>,
  ref: React.Ref<HTMLLabelElement>
) {
  const baseStyles = 'text-sm font-medium text-nest-foreground';

  return (
    <label
      ref={ref as any}
      htmlFor={htmlFor}
      className={cn(
        baseStyles,
        error && 'text-nest-error',
        disabled && 'text-nest-muted cursor-not-allowed',
        focused && 'text-nest-primary',
        className
      )}
      {...props}
    >
      {children}
      {required && <span className="ml-1 text-nest-error">*</span>}
    </label>
  );
});

export interface FormHelperTextProps {
  /**
   * The content of the helper text.
   */
  children?: ReactNode;
  /**
   * If true, the helper text is in an error state.
   * @default false
   */
  error?: boolean;
  /**
   * If true, the helper text is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * If true, the helper text is visually hidden but still accessible.
   * @default false
   */
  visuallyHidden?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * FormHelperText component - helper or error text for form fields.
 * Follows MUI FormHelperText API conventions.
 *
 * FormHelperText displays additional information or error messages
 * below a form field.
 *
 * @example
 * // Helper text
 * <FormHelperText>We'll never share your email</FormHelperText>
 *
 * @example
 * // Error text
 * <FormHelperText error>Please enter a valid email</FormHelperText>
 *
 * @example
 * // Visually hidden (for screen readers only)
 * <FormHelperText visuallyHidden>This field is required</FormHelperText>
 */
export const FormHelperText = forwardRef<HTMLParagraphElement, FormHelperTextProps>(function FormHelperText(
  {
    children,
    error = false,
    disabled = false,
    visuallyHidden = false,
    className,
    ...props
  }: FormHelperTextProps & React.HTMLAttributes<HTMLParagraphElement>,
  ref: React.Ref<HTMLParagraphElement>
) {
  const baseStyles = 'text-xs text-nest-muted mt-0.5';

  return (
    <p
      ref={ref as any}
      className={cn(
        baseStyles,
        error && 'text-nest-error',
        disabled && 'opacity-60',
        visuallyHidden && 'sr-only',
        className
      )}
      {...props}
    >
      {children}
    </p>
  );
});
