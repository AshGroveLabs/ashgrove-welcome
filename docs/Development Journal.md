---
modified: 2026-07-14
type: development-journal
project: AshGrove Welcome
status: active
---

# Development Journal

## 2026-07-14 — v0.6.1 Final Milestone Closure

MILESTONE HANDOFF REVIEW approved `v0.6.1.12` as the final accepted revision with non-blocking deferrals. CODE REVIEW was **APPROVED WITH NON-BLOCKING NOTES**, the corrected CODE CHANGE WALKTHROUGH was accepted, and manual host GUI validation passed the complete available → scheduled install → reboot → installed → scheduled removal → reboot → available lifecycle.

Findings `F-001` through `F-004` are closed. The **System Update Scheduled** replacement card is accepted behavior for rpm-ostree reboot-required install/removal states. Deferred notes are revision-specific automated-validation documentation, manual-validation record hygiene, and rpm-ostree detection/parser technical debt.

No source code was changed for milestone closure. The next legal workflow action is `COMMIT / PUSH`; `v0.6.2 — Task Progress and Logging` must wait until commit/push and source refresh are complete.

## 2026-07-10 — v0.6.1 Preliminary Closure (Superseded)

### Summary

This preliminary closure record was superseded by the corrective review and implementation chain ending at `v0.6.1.12`.

The final accepted implementation revision is:

```text
v0.6.1.12
```

The final validation artifact is:

```text
ashgrove_welcome_v0.6.1.9.zip
```

### What Changed

The milestone replaced the temporary legacy dialog-centered install path with inline Development Pack workflow behavior.

Validated behavior:

- Kate installs through `Install Selected`.
- Kate uninstalls through the red trash action.
- rpm-ostree install status worked.
- rpm-ostree uninstall status worked.
- Checkbox is enabled when Kate is not installed.
- Checkbox is disabled after Kate is installed.
- Red trash can displays after Kate installation.
- Red trash can removes Kate successfully.

### Fix History

The milestone started as `v0.6.1.0`. The v0.6.1.9 candidate subsequently entered corrective review, and the final accepted revision is `v0.6.1.12`.

The fixes addressed:

- Removing dry-run/review UI.
- Direct install behavior.
- Exhaustive workflow status handling.
- Card progress and sidebar task percentage.
- Checkbox/trash behavior.
- Host Kate detection.
- Unknown/stale state normalization.
- Container runtime action guard.
- UI freeze/rpm-ostree crash resilience.
- Pending reboot / active runtime detection.

### Versioning Migration

This milestone was also the first active milestone to adopt the Milestone and Fix Versioning Directive.

Previous behavior:

- Milestone and fix work were sometimes discussed using the same three-part milestone number.

New behavior:

- Roadmap milestones use three-part versions, such as `v0.6.1`.
- Implementation revisions use four-part versions, such as `v0.6.1.0`.
- Fix revisions increment only the final number, such as `v0.6.1.1`.
- Fixes do not consume the next roadmap milestone.
- Code review uses only the latest revision.
- Validation artifacts use the full four-part revision.

### Current State

- Last completed milestone: `v0.6.1 — Inline Pack Install Workflow`
- Final accepted revision: `v0.6.1.12`
- Next milestone: `v0.6.2 — Task Progress and Logging`
- Git commit state: Pending
- Git push state: Pending

### Uncertainty

No unresolved functional blocker remains based on user validation.

The only pending process item is committing and pushing the final source and documentation changes before starting `v0.6.2`.
