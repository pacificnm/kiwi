import { forwardRef, type HTMLAttributes, type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import { CheckCircle, AlertCircle, AlertTriangle, Info, X } from 'lucide-react';
import { IconButton } from '../inputs/IconButton';

export type AlertSeverity = 'success' | 'error' | 'warning' | 'info';
export type AlertVariant = 'filled' | 'outlined' | 'standard';

export interface AlertProps extends HTMLAttributes<HTMLDivElement> {
  /**
   * The content of the alert.
   */
  children: ReactNode;
  /**
   * The severity of the alert.
   * @default 'info'
   */
  severity?: AlertSeverity;
  /**
   * The visual style of the alert.
   * @default 'filled'
   */
  variant?: AlertVariant;
  /**
   * Optional icon to display. Pass `null` to hide the icon.
   */
  icon?: ReactNode;
  /**
   * Show a close button.
   * @default false
   */
  onClose?: () => void;
  /**
   * Action to display on the right side.
   */
  action?: ReactNode;
}

const SEVERITY_CONFIG: Record<
  AlertSeverity,
  { icon: React.ComponentType<{ className?: string }>; colorClass: string; bgClass: string }
> = {
  success: {
    icon: CheckCircle,
    colorClass: 'text-nest-success',
    bgClass: 'bg-nest-success',
  },
  error: {
    icon: AlertCircle,
    colorClass: 'text-nest-error',
    bgClass: 'bg-nest-error',
  },
  warning: {
    icon: AlertTriangle,
    colorClass: 'text-nest-warning',
    bgClass: 'bg-nest-warning',
  },
  info: {
    icon: Info,
    colorClass: 'text-nest-info',
    bgClass: 'bg-nest-info',
  },
};

const VARIANT_STYLES: Record<AlertVariant, Record<AlertSeverity, string>> = {
  filled: {
    success: 'bg-nest-success/10 text-nest-success border-nest-success',
    error: 'bg-nest-error/10 text-nest-error border-nest-error',
    warning: 'bg-nest-warning/10 text-nest-warning border-nest-warning',
    info: 'bg-nest-info/10 text-nest-info border-nest-info',
  },
  outlined: {
    success: 'bg-transparent text-nest-success border border-nest-success',
    error: 'bg-transparent text-nest-error border border-nest-error',
    warning: 'bg-transparent text-nest-warning border border-nest-warning',
    info: 'bg-transparent text-nest-info border border-nest-info',
  },
  standard: {
    success: 'bg-transparent text-nest-success',
    error: 'bg-transparent text-nest-error',
    warning: 'bg-transparent text-nest-warning',
    info: 'bg-transparent text-nest-info',
  },
};

/**
 * Alert component for displaying messages.
 * Follows MUI Alert API conventions.
 *
 * @example
 * // Basic usage
 * <Alert severity="success">Operation completed!</Alert>
 *
 * @example
 * // With close button
 * <Alert severity="error" onClose={() => setAlert(null)}>
 *   Something went wrong
 * </Alert>
 *
 * @example
 * // With action
 * <Alert
 *   severity="warning"
 *   action={<Button size="small">Undo</Button>}
 * >
 *   Item will be deleted
 * </Alert>
 */
export const Alert = forwardRef<HTMLDivElement, AlertProps>(function Alert(
  {
    className,
    children,
    severity = 'info',
    variant = 'filled',
    icon,
    onClose,
    action,
    role = 'alert',
    ...props
  },
  ref
) {
  const config = SEVERITY_CONFIG[severity];
  const variantStyles = VARIANT_STYLES[variant][severity];

  const defaultIcon = icon !== undefined ? icon : <config.icon className="size-5 shrink-0" />;

  return (
    <div
      ref={ref}
      className={cn(
        'flex items-start gap-3 rounded-nest-md p-3',
        variantStyles,
        className
      )}
      role={role}
      {...props}
    >
      {defaultIcon && <div className={cn('mt-0.5', config.colorClass)}>{defaultIcon}</div>}
      <div className="flex-1 text-sm">{children}</div>
      <div className="flex items-center gap-1">
        {action}
        {onClose && (
          <IconButton
            aria-label="Close alert"
            onClick={onClose}
            size="small"
            color={severity}
          >
            <X className="size-4" />
          </IconButton>
        )}
      </div>
    </div>
  );
});
