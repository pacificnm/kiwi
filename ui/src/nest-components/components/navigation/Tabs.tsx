import { forwardRef, type ReactNode, createContext, useContext } from 'react';
import { cn } from '../../lib/cn';
import { useControllableState } from '../../hooks/useControllableState';

export type TabsVariant = 'standard' | 'fullWidth';
export type TabsOrientation = 'horizontal' | 'vertical';

interface TabsContextValue {
  value?: string | number;
  onChange?: (value: string | number) => void;
  variant?: TabsVariant;
  orientation?: TabsOrientation;
}

const TabsContext = createContext<TabsContextValue | undefined>(undefined);

export interface TabsProps {
  /**
   * The selected tab value.
   */
  value?: string | number;
  /**
   * Default selected tab for uncontrolled tabs.
   */
  defaultValue?: string | number;
  /**
   * Callback fired when the tab changes.
   */
  onChange?: (value: string | number) => void;
  /**
   * The variant of the tabs.
   * @default 'standard'
   */
  variant?: TabsVariant;
  /**
   * The orientation of the tabs.
   * @default 'horizontal'
   */
  orientation?: TabsOrientation;
  /**
   * The content of the tabs (Tab components and/or TabPanel components).
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Tabs component - a container for tab panels.
 * Follows MUI Tabs API conventions.
 *
 * Tabs organizes content into multiple sections that can be switched between.
 *
 * @example
 * // Basic tabs
 * <Tabs value={value} onChange={(v) => setValue(v)}>
 *   <Tab value="one" label="Tab One" />
 *   <Tab value="two" label="Tab Two" />
 * </Tabs>
 *
 * @example
 * // Full width tabs
 * <Tabs variant="fullWidth" defaultValue="one">
 *   <Tab value="one" label="First" />
 *   <Tab value="two" label="Second" />
 * </Tabs>
 */
export const Tabs = forwardRef<HTMLDivElement, TabsProps>(function Tabs(
  {
    value: valueProp,
    defaultValue,
    onChange,
    variant = 'standard',
    orientation = 'horizontal',
    children,
    className,
    ...props
  }: TabsProps,
  ref: React.Ref<HTMLDivElement>
) {
  const [value, setValue] = useControllableState<string | number>({
    value: valueProp,
    defaultValue,
    onChange,
  });

  const baseStyles = cn(
    'border-b border-nest-border',
    orientation === 'vertical' && 'border-b-0 border-r',
    orientation === 'horizontal' && 'flex-row',
    orientation === 'vertical' && 'flex-col',
    variant === 'fullWidth' && 'w-full',
    className
  );

  return (
    <TabsContext.Provider
      value={{
        value,
        onChange: (newValue) => setValue(newValue),
        variant,
        orientation,
      }}
    >
      <div
        ref={ref as any}
        className={cn('flex', baseStyles)}
        role="tablist"
        {...props}
      >
        {children}
      </div>
    </TabsContext.Provider>
  );
});

export interface TabProps {
  /**
   * The value of the tab.
   */
  value: string | number;
  /**
   * The label to display.
   */
  label?: ReactNode;
  /**
   * The icon to display.
   */
  icon?: ReactNode;
  /**
   * If true, the tab is disabled.
   * @default false
   */
  disabled?: boolean;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * Tab component - a single tab within a Tabs container.
 * Follows MUI Tab API conventions.
 *
 * @example
 * <Tab value="home" label="Home" />
 *
 * @example
 * <Tab value="settings" icon={<SettingsIcon />} label="Settings" />
 */
export const Tab = forwardRef<HTMLButtonElement, TabProps>(function Tab(
  {
    value,
    label,
    icon,
    disabled = false,
    className,
    ...props
  }: TabProps,
  ref: React.Ref<HTMLButtonElement>
) {
  const context = useContext(TabsContext);

  const selected = context?.value === value;
  const orientation = context?.orientation ?? 'horizontal';
  const onChange = context?.onChange;

  const handleClick = () => {
    if (onChange && !disabled) {
      onChange(value);
    }
  };

  const baseStyles = cn(
    'flex items-center justify-center gap-2',
    'px-4 py-2',
    'text-sm font-medium',
    'text-nest-muted',
    'border-b-2 border-transparent',
    'transition-colors duration-200',
    'hover:text-nest-primary hover:border-nest-primary/30',
    'focus:outline-none focus:ring-2 focus:ring-nest-primary/50 focus:ring-offset-2',
    'disabled:opacity-50 disabled:cursor-not-allowed',
    orientation === 'horizontal' && 'min-w-fit',
    orientation === 'vertical' && 'w-full text-left justify-start border-b-0 border-l-2',
    orientation === 'vertical' && 'hover:border-l-nest-primary/30',
    selected && 'text-nest-primary border-nest-primary',
    disabled && 'hover:text-nest-muted hover:border-transparent',
    className
  );

  return (
    <button
      ref={ref}
      type="button"
      role="tab"
      aria-selected={selected}
      disabled={disabled}
      onClick={handleClick}
      className={baseStyles}
      {...props}
    >
      {icon}
      {label}
    </button>
  );
});

export interface TabPanelProps {
  /**
   * The value of the tab this panel corresponds to.
   */
  value: string | number;
  /**
   * The content of the panel.
   */
  children?: ReactNode;
  /**
   * Additional CSS classes.
   */
  className?: string;
}

/**
 * TabPanel component - the content panel for a tab.
 * Follows MUI TabPanel API conventions.
 *
 * @example
 * <TabPanel value="home">
 *   <p>Home content here</p>
 * </TabPanel>
 */
export const TabPanel = forwardRef<HTMLDivElement, TabPanelProps>(function TabPanel(
  {
    value,
    children,
    className,
    ...props
  }: TabPanelProps,
  ref: React.Ref<HTMLDivElement>
) {
  const context = useContext(TabsContext);

  const selected = context?.value === value;

  const baseStyles = cn(
    'p-4',
    !selected && 'hidden',
    className
  );

  return (
    <div
      ref={ref as any}
      role="tabpanel"
      className={baseStyles}
      hidden={!selected}
      {...props}
    >
      {selected && children}
    </div>
  );
});
