---
created: 2026-06-25 22:43
modified: 2026-07-01
type: architecture
project: Forge Welcome
parent_project: Forge OS
status: active
current_version: v0.6.0
current_milestone_series: v0.6.x — Production UI/UX Finalization
current_milestone: v0.6.0 — Production UI/UX Foundation
current_sprint: v0.6.0 Sprint 1 — Production UI/UX Foundation
last_completed_milestone: v0.5.9 — Installation Workflow Stabilization
---

# Architecture

## Purpose

This document defines the current architecture of **Forge Welcome**, the desktop welcome and setup application for the Forge OS ecosystem.

Forge Welcome provides onboarding, package pack selection, installable item presentation, inline installation workflows, dry-run safety, transaction result reporting, package status refresh, controlled command execution, and production-ready Slint UI/UX components.

---

# Current Architecture State

| Area | Status |
|---|---|
| Project | Forge Welcome |
| Parent Project | Forge OS |
| Current Version | v0.6.0 |
| Current Milestone Series | v0.6.x — Production UI/UX Finalization |
| Current Implementation Milestone | v0.6.0 — Production UI/UX Foundation |
| Current Sprint | v0.6.0 Sprint 1 — Production UI/UX Foundation |
| Current Implementation State | v0.5.9 installation workflow stabilization complete; v0.6.x UI/UX direction updated |
| Current Task | Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout |
| Next Task | Inline selection/progress/logging wiring |
| Last Completed Milestone | v0.5.9 — Installation Workflow Stabilization |
| Real Installation Status | Implemented for Development Pack only |
| Safety State | Dry-run default; real Development Pack execution explicitly confirmed/action-gated and boundary-gated |

---

# High-Level Architecture

```text
Forge Welcome
│
├── User Interface Layer
│   ├── Slint pages
│   ├── Home page
│   ├── Development page
│   ├── Future pack pages
│   ├── ForgeScrollArea
│   ├── PackItemCard
│   ├── TaskProgressBar
│   ├── Inline install status/result area
│   └── Legacy InstallReviewDialog path during migration
│
├── Application State Layer
│   ├── Active page state
│   ├── Active pack state
│   ├── Installable item state
│   ├── Selected item state
│   ├── Installed status state
│   ├── Per-item progress state
│   ├── Global task progress state
│   ├── Transaction result state
│   └── Execution state
│
├── Pack Workflow Layer
│   ├── PackId
│   ├── Pack metadata
│   ├── Installable item metadata
│   ├── Pack preview
│   ├── Pack dry run
│   ├── Pack refresh
│   ├── Development Pack execution
│   └── Future pack UI routing
│
├── Installation Workflow Layer
│   ├── Inline selection
│   ├── Install Selected action
│   ├── Progress engine
│   ├── Transaction result model
│   ├── Inline result rendering
│   ├── ExecutionMode
│   ├── ExecutionBoundary
│   ├── CommandSpec
│   ├── CommandResult
│   ├── ExecutionReport
│   ├── ExecutionWorkflowStatus
│   ├── Error classification
│   ├── Recovery guidance
│   ├── Workflow logging
│   └── Guarded command execution
│
└── Forge OS Integration Layer
    ├── Package profiles
    ├── System status
    ├── Development Pack package execution
    └── Future rpm-ostree / Flatpak / Distrobox support
```

---

# Completed v0.5.9 Stabilization Architecture

`v0.5.9 — Installation Workflow Stabilization` is complete.

Completed stabilization flow:

```text
Install Preview
    ↓
Dry Run
    ↓
Dry-Run Result
    ↓
Explicit Final Confirmation
    ↓
ExecutionBoundary Check
    ↓
Controlled Command Invocation
    ↓
CommandResult
    ↓
Error Classification
    ↓
TransactionResult
    ↓
Rendered Slint Output
    ↓
Pack Refresh
    ↓
Regression Tests
```

This remains the safety reference model while v0.6.x moves from dialog-centered presentation to inline page-centered presentation.

---

# v0.6.x Production UI/UX Architecture

## Objective

Replace the current dialog-centered install UX with a reusable inline install-page architecture.

## Target Flow

```text
Pack Page
    ↓
ForgeScrollArea
    ↓
PackItemCard list
    ↓
User selects items
    ↓
Install Selected
    ↓
ExecutionBoundary check
    ↓
Controlled command invocation
    ↓
Per-item progress/status
    ↓
Global Tasks (%) progress
    ↓
Inline result summary
    ↓
Pack refresh
    ↓
Persistent workflow log
```

---

# Reusable Slint Components

## ForgeScrollArea.slint

Purpose:

- Provide reusable Forge-themed scroll behavior.
- Support long installable item lists.
- Support dynamic content sizing.
- Replace ad hoc `Flickable` usage where practical.

## PackItemCard.slint

Purpose:

- Display one installable application or setup item.
- Show checkbox, icon, name, description, source/package metadata, and right-side action/status.
- Support installed, available, selected, installing, failed, skipped, and blocked states.

## TaskProgressBar.slint

Purpose:

- Display global `Tasks (%)` progress.
- Show progress for the active install workflow.
- Support hidden/idle, running, success, and failed states.

---

# Recommended Data Model

```rust
InstallableItem {
    id,
    name,
    description,
    icon_name,
    source,
    package_id,
    install_kind,
    installed,
    selected,
    state,
    progress_percent,
}
```

```rust
InstallItemState {
    Available,
    Selected,
    Pending,
    Installing,
    Installed,
    Failed,
    Skipped,
    Blocked,
}
```

```rust
TaskProgress {
    completed,
    total,
    percent,
    current_item_name,
    message,
}
```

---

# Inline Install Rules

- The visible pack page item list is the review surface.
- `Install Selected` is the explicit user action.
- No installation sub-dialog should be required in the final production install UX.
- During migration, `InstallReviewDialog.slint` may remain as a temporary legacy path.
- Real execution remains Development Pack only until later milestones.
- Real Gaming Pack execution remains disabled.
- The red trash/remove action appears only where semantically valid.
- Fine-grained package manager progress should not be displayed unless the app parses real output.
- Staged progress is acceptable: pending, starting, installing, refreshing, complete, failed.

---

# Logging Architecture

Forge Welcome should write workflow logs to:

```text
~/.local/state/forge-welcome/forge-welcome.log
```

Log events:

- application started
- runtime detected
- pack opened
- selected items changed
- install started
- command started
- command completed
- exit code captured
- transaction result created
- package status refreshed
- install completed or failed

Do not log:

- passwords
- tokens
- full environment dumps
- unredacted secrets

---

# Architecture Rules

1. Preserve dry-run safety.
2. Preserve explicit user action before real execution.
3. Preserve `ExecutionBoundary.commands_allowed == true` as the execution gate.
4. Do not execute arbitrary shell strings.
5. Limit real execution to Development Pack until later milestones.
6. Keep real Gaming Pack execution disabled during v0.6.x UI foundation work.
7. Keep failure classification data separate from Slint rendering.
8. Convert command/error outcomes into transaction-result output.
9. Keep user-facing messages clear and practical.
10. Prefer reusable Slint components over page-specific duplication.
11. Use Slint terminology for UI documentation.
12. Commit and push after completed milestones using milestone commit format.

---

# Architecture Status

| Area | Status |
|---|---|
| Pack-aware workflow | Implemented |
| Installation progress engine | Implemented |
| Transaction result model | Implemented |
| Generic refresh engine | Implemented |
| Execution mode model | Complete |
| Execution boundary model | Complete |
| Command result model | Complete |
| Guarded Development Pack execution | Complete |
| Error classification model | Complete |
| Installation workflow stabilization | Complete |
| ForgeScrollArea | Current task |
| PackItemCard | Current task |
| TaskProgressBar | Current task |
| Inline install workflow | Planned next |
| Persistent workflow logging | Planned next |
| Gaming Pack UI expansion | Deferred until shared UI foundation is in place |
| Multi-pack real execution | Future |
| Forge OS integration | Partial / Future |

---

# Recommended Next Prompt

```text
IMPLEMENT PROJECT

Project:
Forge Welcome

Current milestone:
v0.6.x — Production UI/UX Finalization

Current sprint:
v0.6.0 Sprint 1 — Production UI/UX Foundation

Current task:
Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout.

Preserve Kate-only validation scope.
Preserve Development Pack behavior.
Preserve dry-run safety.
Preserve explicit user action before real execution.
Preserve ExecutionBoundary.commands_allowed as the execution gate.
Do not enable real Gaming Pack execution.
Do not introduce arbitrary shell execution.
Forge Welcome uses Slint for the GUI. Use .slint terminology, not QML.
```
