import { useCallback, useEffect, useState } from 'react';

/**
 * UseControllableState options.
 */
interface UseControllableStateOptions<T> {
  /**
   * The controlled value. If provided, the state becomes controlled.
   */
  value?: T;
  /**
   * Callback fired when the state changes.
   */
  onChange?: (value: T) => void;
  /**
   * The default value for uncontrolled mode.
   */
  defaultValue?: T;
}

/**
 * A hook that supports both controlled and uncontrolled usage patterns.
 * Follows React's recommended pattern for composable components.
 *
 * @example
 * // Uncontrolled
 * const [value, setValue] = useControllableState({ defaultValue: '' });
 *
 * @example
 * // Controlled
 * const [value, setValue] = useControllableState({ value: props.value, onChange: props.onChange });
 */
export function useControllableState<T>({
  value,
  onChange,
  defaultValue,
}: UseControllableStateOptions<T>): [T, (newValue: T) => void] {
  const [internalValue, setInternalValue] = useState<T>(defaultValue as T);

  const isControlled = value !== undefined;

  const currentValue = isControlled ? value : internalValue;

  const setValue = useCallback(
    (newValue: T) => {
      if (!isControlled) {
        setInternalValue(newValue);
      }
      onChange?.(newValue);
    },
    [isControlled, onChange]
  );

  // Sync internal state when controlled value changes
  useEffect(() => {
    if (isControlled && value !== internalValue) {
      setInternalValue(value as T);
    }
  }, [isControlled, value, internalValue]);

  return [currentValue, setValue];
}
