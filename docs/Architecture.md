---
modified: 2026-07-10
type: architecture
project: AshGrove Welcome
status: active
last_completed_milestone: v0.6.1 — Inline Pack Install Workflow
active_roadmap_milestone: v0.6.2 — Task Progress and Logging
---

# Architecture

## Current Architecture State

| Area | State |
|---|---|
| Last completed milestone | `v0.6.1 — Inline Pack Install Workflow` |
| Final accepted revision | `v0.6.1.9` |
| Active next milestone | `v0.6.2 — Task Progress and Logging` |
| GUI framework | Slint |
| Language | Rust |
| Package workflow target | Host rpm-ostree / Flatpak workflows |
| Runtime validation | Host-only |
| Build validation | May run in `forge-dev` |

## v0.6.1 Completed Flow

### Install

```text
Development Page
    ↓
PackItemCard for Kate
    ↓
Checkbox selection
    ↓
Install Selected
    ↓
Runtime guard
    ↓
ExecutionBoundary check
    ↓
Background worker command execution
    ↓
Card progress / sidebar Tasks (%)
    ↓
Refresh installed state
    ↓
Installed state or pending reboot state
```

### Uninstall

```text
Installed/removable Kate
    ↓
Red trash action
    ↓
Runtime guard
    ↓
ExecutionBoundary check
    ↓
Background worker command execution
    ↓
Refresh installed state
    ↓
Available/not-installed state
```

## State Model

The UI must distinguish:

| State | UI Behavior |
|---|---|
| Not installed | Checkbox enabled; trash hidden |
| Installing | Checkbox disabled; progress visible |
| Installed/removable | Checkbox disabled; trash visible |
| Installed/non-removable | Checkbox disabled; trash hidden/managed |
| Pending reboot | Checkbox disabled; trash hidden until reboot-active state |
| Failed | Show failure state and allow retry when safe |

## Runtime Environment Rule

Package detection and package actions must not run from `forge-dev`.

Host preflight:

```bash
test -f /run/.containerenv && echo "container - stop" && exit 1 || echo "host - OK"
```

## Logging Architecture

Log file:

```text
~/.local/state/ashgrove-welcome/ashgrove-welcome.log
```

v0.6.2 should strengthen logs for:

- runtime detection
- selected item state
- command planning
- command start
- command completion
- exit status
- refresh result
- final UI state
- recovery guidance

## Safety Rules

1. Preserve explicit user action before package operations.
2. Preserve `ExecutionBoundary.commands_allowed == true` as the execution gate.
3. Do not introduce arbitrary shell execution.
4. Do not enable real Gaming Pack execution.
5. Do not execute package actions inside `forge-dev`.
6. Do not log secrets, tokens, passwords, or full environment dumps.

## Technical Debt

- GUI `main.rs` now carries substantial workflow orchestration.
- v0.6.2 should improve logging/progress without a broad rewrite.
- A later milestone may extract install workflow orchestration into a smaller Rust module.
