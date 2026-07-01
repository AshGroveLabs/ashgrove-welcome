---
created: 2026-06-25
modified: 2026-07-01
type: current-sprint
project: Forge Welcome
parent_project: Forge OS
status: active
current_version: v0.6.0
current_milestone_series: v0.6.x — Production UI/UX Finalization
current_milestone: v0.6.0 — Production UI/UX Foundation
current_sprint: v0.6.0 Sprint 1 — Production UI/UX Foundation
last_completed_milestone: v0.5.9 — Installation Workflow Stabilization
---

# Current Sprint

## Sprint Dashboard

| Field | Value |
|---|---|
| Project | Forge Welcome |
| Parent Project | Forge OS |
| Current Version | v0.6.0 |
| Current Milestone Series | v0.6.x — Production UI/UX Finalization |
| Current Milestone | v0.6.0 — Production UI/UX Foundation |
| Sprint | v0.6.0 Sprint 1 — Production UI/UX Foundation |
| Status | Ready |
| Current Task | Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout |
| Next Task | Wire inline item selection, progress, and logging |
| Blockers | None known |
| Build Status | Passing after v0.5.9 validation; needs revalidation after implementation |
| Test Status | Passing after v0.5.9 validation; UI state tests needed |
| Review Status | v0.6.0 production UI/UX direction documented; implementation not yet reviewed |
| Last Completed Milestone | v0.5.9 — Installation Workflow Stabilization |
| Last Updated | 2026-07-01 |

---

## Sprint Goal

Implement the reusable Slint UI foundation for production-ready Forge Welcome install pages.

The sprint replaces the Gaming Pack-first start of v0.6.0 with a shared production UI/UX foundation. Gaming Pack expansion resumes after the shared item-card, scroll-area, progress, and inline install workflow patterns are in place.

---

## Milestone Context

`v0.5.9 — Installation Workflow Stabilization` is complete and validated.

Completed capabilities available for v0.6.0:

- Pack-aware installation workflow foundation.
- Installation progress state.
- Transaction result model.
- Pack refresh engine.
- Execution mode and execution boundary model.
- Guarded real execution for Development Pack only.
- Installation error classification and recovery guidance.
- Workflow status interpretation through core helpers.
- Stabilized command/result mapping.
- Dry-run safety.
- Explicit confirmation and boundary-gated real execution.

Standing rule:

```text
ExecutionMode is intent.
ExecutionBoundary is permission.
```

No command may execute unless:

```rust
execution_plan.command_boundary.commands_allowed == true
```

---

## Sprint Tasks

- [ ] Create `crates/forge-welcome-gui/ui/components/ForgeScrollArea.slint`.
- [ ] Create `crates/forge-welcome-gui/ui/components/PackItemCard.slint`.
- [ ] Create `crates/forge-welcome-gui/ui/components/TaskProgressBar.slint`.
- [ ] Update `DevelopmentPage.slint` to use a scrollable item list.
- [ ] Render Kate as the only Development Pack validation item.
- [ ] Show checkbox, icon placeholder, name, description, source/package metadata, and right-side action/status for Kate.
- [ ] Replace `Review Installation` label with `Install Selected`.
- [ ] Preserve current Development Pack execution behavior while the inline workflow is being introduced.
- [ ] Keep `InstallReviewDialog.slint` as temporary legacy/deprecated path only if needed.
- [ ] Prepare state names for item status and task progress.
- [ ] Preserve dry-run safety.
- [ ] Preserve explicit user action before real execution.
- [ ] Preserve `ExecutionBoundary.commands_allowed` as the execution gate.
- [ ] Ensure real Gaming Pack execution remains disabled.
- [ ] Run formatter, build, lint, tests, and manual Slint GUI validation.

---

## Acceptance Criteria

- [ ] `ForgeScrollArea.slint` exists and can contain variable-height content.
- [ ] `PackItemCard.slint` exists and displays installable item content.
- [ ] `TaskProgressBar.slint` exists and can show `Tasks (%)` state.
- [ ] Development page uses a scrollable item section.
- [ ] Kate appears as a production-style card.
- [ ] Kate remains the only Development Pack validation item.
- [ ] Main install button reads `Install Selected`.
- [ ] No install sub-dialog is introduced for the target production workflow.
- [ ] Existing safety behavior remains intact.
- [ ] Real Gaming Pack execution remains disabled.
- [ ] No arbitrary shell execution is introduced.
- [ ] `cargo fmt --all` succeeds.
- [ ] `cargo check` succeeds.
- [ ] `cargo clippy` succeeds.
- [ ] `cargo test` succeeds.
- [ ] Manual Slint GUI validation passes.

---

## Safety Constraints

Allowed in v0.6.0 Sprint 1:

- Reusable `ForgeScrollArea` component.
- Reusable `PackItemCard` component.
- Reusable `TaskProgressBar` component.
- Scrollable Development Pack item list.
- Kate-only production-style item card.
- Button label update to `Install Selected`.
- UI state preparation for inline install workflow.

Not allowed in v0.6.0 Sprint 1:

- Real execution for Gaming Pack.
- Arbitrary shell command execution.
- Hidden command execution.
- Execution without explicit user action.
- Execution without `ExecutionBoundary.commands_allowed == true`.
- Removal of safety gates.
- Multi-pack real execution.
- Direct password handling in Forge Welcome.

---

## Validation Checklist

Run from the Forge Welcome repository root:

```bash
cargo fmt --all
cargo check
cargo clippy
cargo test
cargo run -p forge-welcome-gui
```

Manual validation:

- [ ] Forge Welcome launches.
- [ ] Development page loads.
- [ ] Development page displays a scrollable item-card area.
- [ ] Kate is the only visible Development Pack validation item.
- [ ] Kate card displays checkbox, icon placeholder, name, description, metadata, and right-side action/status area.
- [ ] Button reads `Install Selected`.
- [ ] Existing Development Pack install safety path still works or remains preserved behind the temporary legacy path.
- [ ] Dry-run performs no system modification.
- [ ] Real execution remains confirmation/action-gated and boundary-gated.
- [ ] Real Gaming Pack execution is not available.
- [ ] No arbitrary shell execution occurs.

---

## Documentation Updates Needed After Implementation

Update these after the v0.6.0 implementation pass:

- `Project Status.md`
- `Current Sprint.md`
- `Milestones.md`
- `Architecture.md`
- `Development Journal.md`
- `CHANGELOG.md`
- `Master Knowledge Base.md`
- `Decisions.md`

---

## Recommended Next Prompt

```text
IMPLEMENT PROJECT

Project:
Forge Welcome

Current version:
v0.6.0

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
