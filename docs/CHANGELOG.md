# Changelog

All notable project changes for AshGrove Welcome are recorded here.

The project was originally developed as Forge Welcome. Internal Rust crate names currently remain `forge-welcome-*`.

## Versioning

Roadmap milestones use three-part versions. Implementation revisions use four-part versions under their parent roadmap milestone.

## v0.6.2 — Task Progress and Logging

**Status:** Planned

### Planned

- Improve sidebar `Tasks (%)` phase tracking.
- Improve card-level workflow messages.
- Strengthen persistent install/uninstall workflow logging.
- Improve failure/recovery diagnostics.
- Preserve v0.6.1 direct install/uninstall behavior.

## v0.6.1 — Inline Pack Install Workflow

**Status:** Complete

### v0.6.1.0

- Implemented initial inline Development Pack install/uninstall workflow.
- Replaced main legacy dialog-centered Development Pack path with inline page behavior.

### v0.6.1.1

- Removed dry-run/review panel from the Development page.
- Switched `Install Selected` to direct guarded installation.

### v0.6.1.2

- Fixed exhaustive workflow status handling after dry-run UI removal.

### v0.6.1.3

- Added visible card-level progress.
- Updated sidebar `Tasks (%)` during direct install workflow.
- Removed visible `Selected` / `Managed` badges from the list page.

### v0.6.1.4

- Corrected checkbox activation.
- Improved Kate detection and trash-can behavior.

### v0.6.1.5

- Corrected host Kate detection when Kate was not active on the host.

### v0.6.1.6

- Normalized stale or unknown installed states so they do not incorrectly disable the available-card path.

### v0.6.1.7

- Added container runtime action guard to prevent package actions from running inside `forge-dev`.

### v0.6.1.8

- Moved install/uninstall execution off the UI thread.
- Improved behavior when rpm-ostree crashes or fails.
- Replaced checkbox-looking fallback icon with a `K` placeholder.

### v0.6.1.9

- Added pending reboot / active runtime detection behavior for rpm-ostree installs.
- Validated Kate install and uninstall workflow.
- Validated checkbox enabled when Kate is not installed.
- Validated checkbox disabled and red trash can visible after Kate installation.
- Validated red trash action removes Kate only.

### Final Validation

- Final accepted revision: `v0.6.1.9`
- Final validation artifact: `ashgrove_welcome_v0.6.1.9.zip`
- Validation result: Passed
- Code review result: Ready

## v0.6.0 — Production UI/UX Foundation

**Status:** Complete

- Added `ForgeScrollArea.slint`.
- Added `PackItemCard.slint`.
- Added `TaskProgressBar.slint`.
- Added production-style Kate item card.
- Moved `Tasks (%)` into sidebar.
- Added source-aware detection foundation.
