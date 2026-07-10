---
modified: 2026-07-10
type: project-milestones
project: AshGrove Welcome
legacy_project_name: Forge Welcome
status: active
last_completed_milestone: v0.6.1 — Inline Pack Install Workflow
active_roadmap_milestone: v0.6.2 — Task Progress and Logging
current_implementation_revision: Not created
latest_validation_artifact: ashgrove_welcome_v0.6.1.9.zip
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
v0.6.1.0
v0.6.1.1
v0.6.1.2
```

Revision zero is the initial implementation of the active roadmap milestone. Fixes increment only the final revision number and do not consume the next roadmap milestone.

## Current Milestone Dashboard

| Item | Value |
|---|---|
| Last completed roadmap milestone | `v0.6.1 — Inline Pack Install Workflow` |
| Final accepted revision | `v0.6.1.9` |
| Final validation artifact | `ashgrove_welcome_v0.6.1.9.zip` |
| Code review result | Ready |
| Active roadmap milestone | `v0.6.2 — Task Progress and Logging` |
| Current implementation revision | `Not created` |
| Current workflow state | Milestone complete; Git commit and push pending |
| Next recommended prompt | `PROJECT STATUS`, then `IMPLEMENT PROJECT MILESTONE` after commit/push |

# Completed Milestones

## v0.6.0 — Production UI/UX Foundation

**Status:** Complete

Summary:

- Created the reusable Slint UI foundation for production-ready installable pack pages.
- Added `ForgeScrollArea.slint`, `PackItemCard.slint`, and `TaskProgressBar.slint`.
- Added Kate as the Development Pack validation item.
- Moved `Tasks (%)` into the sidebar.
- Added source-aware detection and uninstall preparation.

## v0.6.1 — Inline Pack Install Workflow

**Status:** Complete

### Roadmap Milestone

- Version: `v0.6.1`
- Objective: Replace the temporary legacy dialog path with inline Development Pack install/uninstall workflow.
- Status: Complete

### Implementation Revisions

| Revision | Type | Description | Validation | Artifact |
|---|---|---|---|---|
| `v0.6.1.0` | Initial implementation | Replaced dialog-centered path with inline Development Pack workflow foundation. | Superseded by fixes | Legacy/generated bundle |
| `v0.6.1.1` | Fix | Removed dry-run/review flow and switched to direct install behavior. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.2` | Fix | Added exhaustive handling for remaining dry-run workflow enum state. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.3` | Fix | Added visible card progress and sidebar `Tasks (%)` updates. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.4` | Fix | Corrected checkbox and trash-can behavior around Kate detection. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.5` | Fix | Corrected host Kate detection when host did not have Kate active. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.6` | Fix | Normalized stale/unknown installed state handling. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.7` | Fix | Added container runtime action guard. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.8` | Fix | Moved install execution off UI thread and reduced rpm-ostree progress-output crash risk. | Superseded by later fixes | Legacy/generated bundle |
| `v0.6.1.9` | Fix | Added pending reboot / final validated install-source behavior. | Passed | `ashgrove_welcome_v0.6.1.9.zip` |

### Final Acceptance

| Item | Result |
|---|---|
| Initial implementation revision | `v0.6.1.0` |
| Final accepted revision | `v0.6.1.9` |
| Number of fix revisions | `9` |
| Final validation artifact | `ashgrove_welcome_v0.6.1.9.zip` |
| Build result | Passed by user validation |
| Validation result | Passed |
| Code review result | Ready |
| Git commit state | Pending |
| Git push state | Pending |

### Completed Acceptance Criteria

- Main Development Pack install flow no longer depends on the legacy modal dialog.
- Kate installs directly from `Install Selected`.
- Installed items are not reinstalled by `Install Selected`.
- Kate uninstall path is routed through per-item red trash action.
- Red trash action removes Kate only.
- Checkbox is enabled when Kate is not installed.
- Checkbox is disabled when Kate is installed.
- Source-aware install/uninstall behavior is preserved.
- Sidebar `Tasks (%)` updates during workflow.
- Item state refreshes after install/uninstall.
- Container runtime action guard blocks accidental package actions from `forge-dev`.
- Real Gaming Pack execution remains disabled.
- No arbitrary shell execution was introduced.

# Current / Planned Milestones

## v0.6.2 — Task Progress and Logging

**Status:** Planned / next

### Roadmap Milestone

- Version: `v0.6.2`
- Objective: Improve visible task progress, workflow logging, and diagnostic evidence for install/uninstall workflows.
- Status: Planned

### Implementation Revisions

| Revision | Type | Description | Validation | Artifact |
|---|---|---|---|---|
| Not created | Initial implementation | Not started | Not run | None |

### Planned Work

- Improve sidebar `Tasks (%)` accuracy and state transitions.
- Improve item-level workflow messages.
- Strengthen persistent log records for install/uninstall workflows.
- Record command start, command completion, exit status, refresh result, and runtime environment.
- Add clearer failure and recovery messages.
- Keep host GUI validation separate from `forge-dev` build validation.
