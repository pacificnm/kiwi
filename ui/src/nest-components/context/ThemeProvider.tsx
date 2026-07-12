import { createContext, useContext, useEffect, useState, type ReactNode } from 'react';

export type ThemeMode = 'light' | 'dark';

interface ThemeContextValue {
  mode: ThemeMode;
  toggleMode: () => void;
  setMode: (mode: ThemeMode) => void;
}

const ThemeContext = createContext<ThemeContextValue | undefined>(undefined);

export interface ThemeProviderProps {
  children: ReactNode;
  defaultMode?: ThemeMode;
  storageKey?: string;
}

/**
 * Tracks a light/dark UI-mode hint and mirrors it onto the root `data-theme`
 * attribute.
 *
 * NOTE: This provider does **not** define any colors. Theming (the actual
 * `--nest-color-*` / `--nest-spacing-*` / `--nest-radius-*` values) is host-driven:
 * the Tauri host injects the active theme's `:root` block at runtime via the
 * `nest_theme_css` command (nest-design → nest-react-theme pipeline), and it
 * supports all built-in themes, not just light/dark. `data-theme` here is only a
 * hint for components that branch on light/dark; it carries no palette.
 *
 * @example
 * <ThemeProvider defaultMode="light" storageKey="nest-theme">
 *   <App />
 * </ThemeProvider>
 */
export function ThemeProvider({
  children,
  defaultMode = 'light',
  storageKey = 'nest-theme',
}: ThemeProviderProps) {
  const [mode, setModeState] = useState<ThemeMode>(() => {
    if (typeof window !== 'undefined' && storageKey) {
      const stored = localStorage.getItem(storageKey);
      if (stored === 'light' || stored === 'dark') {
        return stored;
      }
    }
    return defaultMode;
  });

  useEffect(() => {
    const root = document.documentElement;
    root.setAttribute('data-theme', mode);
    if (storageKey) {
      localStorage.setItem(storageKey, mode);
    }
  }, [mode, storageKey]);

  const setMode = (newMode: ThemeMode) => {
    setModeState(newMode);
  };

  const toggleMode = () => {
    setModeState((prev) => (prev === 'light' ? 'dark' : 'light'));
  };

  return (
    <ThemeContext.Provider value={{ mode, toggleMode, setMode }}>
      {children}
    </ThemeContext.Provider>
  );
}

/**
 * Hook to access theme context. Must be used within a ThemeProvider.
 *
 * @throws Error if used outside ThemeProvider
 */
export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}
