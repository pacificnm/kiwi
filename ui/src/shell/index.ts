/**
 * Shared Nest desktop primitives for Kiwi (no ribbon shell).
 *
 * Kiwi uses {@link WorkbenchShell} instead of {@code AppShell}.
 */

export { ErrorBoundary } from "../components/ErrorBoundary";
export { Icon } from "../components/Icon";
export { ConfirmDialog } from "../components/ConfirmDialog";
export { StatusBar } from "../components/StatusBar";
export { ToastViewport } from "../components/ToastViewport";

export {
  ToastProvider,
  useToast,
  type ToastItem,
  type ToastOptions,
  type ToastVariant,
} from "../context/ToastContext";
export {
  StatusBarProvider,
  useStatusBar,
  type StatusVariant,
} from "../context/StatusBarContext";

export { isTauri, quitApp } from "../lib/tauri";
