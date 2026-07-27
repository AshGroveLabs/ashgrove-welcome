# Changelog

All notable project changes for AshGrove Welcome are recorded here.

The project was originally developed as Forge Welcome. Internal Rust crate names currently remain `forge-welcome-*`.

## Versioning

Roadmap milestones use three-part versions. Implementation revisions use four-part versions under their parent roadmap milestone.

## v0.6.2 — Task Progress and Logging

**Status:** Complete

**Final accepted revision:** `v0.6.2.8`

### Completed

- Improved visible task progress for the Development Pack Kate install/remove workflow.
- Added rpm-ostree stdout/stderr capture, streaming progress parsing, and progress clamping so live UI progress stays below `100%` until command completion and refresh.
- Kept compact sidebar/card UI messages concise while preserving raw rpm-ostree output in logs and command results.
- Replaced brittle rpm-ostree JSON scanning with structured `serde_json` parsing.
- Corrected Kate source classification so confirmed layered/removable Kate exposes red trash and unknown source evidence fails closed.
- Corrected the product rule that `Managed` is not a normal fallback state for Kate pack cards.
- Cleaned up pack page naming, header density, application card layout, source labels, icon sizing, and the `Install Selected` control.
- Added persistent structured runtime log events for `progress_transition`, `command_output_line`, `command_result`, `refresh_result`, and `final_ui_state`.
- Corrected stale milestone documentation after implementation and review cycles.

### Validation

- BUILD AND VALIDATE passed for `v0.6.2.8`.
- CODE REVIEW approved `v0.6.2.8` for MILESTONE HANDOFF REVIEW.
- MILESTONE HANDOFF REVIEW was produced and accepted by the user.

### Process Note

The `v0.6.2` handoff was accepted despite lacking full code-symbol-level walkthrough detail. Future code milestone handoff reviews must include code-symbol-level walkthroughs.

## v0.6.2.8 — Persistent Runtime Logging Fix

### Fixed

- Restored persistent structured runtime log events for task progress, command output lines, command completion, post-command refresh, and final UI state.
- Corrected milestone documentation that still described `v0.6.1.15` as the active checkpoint and `v0.6.2` as blocked before implementation.

## v0.6.2.7 — Pack Page Density Cleanup

### Changed

- Compact pack page titles, Development Pack list spacing, application card sizing, icon sizing, and source labels.
- Shortened current pack application descriptions so cards fit cleanly without restoring removed pack summary text.

## v0.6.2.6 — Pack Page Layout Cleanup

### Changed

- Renamed pack content page titles to use the `<Name> Pack` pattern.
- Removed redundant pack-page subtitle and summary-header presentation before the application list.
- Reduced the Development Pack `Install Selected` control size while preserving its enabled, disabled, and click behavior.

## v0.6.2.5 — Development Pack Source Attention State

### Fixed

- Stopped presenting active, non-removable Kate as a normal `Managed` Development Pack card state.
- Active Kate now shows removal only when structured rpm-ostree booted layered/requested evidence confirms the removable layered source.
- Active Kate without confirmed layered/removable source evidence fails closed as a source attention state with checkbox and trash hidden.

### Changed

- Development Pack application cards treat unknown or conflicting installed-source evidence as detection issues rather than completed managed items.

## v0.6.2.2 — Task Progress UI Refinement

### Fixed

- Kept rpm-ostree progress detail out of compact application-card status areas.
- Kept sidebar `Tasks (%)` progress concise and bounded during live command execution.
- Preserved full rpm-ostree stdout/stderr detail in command results and logs.

### Deferred

- Installation Page / Multi-Pack Installation Queue remains future work. It should list selected applications across packs as install cards with per-application progress bars while the sidebar keeps overall `Tasks (%)` progress.

## v0.6.1.15 — Application Catalog Foundation

**Status:** Complete — historical checkpoint

### Added

- Application catalog.
- Pack membership by stable application ID.
- Typed install variants.
- Flatpak-first resolver.
- Validated application and pack configuration.
- Trusted manifest runtime search.
- Lifecycle and planning models.

### Changed

- Kate compatibility now resolves through stable catalog identity.
- Runtime lifecycle uses invariant-safe transitions.

### Fixed

- Invalid catalogs becoming accessible before validation.
- Non-install-safe manifest lookup.
- Unvalidated detection/display identifiers.
- Contradictory lifecycle states.
- `Installed` lifecycle accepting non-installed evidence.

### Deferred

- Multi-item Slint model and source selector.
- Grouped installation.
- Generic removal.
- D-Bus progress and the Installation screen.

## v0.6.1 — Inline Pack Install Workflow

**Status:** Historical checkpoint for current repository state

`v0.6.1.15` remains recorded as the application catalog foundation checkpoint. It is not the active project checkpoint for this repository state.

Remaining planned corrective revisions:

- `v0.6.1.16 — Multi-Item Slint Model and Source Selector`
- `v0.6.1.17 — Grouped Install Execution`
- `v0.6.1.18 — Generic Removal and Final Foundation Closure`

## v0.6.0 — Production UI/UX Foundation

**Status:** Complete

- Added `ForgeScrollArea.slint`.
- Added `PackItemCard.slint`.
- Added `TaskProgressBar.slint`.
- Added production-style Kate item card.
- Moved `Tasks (%)` into sidebar.
- Added source-aware detection foundation.
