# Kiwi

## Terminal-Native AI Development Workspace

Version: 0.1 Draft
Status: Project Initiation Document

---

# Executive Summary

Kiwi is a Rust-based terminal application designed to orchestrate modern software development workflows from a single terminal-native interface.

Kiwi is not a text editor.

Kiwi is a development workbench that integrates:

* AI Agents
* Shell Sessions
* Git
* GitHub Issues
* GitHub Pull Requests
* File Navigation
* Search
* Diff Review
* External Editors

The objective is to provide a Cursor-like development experience entirely within the terminal while remaining editor-agnostic.

---

# Product Vision

## Mission

Provide a complete software development workspace without requiring a graphical IDE.

## Philosophy

Orchestrator First.

Editor Second.

Kiwi coordinates tools rather than replacing them.

Supported editors include:

* Vim
* Neovim
* Helix
* Nano
* Micro
* VS Code
* Cursor
* Zed

---

# Primary User Workflows

## Issue Driven Development

1. Open GitHub Issue
2. Create branch
3. Launch AI Agent
4. Implement changes
5. Review diff
6. Commit changes
7. Create Pull Request
8. Merge

Entire workflow remains inside Kiwi.

---

## AI Driven Development

1. Open repository
2. Select issue
3. Open Agent tab
4. Give instructions
5. Review generated changes
6. View diff
7. Edit files if necessary
8. Commit and push

---

## Traditional Development

1. Browse files
2. Open editor
3. Run commands
4. View Git status
5. Create PR

---

# Layout

## Workspace Layout

```text
┌────────────────────────────┬─────────────────────────────────────────────┐
│ Files Git GH Search        │ Agent Issues PRs Diff Preview Logs         │
├────────────────────────────┼─────────────────────────────────────────────┤
│                            │                                             │
│ Left Navigation            │ Main Workspace                             │
│                            │                                             │
│ File Tree                  │ Active Tab                                │
│ Git Changes                │                                             │
│ Search                     │                                             │
│ GitHub Navigation          │                                             │
│                            │                                             │
├────────────────────────────┼─────────────────────────────────────────────┤
│ Command Palette            │ Shell                                      │
└────────────────────────────┴─────────────────────────────────────────────┘

Status Bar
```

---

# Navigation Model

## Left Navigation Tabs

### Files

Repository file browser.

### Git

Git status view (changed files; `Enter` opens main Diff tab).

### GH

GitHub issue list (navigate here; detail in main Issues tab).

### Search

Repository search.

---

## Main Workspace Tabs

### Agent

AI Agent session.

### Issues

GitHub issue detail (select from GH left tab).

### PRs

GitHub Pull Requests.

### Diff

Detailed diff viewer.

### Preview

Read-only file preview.

### Logs

Application logs.

---

# Mouse Support

## Goals

Provide lightweight mouse interaction.

### Supported

* Select tabs
* Select files
* Select issues
* Select pull requests
* Focus panes
* Scroll content
* Paste content

### Text Selection

Terminal-native selection must continue working.

Support:

* Shift + Drag
* Terminal clipboard integration

### Configuration

```toml
[mouse]
enabled = true
mode = "hybrid"
```

---

# Theme System

## Goals

Provide modern visual experience.

Support:

* Dark themes
* Light themes
* User themes

---

## Built-In Themes

### Kiwi Dark

Default theme.

### Kiwi Light

Light mode.

### Dracula

Popular dark theme.

### Catppuccin Mocha

Popular dark theme.

### Catppuccin Latte

Popular light theme.

### Gruvbox

Popular theme.

### Nord

Popular theme.

### Tokyo Night

Popular theme.

---

## Theme Configuration

```toml
[theme]
name = "kiwi-dark"
```

Custom:

```toml
[theme]
custom = "~/.config/kiwi/themes/custom.toml"
```

---

# Color Guidelines

## Git

Added:
Green

Modified:
Yellow

Deleted:
Red

Untracked:
Blue

---

## Issues

Open:
Cyan

In Progress:
Yellow

Closed:
Gray

---

## Pull Requests

Open:
Blue

Draft:
Yellow

Merged:
Green

Closed:
Gray

---

## Agent

Thinking:
Purple

Executing:
Blue

Success:
Green

Error:
Red

Warning:
Yellow

---

# Configuration System

## User Configuration

```text
~/.config/kiwi/config.toml
```

## Project Configuration

```text
.kiwi.toml
```

---

## Resolution Order

1. CLI Arguments
2. Project Configuration
3. User Configuration
4. Defaults

---

## Example Configuration

```toml
[app]
left_width = 30

[theme]
name = "kiwi-dark"

[editor]
command = "nvim"

[agent]
command = "agent"

[shell]
command = "bash"

[mouse]
enabled = true

[git]
watch = true
show_untracked = true

[github]
command = "gh"
```

---

# File Browser Requirements

## Features

* Tree view
* Expand folders
* Collapse folders
* Lazy loading
* Search
* Preview
* Open editor

---

## Ignore Defaults

```text
.git
node_modules
target
dist
build
.next
.nuxt
.venv
```

---

# External Editor Requirements

## Supported Editors

* Vim
* Neovim
* Helix
* Nano
* Micro
* VS Code
* Cursor
* Zed

---

## Resolution Order

1. Configured editor
2. VISUAL
3. EDITOR
4. Nano

---

# Git Requirements

## Display

* Branch
* Ahead/Behind
* Modified files
* Added files
* Deleted files
* Untracked files

---

## Refresh Strategy

No polling.

Use:

* File watcher
* Debounced updates
* Incremental updates

Must preserve:

* Scroll position
* Selection
* Focus state

---

# GitHub Requirements

## Issues

Support:

* List
* View
* Search
* Assign
* Comment
* Label
* Create branch

---

## Pull Requests

Support:

* List
* View
* Create
* Review
* Open browser

---

## Initial Integration

Use:

```bash
gh
```

Future evaluation:

GitHub GraphQL API.

---

# Shell Requirements

Embedded PTY.

Support:

* Bash
* Zsh
* Fish
* User shell

Must support:

* Long-running commands
* Interactive commands
* Bracketed paste

---

# Agent Requirements

Embedded PTY.

Initial support:

* Cursor Agent

Future:

* Multiple agents
* Agent orchestration
* Agent history

---

# Search Requirements

Support:

* File search
* Content search
* Symbol search

Future:

* Ripgrep integration
* Tree-sitter integration

---

# Status Bar

Display:

```text
Kiwi | Repository | Branch | Agent State | Git State | Issue
```

Example:

```text
Kiwi | cityartwalks | feature/42 | Agent Running | 3 Modified
```

---

# Technology Stack

## Language

Rust

---

## Core Crates

```toml
ratatui
crossterm
tokio
portable-pty
notify
serde
toml
git2
anyhow
```

---

## Future Crates

```toml
octocrab
tree-sitter
ignore
walkdir
```

---

# Required ADRs

ADR-001 Workspace Philosophy

ADR-002 TUI Framework Selection

ADR-003 Layout Architecture

ADR-004 Theme System

ADR-005 Configuration System

ADR-006 PTY Architecture

ADR-007 State Management

ADR-008 File Tree Architecture

ADR-009 Search Architecture

ADR-010 Git Integration

ADR-011 File Watcher Architecture

ADR-012 GitHub Integration

ADR-013 External Editor Strategy

ADR-014 Command Palette Architecture

ADR-015 Mouse Interaction

ADR-016 Workspace Persistence

ADR-017 Multi-Agent Future Design

ADR-018 Plugin Architecture

---

# Required Specifications

SPEC-001 Startup Lifecycle

SPEC-002 Layout Engine

SPEC-003 Theme Engine

SPEC-004 Navigation System

SPEC-005 File Explorer

SPEC-006 File Preview

SPEC-007 Search System

SPEC-008 Git Service

SPEC-009 GitHub Service

SPEC-010 Agent Service

SPEC-011 Shell Service

SPEC-012 Diff Viewer

SPEC-013 Command Palette

SPEC-014 Mouse Support

SPEC-015 Editor Launcher

SPEC-016 State Management

SPEC-017 Workspace Persistence

SPEC-018 Configuration Loader

SPEC-019 Status Bar

SPEC-020 Plugin Framework

---

# Milestones

## Milestone 1

Foundation

* Project scaffold
* Config loader
* Theme system
* Layout rendering
* Keyboard navigation

## Milestone 2

Terminal Services

* Shell PTY
* Agent PTY
* Command palette

## Milestone 3

File Management

* File tree
* Preview
* Editor launcher
* Search

## Milestone 4

Git Integration

* Git status
* File watcher
* Diff viewer

## Milestone 5

GitHub Integration

* Issues
* Pull Requests
* Branch workflows

## Milestone 6

Workspace Features

* Persistence
* Saved sessions
* Theme packs

## Milestone 7

Advanced Features

* Multi-agent support
* Plugins
* Performance optimization

---

# Success Criteria

A developer can:

1. Open a repository.
2. Browse files.
3. View issues.
4. Launch an AI agent.
5. Edit files.
6. Review diffs.
7. Create a pull request.

Without leaving Kiwi.

