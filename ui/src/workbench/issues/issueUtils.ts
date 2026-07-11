import type { CSSProperties } from "react";
import type { GitHubIssueListItem, GitHubLabel } from "../../lib/github";

/** GitHub-style relative timestamp (e.g. "2 days ago"). */
export function formatRelativeTime(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  const seconds = Math.floor((Date.now() - date.getTime()) / 1000);
  if (seconds < 60) {
    return "just now";
  }
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) {
    return `${minutes} minute${minutes === 1 ? "" : "s"} ago`;
  }
  const hours = Math.floor(minutes / 60);
  if (hours < 24) {
    return `${hours} hour${hours === 1 ? "" : "s"} ago`;
  }
  const days = Math.floor(hours / 24);
  if (days < 30) {
    return `${days} day${days === 1 ? "" : "s"} ago`;
  }
  const months = Math.floor(days / 30);
  if (months < 12) {
    return `${months} month${months === 1 ? "" : "s"} ago`;
  }
  const years = Math.floor(months / 12);
  return `${years} year${years === 1 ? "" : "s"} ago`;
}

/** Short date for issue metadata (e.g. "Jul 8, 2026"). */
export function formatIssueDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) {
    return iso;
  }
  return date.toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "numeric",
  });
}

/** Text color that contrasts with a GitHub label hex background. */
export function labelTextColor(hex: string | null | undefined): string {
  if (!hex) {
    return "rgb(var(--nest-foreground))";
  }
  const normalized = hex.replace("#", "");
  if (normalized.length !== 6) {
    return "rgb(var(--nest-foreground))";
  }
  const r = Number.parseInt(normalized.slice(0, 2), 16);
  const g = Number.parseInt(normalized.slice(2, 4), 16);
  const b = Number.parseInt(normalized.slice(4, 6), 16);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  return luminance > 0.6 ? "#1f2328" : "#ffffff";
}

export function labelStyle(label: GitHubLabel): CSSProperties {
  const background = label.color ? `#${label.color}` : "rgb(var(--nest-muted) / 0.2)";
  return {
    backgroundColor: background,
    color: labelTextColor(label.color),
    border: label.color ? `1px solid rgba(0,0,0,0.08)` : "1px solid rgb(var(--nest-border))",
  };
}

export function filterIssues(
  issues: GitHubIssueListItem[],
  query: string,
): GitHubIssueListItem[] {
  const needle = query.trim().toLowerCase();
  if (!needle) {
    return issues;
  }
  return issues.filter((issue) => {
    if (`#${issue.number}`.includes(needle) || String(issue.number).includes(needle)) {
      return true;
    }
    if (issue.title.toLowerCase().includes(needle)) {
      return true;
    }
    if (issue.author?.login.toLowerCase().includes(needle)) {
      return true;
    }
    return issue.labels.some((label) => label.name.toLowerCase().includes(needle));
  });
}
