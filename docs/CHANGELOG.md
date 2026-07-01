# CHANGELOG

All notable changes to Forge Welcome are tracked here.

## [Unreleased]

### Changed

- Reframed the active v0.6.x direction from Gaming Pack-first expansion to `v0.6.x — Production UI/UX Finalization`.
- Replaced the planned v0.6.0 Gaming Pack-first sprint with `v0.6.0 — Production UI/UX Foundation`.
- Moved `ForgeScrollArea` from deferred/future work into the active v0.6.x milestone scope.
- Updated install UX direction from dialog-centered installation to inline pack-page installation.
- Defined `Install Selected` as the target explicit user action for selected pack items.
- Marked installation sub-dialogs as non-final for the production install UX.
- Deferred Gaming Pack UI expansion until the shared production UI/UX foundation is implemented.

### Added

- Planned reusable Slint component: `ForgeScrollArea.slint`.
- Planned reusable Slint component: `PackItemCard.slint`.
- Planned reusable Slint component: `TaskProgressBar.slint`.
- Planned Discover-style installable item cards for pack pages.
- Planned lower-left `Tasks (%)` global progress indicator.
- Planned persistent workflow log at `~/.local/state/forge-welcome/forge-welcome.log`.
- Planned inline per-item progress and status states.

### Deprecated

- Dialog-centered installation workflow as the final production UX.
- `Review Installation` wording for the main install button.

### Safety

- Reaffirmed that dry-run safety remains required.
- Reaffirmed that real execution requires explicit user action.
- Reaffirmed that `ExecutionBoundary.commands_allowed == true` remains the execution gate.
- Reaffirmed that real Gaming Pack execution remains disabled.
- Reaffirmed that arbitrary shell execution remains forbidden.

---

## [v0.5.9] — Installation Workflow Stabilization

### Completed

- Stabilized guarded Development Pack installation workflow.
- Added `ExecutionWorkflowStatus`.
- Added execution-report helper methods.
- Added command-result helper methods.
- Reduced duplication in command/result mapping.
- Preserved dry-run behavior.
- Preserved explicit confirmation and execution boundary gate.
- Preserved Development Pack-only real execution.
- Avoided arbitrary shell execution.
- Completed formatter, build, lint, tests, and GUI validation.
