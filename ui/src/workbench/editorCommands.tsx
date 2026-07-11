import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useRef,
  useState,
  type ReactNode,
} from "react";
import type { editor } from "monaco-editor";

type EditorCommandState = {
  canUndo: boolean;
  canRedo: boolean;
  canCut: boolean;
  canCopy: boolean;
  canPaste: boolean;
};

const DEFAULT_STATE: EditorCommandState = {
  canUndo: false,
  canRedo: false,
  canCut: false,
  canCopy: false,
  canPaste: false,
};

type EditorCommandsContextValue = EditorCommandState & {
  undo: () => void;
  redo: () => void;
  cut: () => void;
  copy: () => void;
  paste: () => void;
  registerEditor: (editorInstance: editor.IStandaloneCodeEditor | null) => () => void;
};

const EditorCommandsContext = createContext<EditorCommandsContextValue | null>(null);

function selectionHasText(editorInstance: editor.IStandaloneCodeEditor): boolean {
  const selection = editorInstance.getSelection();
  if (!selection || selection.isEmpty()) {
    return false;
  }
  const model = editorInstance.getModel();
  if (!model) {
    return false;
  }
  return model.getValueInRange(selection).length > 0;
}

function runAction(editorInstance: editor.IStandaloneCodeEditor, actionId: string) {
  const action = editorInstance.getAction(actionId);
  if (action?.isSupported()) {
    void action.run();
  }
}

export function EditorCommandsProvider({ children }: { children: ReactNode }) {
  const editorRef = useRef<editor.IStandaloneCodeEditor | null>(null);
  const [state, setState] = useState<EditorCommandState>(DEFAULT_STATE);

  const refresh = useCallback(() => {
    const editorInstance = editorRef.current;
    if (!editorInstance) {
      setState(DEFAULT_STATE);
      return;
    }
    const model = editorInstance.getModel();
    const isReadOnly = editorInstance.getRawOptions().readOnly === true;
    const hasSelection = selectionHasText(editorInstance);
    setState({
      canUndo: !isReadOnly && (model?.canUndo() ?? false),
      canRedo: !isReadOnly && (model?.canRedo() ?? false),
      canCut: !isReadOnly && hasSelection,
      canCopy: hasSelection,
      canPaste: !isReadOnly,
    });
  }, []);

  const registerEditor = useCallback(
    (editorInstance: editor.IStandaloneCodeEditor | null) => {
      editorRef.current = editorInstance;
      refresh();
      if (!editorInstance) {
        return () => undefined;
      }
      const subs = [
        editorInstance.onDidChangeCursorSelection(refresh),
        editorInstance.onDidChangeModelContent(refresh),
      ];
      return () => {
        subs.forEach((sub) => sub.dispose());
        if (editorRef.current === editorInstance) {
          editorRef.current = null;
        }
        refresh();
      };
    },
    [refresh],
  );

  const withEditor = useCallback(
    (actionId: string) => {
      const editorInstance = editorRef.current;
      if (!editorInstance) {
        return;
      }
      runAction(editorInstance, actionId);
      refresh();
    },
    [refresh],
  );

  const value = useMemo<EditorCommandsContextValue>(
    () => ({
      ...state,
      undo: () => withEditor("undo"),
      redo: () => withEditor("redo"),
      cut: () => withEditor("editor.action.clipboardCutAction"),
      copy: () => withEditor("editor.action.clipboardCopyAction"),
      paste: () => withEditor("editor.action.clipboardPasteAction"),
      registerEditor,
    }),
    [registerEditor, state, withEditor],
  );

  return (
    <EditorCommandsContext.Provider value={value}>{children}</EditorCommandsContext.Provider>
  );
}

export function useEditorCommands() {
  const ctx = useContext(EditorCommandsContext);
  if (!ctx) {
    throw new Error("useEditorCommands must be used within EditorCommandsProvider");
  }
  return ctx;
}
