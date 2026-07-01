---
type: project-milestones
project: Forge Welcome
status: active
created: 2026-06-25
modified: 2026-07-01
current_version: v0.6.0
current_milestone_series: v0.6.x — Production UI/UX Finalization
current_milestone: v0.6.0 — Production UI/UX Foundation
last_completed_milestone: v0.5.9 — Installation Workflow Stabilization
next_milestone_preview: v0.6.1 — Inline Pack Install Workflow
---

# Forge Welcome Milestones

## Current Milestone

| Item | Value |
|---|---|
| Current Version | v0.6.0 |
| Current Milestone Series | v0.6.x — Production UI/UX Finalization |
| Current Implementation Milestone | v0.6.0 — Production UI/UX Foundation |
| Status | Ready |
| Previous Completed Milestone | v0.5.9 — Installation Workflow Stabilization |
| Current Sprint | v0.6.0 Sprint 1 — Production UI/UX Foundation |
| Current Task | Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout |
| Next Task | Add inline selection/progress wiring and logging |
| Next Recommended Prompt | IMPLEMENT PROJECT |

---

# Completed Milestones

## v0.5.9 — Installation Workflow Stabilization

**Status:** ✅ Complete

### Objective

Stabilize the installation workflow before expanding to more package packs.

### Completed

- Reviewed and stabilized the installation workflow architecture.
- Reviewed command result and transaction result flow.
- Reviewed refresh behavior.
- Reviewed dry-run and execution mode separation.
- Confirmed execution boundary and confirmation gates remain mandatory.
- Added `ExecutionWorkflowStatus`.
- Added execution-report helper methods.
- Added command-result helper methods.
- Reduced duplication in command/result mapping.
- Improved workflow status naming and responsibility placement.
- Added or refined regression tests for key safety paths.
- Preserved dry-run behavior.
- Preserved guarded real Development Pack execution.
- Preserved explicit confirmation and `ExecutionBoundary.commands_allowed` gate.
- Preserved command-result classification.
- Preserved transaction-result rendering.
- Preserved Development Pack refresh behavior.
- Avoided arbitrary shell execution.
- Completed formatter, build, lint, tests, and GUI validation successfully.
- Source code was committed and pushed.

### Safety Rule

Real command execution remains gated by:

```rust
execution_plan.command_boundary.commands_allowed == true
```

The following remains insufficient:

```rust
dry_run == false
```

---

# Current / Planned Milestones

## v0.6.x — Production UI/UX Finalization

**Status:** Active

### Objective

Make Forge Welcome's UI/UX production-ready for all installable pack pages before expanding pack functionality.

### Governing Direction

The previous Gaming Pack-first v0.6.0 plan is superseded. Gaming Pack expansion moves behind the shared UI/UX foundation.

---

## v0.6.0 — Production UI/UX Foundation

**Status:** Ready

### Objective

Create the reusable Slint UI foundation for production-ready pack pages.

### Planned Work

- Create `ForgeScrollArea.slint`.
- Create `PackItemCard.slint`.
- Create `TaskProgressBar.slint`.
- Render the Development Pack Kate validation item as a Discover-style item card.
- Make the Development Pack item area scrollable.
- Replace `Review Installation` with `Install Selected`.
- Keep the installation dialog as temporary legacy/deprecated infrastructure only if needed.
- Preserve the existing Development Pack execution path until inline workflow wiring is complete.

### Non-Goals

- Do not enable real Gaming Pack execution.
- Do not implement multi-pack real execution.
- Do not remove safety gates.
- Do not introduce arbitrary shell execution.
- Do not remove `InstallReviewDialog.slint` until the inline install workflow is validated.

### Acceptance Criteria

- `ForgeScrollArea.slint` exists and is reusable.
- `PackItemCard.slint` exists and can represent an installable app.
- `TaskProgressBar.slint` exists and can show global progress.
- Development Pack displays Kate as a card with checkbox, icon placeholder, name, description, metadata, and right-side action/status area.
- Development Pack list area is scrollable.
- Button label is `Install Selected`.
- Kate-only validation scope is preserved.
- Dry-run safety is preserved.
- Real execution still requires explicit user action and execution boundary permission.
- Cargo validation succeeds.
- Manual Slint GUI validation passes.

---

## v0.6.1 — Inline Pack Install Workflow

**Status:** Planned

### Objective

Replace the dialog-centered install flow with inline pack-page selection and install behavior.

### Planned Work

- Add selection state for installable items.
- Execute selected Development Pack items from the inline page.
- Remove sub-dialog use from the main install process.
- Show inline item result states.
- Preserve Kate-only validation scope for first pass.

---

## v0.6.2 — Task Progress and Logging

**Status:** Planned

### Objective

Add visible progress and persistent diagnostic logging for real install workflows.

### Planned Work

- Add lower-left `Tasks (%)` global progress indicator.
- Add staged per-item progress state.
- Add persistent log file at `~/.local/state/forge-welcome/forge-welcome.log`.
- Record workflow start/finish, command start/finish, exit code, duration, and refresh result.
- Avoid logging passwords, tokens, or secrets.

---

## v0.6.3 — Multi-Pack UI Application

**Status:** Planned

### Objective

Apply the production UI pattern to Gaming, Productivity, Cloud & Sync, and Forge Ecosystem pages.

### Planned Work

- Add pack item cards to future pack pages.
- Add preview/dry-run state for non-Development packs.
- Keep real execution disabled for non-Development packs.

---

## v0.6.4 — UI/UX Stabilization and Polish

**Status:** Planned

### Objective

Stabilize spacing, scrolling, empty states, error states, accessibility labels, and visual polish.

---

# Later Milestones

## v0.7.x — Forge OS Integration

Package profile integration, rpm-ostree planning, Flatpak planning, and Distrobox planning remain future work.

## v0.8.x — First-Run Experience and Settings

First-run onboarding, system dashboard, and preferences remain future work.

## v0.9.x — Pre-Release Stabilization

Feature freeze, documentation pass, and release-candidate preparation remain future work.

---

# Milestone Rules

A milestone is complete only when:

- Implementation is complete.
- Project builds successfully.
- Manual validation passes.
- Documentation is updated.
- `Current Sprint.md` is updated.
- `Project Status.md` is updated.
- `Development Journal.md` is updated.
- `CHANGELOG.md` is updated when applicable.
- Git commit is completed.
- GitHub push is completed.

Milestone commit format:

```bash
git commit -m "Milestone <version>: <milestone description>"
```

---

# Prompt Loop

```text
PROJECT STATUS
IMPLEMENT PROJECT
IMPLEMENT FIX
IMPLEMENT INVESTIGATION
CODE REVIEW
MILESTONE COMPLETE
```
