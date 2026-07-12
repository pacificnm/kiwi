import { forwardRef, type ReactNode, useEffect, useState } from 'react';
import { cn } from '../../lib/cn';

export interface FadeProps {
  /**
   * If true, the component is shown.
   */
  in?: boolean;
  /**
   * The content of the component.
   */
  children?: ReactNode;
  /**
   * The duration of the transition in milliseconds.
   * @default 300
   */
  timeout?: number;
  /**
   * If true, the child is unmounted after the exit transition.
   * @default false
   */
  unmountOnExit?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Fade component - fades content in and out.
 * Follows MUI Fade API conventions.
 *
 * Fade shows/hides content with an opacity transition.
 *
 * @example
 * // Basic fade
 * <Fade in={open}><div>Content</div></Fade>
 *
 * @example
 * // With timeout
 * <Fade in={open} timeout={500}><div>Content</div></Fade>
 */
export const Fade = forwardRef<HTMLDivElement, FadeProps>(function Fade(
  {
    in: inProp = false,
    children,
    timeout = 300,
    unmountOnExit = false,
    className,
    ...props
  }: FadeProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [mounted, setMounted] = useState(inProp);
  const [visible, setVisible] = useState(inProp);

  useEffect(() => {
    if (inProp) {
      setMounted(true);
      // Trigger reflow for transition
      requestAnimationFrame(() => {
        setVisible(true);
      });
    } else {
      setVisible(false);
      const timer = setTimeout(() => {
        setMounted(false);
      }, timeout);
      return () => clearTimeout(timer);
    }
  }, [inProp, timeout]);

  if (unmountOnExit && !mounted) {
    return null;
  }

  const baseStyles = cn(
    'transition-opacity duration-300 ease-in-out',
    visible ? 'opacity-100' : 'opacity-0',
    className
  );

  return (
    <div
      ref={ref as any}
      className={baseStyles}
      style={{ transitionDuration: `${timeout}ms` }}
      {...props}
    >
      {children}
    </div>
  );
});

export interface GrowProps {
  /**
   * If true, the component is shown.
   */
  in?: boolean;
  /**
   * The content of the component.
   */
  children?: ReactNode;
  /**
   * The duration of the transition in milliseconds.
   * @default 300
   */
  timeout?: number;
  /**
   * If true, the child is unmounted after the exit transition.
   * @default false
   */
  unmountOnExit?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Grow component - grows content in and out.
 * Follows MUI Grow API conventions.
 *
 * Grow shows/hides content with a scale and opacity transition.
 *
 * @example
 * // Basic grow
 * <Grow in={open}><div>Content</div></Grow>
 *
 * @example
 * // With timeout
 * <Grow in={open} timeout={400}><div>Content</div></Grow>
 */
export const Grow = forwardRef<HTMLDivElement, GrowProps>(function Grow(
  {
    in: inProp = false,
    children,
    timeout = 300,
    unmountOnExit = false,
    className,
    ...props
  }: GrowProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [mounted, setMounted] = useState(inProp);
  const [visible, setVisible] = useState(inProp);

  useEffect(() => {
    if (inProp) {
      setMounted(true);
      requestAnimationFrame(() => {
        setVisible(true);
      });
    } else {
      setVisible(false);
      const timer = setTimeout(() => {
        setMounted(false);
      }, timeout);
      return () => clearTimeout(timer);
    }
  }, [inProp, timeout]);

  if (unmountOnExit && !mounted) {
    return null;
  }

  const baseStyles = cn(
    'transition-all duration-300 ease-out',
    'transform-origin-center',
    visible ? 'opacity-100 scale-100' : 'opacity-0 scale-95',
    className
  );

  return (
    <div
      ref={ref as any}
      className={baseStyles}
      style={{ transitionDuration: `${timeout}ms` }}
      {...props}
    >
      {children}
    </div>
  );
});

export interface CollapseProps {
  /**
   * If true, the component is shown.
   */
  in?: boolean;
  /**
   * The content of the component.
   */
  children?: ReactNode;
  /**
   * The duration of the transition in milliseconds.
   * @default 300
   */
  timeout?: number;
  /**
   * If true, the child is unmounted after the exit transition.
   * @default false
   */
  unmountOnExit?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
  /**
   * The orientation of the collapse.
   * @default 'vertical'
   */
  orientation?: 'vertical' | 'horizontal';
}

/**
 * Collapse component - collapses content in and out.
 * Follows MUI Collapse API conventions.
 *
 * Collapse shows/hides content with a height (or width) transition.
 *
 * @example
 * // Basic collapse
 * <Collapse in={open}><div>Content</div></Collapse>
 *
 * @example
 * // Horizontal collapse
 * <Collapse in={open} orientation="horizontal"><div>Content</div></Collapse>
 */
export const Collapse = forwardRef<HTMLDivElement, CollapseProps>(function Collapse(
  {
    in: inProp = false,
    children,
    timeout = 300,
    unmountOnExit = false,
    className,
    orientation = 'vertical',
    ...props
  }: CollapseProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [mounted, setMounted] = useState(inProp);
  const [visible, setVisible] = useState(inProp);

  useEffect(() => {
    if (inProp) {
      setMounted(true);
      requestAnimationFrame(() => {
        setVisible(true);
      });
    } else {
      setVisible(false);
      const timer = setTimeout(() => {
        setMounted(false);
      }, timeout);
      return () => clearTimeout(timer);
    }
  }, [inProp, timeout]);

  if (unmountOnExit && !mounted) {
    return null;
  }

  const isVertical = orientation === 'vertical';

  const baseStyles = cn(
    'transition-all duration-300 ease-out',
    'overflow-hidden',
    visible
      ? isVertical
        ? 'grid grid-rows-[1fr] opacity-100'
        : 'opacity-100'
      : isVertical
        ? 'grid-rows-[0fr] opacity-0'
        : 'w-0 opacity-0',
    className
  );

  return (
    <div
      ref={ref as any}
      className={baseStyles}
      style={{ transitionDuration: `${timeout}ms` }}
      {...props}
    >
      <div className={cn('overflow-hidden', !visible && 'invisible')}>
        {children}
      </div>
    </div>
  );
});
