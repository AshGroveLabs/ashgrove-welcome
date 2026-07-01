---
created: 2026-06-25
modified: 2026-07-01
type: master-knowledge-base
project: Forge Welcome
parent_project: Forge OS
status: active
current_version: v0.6.0
current_milestone_series: v0.6.x — Production UI/UX Finalization
current_milestone: v0.6.0 — Production UI/UX Foundation
current_sprint: v0.6.0 Sprint 1 — Production UI/UX Foundation
current_task: Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout
last_completed_milestone: v0.5.9 — Installation Workflow Stabilization
next_milestone_preview: v0.6.1 — Inline Pack Install Workflow
---

# Master Knowledge Base

## Project Identity

| Field | Value |
|---|---|
| Project | Forge Welcome |
| Parent Project | Forge OS |
| Project Type | Desktop welcome and setup application |
| GUI Framework | Slint |
| Current Version | v0.6.0 |
| Current Milestone Series | v0.6.x — Production UI/UX Finalization |
| Current Implementation Milestone | v0.6.0 — Production UI/UX Foundation |
| Current Sprint | v0.6.0 Sprint 1 — Production UI/UX Foundation |
| Current Task | Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout |
| Last Completed Milestone | v0.5.9 — Installation Workflow Stabilization |
| Safety State | Dry-run default; real Development Pack execution is confirmed/action-gated and boundary-gated |

---

# Current Project State

`v0.5.9 — Installation Workflow Stabilization` is complete, validated, committed, and pushed.

The active v0.6.x direction has changed from Gaming Pack-first expansion to:

```text
v0.6.x — Production UI/UX Finalization
```

The immediate implementation milestone is:

```text
v0.6.0 — Production UI/UX Foundation
```

---

# Key Decision

Forge Welcome must have a production-ready, inline, Discover-inspired install UI before expanding additional pack pages.

The install page should show installable applications as cards, allow item selection inline, use `Install Selected` as the explicit action, show per-item progress/status, show a global `Tasks (%)` indicator, and write persistent workflow logs.

---

# Consolidated UI/UX Requirement

## Page Pattern

```text
Pack Page
    ↓
ForgeScrollArea
    ↓
PackItemCard list
    ↓
Install Selected
    ↓
Inline progress/status
    ↓
Inline result summary
    ↓
Pack refresh
```

## Required Components

- `ForgeScrollArea.slint`
- `PackItemCard.slint`
- `TaskProgressBar.slint`

## Pack Item Card Content

Each card should support:

- checkbox
- app icon or icon placeholder
- app name
- app description
- source/package metadata
- right-side action/status area

## Item States

- Available
- Selected
- Pending
- Installing
- Installed
- Failed
- Skipped
- Blocked

## Global Progress

A lower-left task indicator should show:

```text
Tasks (0%)
Tasks (50%)
Tasks (100%)
```

---

# Completed v0.5.9 Knowledge

## Architecture Completed

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

## Safety Principle

```text
ExecutionMode is intent.
ExecutionBoundary is permission.
```

## Required Execution Gate

```rust
execution_plan.command_boundary.commands_allowed == true
```

## Important Negative Rule

```text
dry_run == false is not sufficient permission to execute commands.
```

---

# Active Milestone

## v0.6.0 — Production UI/UX Foundation

**Status:** Ready

### Objective

Create the reusable Slint UI foundation for production-ready installable pack pages.

### Current Task

```text
Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout.
```

### Planned Work

- Create `ForgeScrollArea.slint`.
- Create `PackItemCard.slint`.
- Create `TaskProgressBar.slint`.
- Use the new components on `DevelopmentPage.slint`.
- Render Kate as the only validation item.
- Replace `Review Installation` with `Install Selected`.
- Preserve existing safety model.

---

# Safety Rules

- Dry-run remains default.
- Real execution requires explicit user action.
- Real execution requires the execution boundary.
- Arbitrary shell execution remains forbidden.
- Hidden execution remains forbidden.
- Multi-pack real execution remains deferred.
- Gaming Pack real execution is not part of v0.6.0.
- Passwords and secrets must not be logged.

---

# Logging Requirement

Persistent workflow logging should write to:

```text
~/.local/state/forge-welcome/forge-welcome.log
```

Required event categories:

- app start
- runtime detected
- pack opened
- install selected
- command started
- command completed
- exit code captured
- transaction result created
- package status refreshed
- install completed or failed

---

# Current Recommended Development Path

```text
Update documentation for v0.6.x Production UI/UX Finalization
    ↓
Implement ForgeScrollArea
    ↓
Implement PackItemCard
    ↓
Implement TaskProgressBar
    ↓
Update Development page with Kate item card
    ↓
Wire inline selection and progress
    ↓
Add persistent logging
    ↓
Apply pattern to Gaming and future pack pages
```

---

# Learning Focus

## Rust

- UI state modeling.
- Enum-driven item state.
- Result/status conversion.
- Safer data flow from core to Slint UI.

## Slint

- Reusable component design.
- Scrollable dynamic content.
- Progress indicators.
- Component properties and callbacks.

## Software Engineering

- Production UI/UX workflow design.
- State-driven UI.
- Reusable page architecture.
- Safety-preserving refactoring.

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
```
