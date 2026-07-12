/**
 * @nest/components - Reusable React Component Library for Nest Desktop Applications
 *
 * A comprehensive collection of React components built with Tailwind CSS
 * and Nest design tokens, following MUI API conventions.
 *
 * Themable values (colors, spacing, radius) are supplied by the host at runtime
 * via the nest-design / nest-react-theme pipeline (nest_theme_css → :root vars).
 * This package ships no palette of its own — only the theme-independent motion in
 * runtime.css below.
 *
 * @packageDocumentation
 */

// Theme-independent component motion (keyframes only — no palette). Imported once
// so animations are always delivered regardless of the consumer's Tailwind config.
import './runtime.css';

// All component categories (inputs, feedback, navigation, surface, layout, data-display)
export * from './components';

// Context providers
export * from './context';

// Hooks
export * from './hooks';

// Utilities
export { cn } from './lib/cn';
