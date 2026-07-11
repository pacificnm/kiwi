import { createContext, useCallback, useContext, useMemo, useState, type ReactNode } from "react";

export type SearchPanelFocus = "find" | "replace";

type SearchPanelFocusContextValue = {
  focusTarget: SearchPanelFocus | null;
  requestFocus: (target: SearchPanelFocus) => void;
  clearFocus: () => void;
};

const SearchPanelFocusContext = createContext<SearchPanelFocusContextValue | null>(null);

export function SearchPanelFocusProvider({ children }: { children: ReactNode }) {
  const [focusTarget, setFocusTarget] = useState<SearchPanelFocus | null>(null);

  const requestFocus = useCallback((target: SearchPanelFocus) => {
    setFocusTarget(target);
  }, []);

  const clearFocus = useCallback(() => {
    setFocusTarget(null);
  }, []);

  const value = useMemo(
    () => ({ focusTarget, requestFocus, clearFocus }),
    [clearFocus, focusTarget, requestFocus],
  );

  return (
    <SearchPanelFocusContext.Provider value={value}>
      {children}
    </SearchPanelFocusContext.Provider>
  );
}

export function useSearchPanelFocus() {
  const ctx = useContext(SearchPanelFocusContext);
  if (!ctx) {
    throw new Error("useSearchPanelFocus must be used within SearchPanelFocusProvider");
  }
  return ctx;
}
