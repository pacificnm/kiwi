import { useRef, useState, useCallback, useEffect } from 'react';
import { cn } from '../../../lib/cn';
import { useFilter } from '@react-aria/i18n';
import { useFocusRing } from '@react-aria/focus';
import { TextField } from '../TextField';

export interface AutocompleteOption {
  value: string;
  label: string;
  description?: string;
  disabled?: boolean;
}

export interface AutocompleteProps {
  /**
   * The options to display in the dropdown.
   */
  options: AutocompleteOption[];
  /**
   * The selected value.
   */
  value?: string;
  /**
   * The default value (uncontrolled).
   */
  defaultValue?: string;
  /**
   * Callback fired when the value changes.
   */
  onChange?: (value: string) => void;
  /**
   * Callback fired when the input value changes (for filtering).
   */
  onInputChange?: (value: string) => void;
  /**
   * Placeholder text for the input.
   */
  placeholder?: string;
  /**
   * Label for the input.
   */
  label?: string;
  /**
   * Helper text to display.
   */
  helperText?: string;
  /**
   * If true, the component is disabled.
   */
  disabled?: boolean;
  /**
   * If true, the field is required.
   */
  required?: boolean;
  /**
   * If true, the field has an error.
   */
  error?: boolean;
  /**
   * Error message to display.
   */
  errorMessage?: string;
  /**
   * If true, allow free-form text entry (not just selection).
   * @default false
   */
  freeSolo?: boolean;
  /**
   * If true, multiple values can be selected.
   * @default false
   */
  multiple?: boolean;
  /**
   * Custom class name for the popover.
   */
  popoverClassName?: string;
  /**
   * Number of items to display in the dropdown.
   * @default 8
   */
  maxItems?: number;
  /**
   * Custom renderer for options.
   */
  renderOption?: (option: AutocompleteOption) => React.ReactNode;
  /**
   * Loading state.
   */
  loading?: boolean;
  /**
   * No options message.
   */
  noOptionsText?: string;
}

/**
 * Autocomplete component - a text input with dropdown suggestions.
 * Follows MUI Autocomplete API conventions with react-aria for accessibility.
 *
 * @example
 * // Basic autocomplete
 * <Autocomplete
 *   options={[
 *     { value: 'react', label: 'React' },
 *     { value: 'vue', label: 'Vue' },
 *     { value: 'angular', label: 'Angular' },
 *   ]}
 *   onChange={(value) => console.log(value)}
 * />
 */
export function Autocomplete({
  options,
  value,
  onChange,
  onInputChange,
  placeholder,
  label,
  helperText,
  disabled = false,
  required = false,
  error = false,
  errorMessage,
  popoverClassName,
  maxItems = 8,
  renderOption,
  loading = false,
  noOptionsText = 'No options',
}: AutocompleteProps) {
  const [isOpen, setIsOpen] = useState(false);
  const [inputValue, setInputValue] = useState('');
  const popoverRef = useRef<HTMLDivElement>(null);

  const { contains } = useFilter({ sensitivity: 'base' });

  // Filter options based on input value
  const filteredOptions = options.filter(
    (option) =>
      !inputValue ||
      contains(option.label, inputValue) ||
      contains(option.value, inputValue)
  );

  // Handle option selection
  const handleSelect = useCallback(
    (optionValue: string) => {
      onChange?.(optionValue);
      setInputValue('');
      setIsOpen(false);
    },
    [onChange]
  );

  // Handle input change
  const handleInputChange = useCallback(
    (value: string) => {
      setInputValue(value);
      onInputChange?.(value);
      if (value && !isOpen) {
        setIsOpen(true);
      }
    },
    [onInputChange, isOpen]
  );

  // Handle keyboard navigation
  const handleKeyDown = useCallback(
    (event: React.KeyboardEvent) => {
      if (event.key === 'ArrowDown' && !isOpen) {
        setIsOpen(true);
      } else if (event.key === 'Escape' && isOpen) {
        setIsOpen(false);
      }
    },
    [isOpen]
  );

  // Close when clicking outside
  useEffect(() => {
    const handleClickOutside = (event: MouseEvent) => {
      if (popoverRef.current && !popoverRef.current.contains(event.target as Node)) {
        setIsOpen(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, []);

  const selectedOption = options.find((opt) => opt.value === value);
  const displayValue = selectedOption?.label || inputValue || '';

  return (
    <div className="relative w-full">
      <TextField
        value={displayValue}
        onChange={(e) => handleInputChange(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        label={label}
        helperText={helperText}
        disabled={disabled}
        required={required}
        error={error ? errorMessage : undefined}
        className="w-full"
        role="combobox"
        aria-expanded={isOpen}
        aria-controls="autocomplete-listbox"
        aria-autocomplete="list"
        autoComplete="off"
      />

      {/* Dropdown button */}
      <button
        type="button"
        tabIndex={-1}
        disabled={disabled}
        onClick={() => setIsOpen(!isOpen)}
        className={cn(
          'absolute right-2 top-1/2 -translate-y-1/2',
          'p-1 rounded',
          'hover:bg-nest-surface',
          'disabled:opacity-50',
          'text-nest-muted'
        )}
        aria-label="Toggle dropdown"
      >
        <svg
          className={cn('w-4 h-4 transition-transform', isOpen && 'rotate-180')}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </button>

      {/* Dropdown popover */}
      {isOpen && filteredOptions.length > 0 && (
        <div
          ref={popoverRef}
          className={cn(
            'absolute z-50 mt-1 w-full',
            'bg-nest-surface',
            'border border-nest-border',
            'rounded-nest-md',
            'shadow-lg',
            'max-h-64 overflow-auto',
            popoverClassName
          )}
          style={{ top: '100%' }}
        >
          <ListBox
            options={filteredOptions.slice(0, maxItems)}
            selectedValue={value}
            onSelect={handleSelect}
            renderOption={renderOption}
          />
        </div>
      )}

      {/* No options message */}
      {isOpen && filteredOptions.length === 0 && !loading && (
        <div
          className={cn(
            'absolute z-50 mt-1 w-full',
            'bg-nest-surface',
            'border border-nest-border',
            'rounded-nest-md',
            'shadow-lg',
            'p-3 text-nest-muted',
            popoverClassName
          )}
          style={{ top: '100%' }}
        >
          {noOptionsText}
        </div>
      )}

      {/* Loading state */}
      {loading && (
        <div
          className={cn(
            'absolute z-50 mt-1 w-full',
            'bg-nest-surface',
            'border border-nest-border',
            'rounded-nest-md',
            'shadow-lg',
            'p-3 text-nest-muted text-center',
            popoverClassName
          )}
          style={{ top: '100%' }}
        >
          Loading...
        </div>
      )}
    </div>
  );
}

interface ListBoxProps {
  options: AutocompleteOption[];
  selectedValue?: string;
  onSelect: (value: string) => void;
  renderOption?: (option: AutocompleteOption) => React.ReactNode;
}

function ListBox({ options, selectedValue, onSelect, renderOption }: ListBoxProps) {
  return (
    <ul id="autocomplete-listbox" role="listbox" className="py-1">
      {options.map((option) => (
        <ListBoxOption
          key={option.value}
          option={option}
          isSelected={option.value === selectedValue}
          onSelect={() => onSelect(option.value)}
          renderOption={renderOption}
        />
      ))}
    </ul>
  );
}

interface ListBoxOptionProps {
  option: AutocompleteOption;
  isSelected: boolean;
  onSelect: () => void;
  renderOption?: (option: AutocompleteOption) => React.ReactNode;
}

function ListBoxOption({ option, isSelected, onSelect, renderOption }: ListBoxOptionProps) {
  const ref = useRef<HTMLLIElement>(null);

  const { focusProps, isFocusVisible } = useFocusRing();

  return (
    <li
      {...focusProps}
      ref={ref}
      onClick={() => !option.disabled && onSelect()}
      className={cn(
        'px-3 py-2 cursor-pointer',
        'text-sm',
        'transition-colors',
        isSelected && 'bg-nest-primary/10 text-nest-primary',
        !isSelected && 'hover:bg-nest-surface',
        option.disabled && 'opacity-50 cursor-not-allowed',
        isFocusVisible && 'outline outline-2 outline-nest-primary outline-offset-[-2px]'
      )}
      role="option"
      aria-selected={isSelected}
      aria-disabled={option.disabled}
    >
      {renderOption ? (
        renderOption(option)
      ) : (
        <>
          <div className="font-medium">{option.label}</div>
          {option.description && (
            <div className="text-xs text-nest-muted mt-0.5">{option.description}</div>
          )}
        </>
      )}
    </li>
  );
}
