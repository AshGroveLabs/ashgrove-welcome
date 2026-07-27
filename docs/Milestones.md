---
modified: 2026-07-27
type: project-milestones
project: AshGrove Welcome
legacy_project_name: Forge Welcome
status: milestone-complete
last_completed_milestone: v0.6.2 — Task Progress and Logging
final_accepted_revision: v0.6.2.8
next_planned_milestone: v0.6.3 — Multi-Item Pack Page Preparation
next_workflow_action: COMMIT / PUSH
---

# AshGrove Welcome Milestones

## Versioning Model

Roadmap milestones use three-part versions:

```text
v0.6.1
v0.6.2
v0.6.3
```

Implementation revisions use four-part versions:

```text
v0.6.2.0
v0.6.2.7
v0.6.2.8
```

Revision zero is the initial implementation of the active roadmap milestone. Fixes increment only the final revision number and do not consume the next roadmap milestone.

## Current Milestone Dashboard

| Item | Value |
|---|---|
| Last completed milestone | `v0.6.2 — Task Progress and Logging` |
| Final accepted revision | `v0.6.2.8` |
| Milestone status | Complete |
| Baseline commit | `ce8ce6779beddef8780dd2e6a8039d6fa4c0807b` |
| Commit / push | Pending |
| Next legal workflow action | `COMMIT / PUSH` |
| Next planned milestone | `v0.6.3 — Multi-Item Pack Page Preparation` |
| v0.6.3 status | Not started |

# Completed Milestones

## v0.6.2 — Task Progress and Logging

**Status:** Complete

**Final accepted revision:** `v0.6.2.8`

### Summary

- Improved visible task progress for real Development Pack package workflows.
- Added rpm-ostree stdout/stderr capture and progress parsing.
- Added persistent structured runtime logs for progress, command output, command result, refresh result, and final UI state.
- Preserved source-aware Kate install/remove behavior and safety gates.
- Corrected Kate source classification and fail-closed unknown-source behavior.
- Cleaned up Development Pack and placeholder pack page layout density.
- Updated stale milestone documentation.

### Implementation Revision Chain

| Revision | Result | Notes |
|---|---|---|
| `v0.6.2.0` | Failed validation | Initial task-phase model; real workflow progress moved only `0 -> 55 -> 100`. |
| `v0.6.2.1` | Superseded | Added rpm-ostree stdout/stderr streaming and progress parsing. |
| `v0.6.2.2` | Superseded | Kept sidebar/card text concise while raw rpm-ostree output remained in logs. |
| `v0.6.2.3` | Failed validation | Attempted Kate source-classification fix; red trash still did not display. |
| `v0.6.2.4` | Superseded | Replaced brittle rpm-ostree JSON scanning with structured `serde_json` parsing. |
| `v0.6.2.5` | Validated behavior to preserve | Corrected product rule so `Managed` is not a normal fallback for Kate pack cards. |
| `v0.6.2.6` | Validated behavior to preserve | Cleaned up pack page layout and pack title consistency. |
| `v0.6.2.7` | CODE REVIEW blocked | Compact card/page layout was accepted, but persistent structured logs and docs were incomplete. |
| `v0.6.2.8` | Final accepted | Added persistent structured runtime log events and corrected stale milestone documentation. |

### Validation Result

- BUILD AND VALIDATE passed.
- CODE REVIEW approved `v0.6.2.8` for MILESTONE HANDOFF REVIEW.
- MILESTONE HANDOFF REVIEW was produced and accepted by the user.

### Process Note

The `v0.6.2` handoff was accepted despite lacking full code-symbol-level walkthrough detail. Future code milestone handoff reviews must include code-symbol-level walkthroughs.

### Non-Blocking Deferrals

- rpm-ostree progress parsing may still be line-fragment fragile.
- Workflow orchestration remains concentrated in GUI `main.rs`.
- Typed rpm-ostree structs are deferred.
- Full rpm-ostree D-Bus transaction client is deferred.
- Installation Page / Multi-Pack Installation Queue is deferred.

## v0.6.0 — Production UI/UX Foundation

**Status:** Complete

Summary:

- Created the reusable Slint UI foundation for production-ready installable pack pages.
- Added `ForgeScrollArea.slint`, `PackItemCard.slint`, and `TaskProgressBar.slint`.
- Added Kate as the Development Pack validation item.
- Moved `Tasks (%)` into the sidebar.
- Added source-aware detection and uninstall preparation.

# Historical Roadmap Context

## v0.6.1 — Inline Pack Install Workflow

**Status:** Historical checkpoint for current repository state

`v0.6.1.15 — Application Catalog Foundation` remains recorded as prior foundation work. It is not the active milestone for this completion state.

# Planned Milestones

## v0.6.3 — Multi-Item Pack Page Preparation

**Status:** Planned; not started.

Do not start `v0.6.3` until `v0.6.2` commit/push is complete and a refreshed PROJECT STATUS authorizes the next milestone.
