import { forwardRef, type ReactNode } from 'react';
import { cn } from '../../lib/cn';
import { Check } from 'lucide-react';

export type StepperOrientation = 'horizontal' | 'vertical';

export interface StepperProps {
  /**
   * The active step (0-indexed).
   */
  activeStep?: number;
  /**
   * The orientation of the stepper.
   * @default 'horizontal'
   */
  orientation?: StepperOrientation;
  /**
   * The content of the stepper (Step components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Stepper component - displays progress through a sequence of steps.
 * Follows MUI Stepper API conventions.
 *
 * Stepper shows the user's progress through a multi-step process.
 *
 * @example
 * // Horizontal stepper
 * <Stepper activeStep={1}>
 *   <Step>Step 1</Step>
 *   <Step>Step 2</Step>
 *   <Step>Step 3</Step>
 * </Stepper>
 *
 * @example
 * // Vertical stepper
 * <Stepper activeStep={1} orientation="vertical">
 *   <Step>Step 1</Step>
 *   <Step>Step 2</Step>
 * </Stepper>
 */
export const Stepper = forwardRef<HTMLDivElement, StepperProps>(function Stepper(
  {
    activeStep = 0,
    orientation = 'horizontal',
    children,
    className,
    ...props
  }: StepperProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'flex',
    orientation === 'horizontal' ? 'flex-row' : 'flex-col',
    className
  );

  return (
    <div ref={ref as any} className={baseStyles} role="progressbar" aria-valuenow={activeStep} {...props}>
      {children}
    </div>
  );
});

export interface StepProps {
  /**
   * If true, the step is completed.
   */
  completed?: boolean;
  /**
   * If true, the step is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * The content of the step.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Step component - a single step within a Stepper.
 * Follows MUI Step API conventions.
 *
 * @example
 * <Step completed={completed}>
 *   <StepLabel>Step Label</StepLabel>
 * </Step>
 */
export const Step = forwardRef<HTMLDivElement, StepProps>(function Step(
  {
    completed = false,
    disabled = false,
    children,
    className,
    ...props
  }: StepProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'flex items-center',
    'relative',
    className
  );

  return (
    <div ref={ref as any} className={baseStyles} {...props}>
      {children}
    </div>
  );
});

export interface StepLabelProps {
  /**
   * If true, the step is completed.
   */
  completed?: boolean;
  /**
   * If true, the step is active.
   */
  active?: boolean;
  /**
   * If true, the step is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * The label content.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * StepLabel component - the label for a Step.
 * Follows MUI StepLabel API conventions.
 *
 * @example
 * <StepLabel>Step Title</StepLabel>
 *
 * @example
 * <StepLabel completed={completed} active={active}>
 *   Step Title
 * </StepLabel>
 */
export const StepLabel = forwardRef<HTMLDivElement, StepLabelProps>(function StepLabel(
  {
    completed = false,
    active = false,
    disabled = false,
    children,
    className,
    ...props
  }: StepLabelProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'flex items-center gap-3',
    disabled && 'opacity-50',
    className
  );

  return (
    <div ref={ref as any} className={baseStyles} {...props}>
      <StepIcon completed={completed} active={active} disabled={disabled} />
      <span className={cn(
        'text-sm font-medium',
        active && 'text-nest-primary',
        completed && 'text-nest-foreground',
        !active && !completed && 'text-nest-muted'
      )}>
        {children}
      </span>
    </div>
  );
});

export interface StepIconProps {
  /**
   * If true, the step is completed.
   */
  completed?: boolean;
  /**
   * If true, the step is active.
   */
  active?: boolean;
  /**
   * If true, the step is disabled.
   * @default false
   */
  disabled?: boolean;
}

/**
 * StepIcon component - the icon/number indicator for a Step.
 *
 * @example
 * <StepIcon completed={completed} active={active} />
 */
export const StepIcon = forwardRef<HTMLDivElement, StepIconProps>(function StepIcon(
  {
    completed = false,
    active = false,
    disabled = false,
    ...props
  }: StepIconProps,
  ref: React.Ref<HTMLDivElement>
) {
  const baseStyles = cn(
    'flex items-center justify-center',
    'size-8 rounded-full',
    'border-2',
    'transition-colors duration-200',
    'text-sm font-medium',
    completed && 'bg-nest-primary border-nest-primary text-white',
    active && !completed && 'border-nest-primary text-nest-primary',
    !active && !completed && 'border-nest-border text-nest-muted',
    disabled && 'opacity-50'
  );

  return (
    <div ref={ref as any} className={baseStyles} {...props}>
      {completed ? (
        <Check className="size-4" />
      ) : (
        <span>•</span>
      )}
    </div>
  );
});
