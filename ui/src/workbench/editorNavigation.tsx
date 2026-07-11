import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from "react";

export type EditorLocation = {
  relPath: string;
  line: number;
  col: number;
  token: number;
};

type EditorNavigationContextValue = {
  location: EditorLocation | null;
  openAt: (relPath: string, line: number, col: number) => void;
  clear: () => void;
};

const EditorNavigationContext = createContext<EditorNavigationContextValue | null>(null);

export function EditorNavigationProvider({ children }: { children: ReactNode }) {
  const [location, setLocation] = useState<EditorLocation | null>(null);

  const openAt = useCallback((relPath: string, line: number, col: number) => {
    setLocation({ relPath, line, col, token: Date.now() });
  }, []);

  const clear = useCallback(() => {
    setLocation(null);
  }, []);

  const value = useMemo(
    () => ({ location, openAt, clear }),
    [clear, location, openAt],
  );

  return (
    <EditorNavigationContext.Provider value={value}>
      {children}
    </EditorNavigationContext.Provider>
  );
}

export function useEditorNavigation() {
  const ctx = useContext(EditorNavigationContext);
  if (!ctx) {
    throw new Error("useEditorNavigation must be used within EditorNavigationProvider");
  }
  return ctx;
}
