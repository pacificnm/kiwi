import type { Terminal } from "@xterm/xterm";
import type { FitAddon } from "@xterm/addon-fit";

/** Reads a CSS custom property from :root (theme-driven xterm colors). */
export function cssVar(name: string, fallback: string): string {
  if (typeof window === "undefined") {
    return fallback;
  }
  const value = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
  return value || fallback;
}

/** xterm theme derived from the active Nest theme's CSS variables. */
export function xtermTheme() {
  return {
    background: cssVar("--nest-color-background", "#1b1f23"),
    foreground: cssVar("--nest-color-foreground", "#cccccc"),
    cursor: cssVar("--nest-color-accent", "#4f8ef7"),
  };
}

/**
 * Fits the terminal only when it is measurable AND the size actually changed.
 *
 * Two failure modes this guards against:
 *  1. xterm's `FitAddon.fit()` dereferences `renderer.dimensions`, which is
 *     undefined while the container has zero size (panel mid-layout) — calling
 *     it then throws `undefined is not an object (… _renderer.value.dimensions)`.
 *  2. Calling `fit()` unconditionally from a `ResizeObserver` creates a feedback
 *     loop: `fit()` resizes the terminal → the observer fires → `fit()` again →
 *     … pegging the CPU with no error. Comparing `proposeDimensions()` to the
 *     current grid means we only resize on a real change, so the loop settles.
 *
 * Returns true when a resize was applied.
 */
export function safeFit(
  term: Terminal | null,
  host: HTMLElement | null,
  fit: FitAddon | null,
): boolean {
  if (!term || !host || !fit) {
    return false;
  }
  if (host.clientWidth === 0 || host.clientHeight === 0 || host.offsetParent === null) {
    return false;
  }
  try {
    const dims = fit.proposeDimensions();
    if (
      !dims ||
      !Number.isFinite(dims.cols) ||
      !Number.isFinite(dims.rows) ||
      dims.cols < 1 ||
      dims.rows < 1
    ) {
      return false;
    }
    if (dims.cols === term.cols && dims.rows === term.rows) {
      return false;
    }
    fit.fit();
    return true;
  } catch {
    return false;
  }
}
