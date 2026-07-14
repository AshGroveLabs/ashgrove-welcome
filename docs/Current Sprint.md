---
modified: 2026-07-14
type: current-sprint
project: AshGrove Welcome
legacy_project_name: Forge Welcome
status: planned
active_roadmap_milestone: v0.6.2 — Task Progress and Logging
current_implementation_revision: Not created
final_accepted_revision: v0.6.1.12
workflow_state: Milestone complete; Git commit and push pending
---

# Current Sprint

## Version State

- Last completed milestone: `v0.6.1 — Inline Pack Install Workflow`
- Active roadmap milestone: `v0.6.2 — Task Progress and Logging`
- Current implementation revision: `Not created`
- Current workflow stage: `Milestone complete; Git commit and push pending`
- Current validation result: `v0.6.1.12 full host GUI lifecycle passed`
- Blocking defects: `F-001` through `F-004` closed; none remaining
- Handoff result: `APPROVED WITH NON-BLOCKING DEFERRALS`
- Next legal workflow action: `COMMIT / PUSH`

## Sprint Dashboard

| Field | Value |
|---|---|
| Project | AshGrove Welcome |
| Last completed milestone | `v0.6.1 — Inline Pack Install Workflow` |
| Final accepted revision | `v0.6.1.12` |
| Active roadmap milestone | `v0.6.2 — Task Progress and Logging` |
| Current implementation revision | `Not created` |
| Sprint | `v0.6.2 Sprint 1 — Task Progress and Logging Foundation` |
| Status | Planned |
| Git commit state | Pending |
| Git push state | Pending |

# Previous Sprint Closure

`v0.6.1 Sprint 1 — Inline Development Pack Workflow` is complete.

Validated behavior:

- Kate installation passed.
- Kate uninstallation passed.
- rpm-ostree install status worked.
- rpm-ostree uninstall status worked.
- Checkbox enabled when Kate is not installed.
- Checkbox disabled after Kate installation.
- Red trash can displayed after Kate installation.
- Red trash can removed Kate successfully.
- Scheduled rpm-ostree installation/removal replaces Kate with the non-actionable **System Update Scheduled** card, identifies the operation, requires reboot, and reaches 100% task progress.

The corrected CODE CHANGE WALKTHROUGH `v0.6.1.12` was accepted. Remaining non-blocking deferrals are revision-specific automated-validation documentation, manual-validation record hygiene, and rpm-ostree detection/parser technical debt.

# Next Sprint

## v0.6.2 Sprint 1 — Task Progress and Logging Foundation

### Sprint Goal

Strengthen visible progress and persistent diagnostic logging for the validated inline install/uninstall workflow.

### Planned Tasks

- [ ] Review current sidebar `Tasks (%)` state transitions.
- [ ] Define task phases for install and uninstall workflows.
- [ ] Improve progress messages for preparing, executing, refreshing, completed, failed, and reboot-required states.
- [ ] Ensure logs record runtime environment, command start, command completion, exit status, refresh result, and final UI state.
- [ ] Add clearer failure/recovery output for rpm-ostree and Flatpak flows.
- [ ] Preserve direct install behavior from v0.6.1.
- [ ] Preserve source-aware red trash uninstall.
- [ ] Preserve container runtime action guard.
- [ ] Preserve `ExecutionBoundary.commands_allowed == true` as execution gate.
- [ ] Preserve no real Gaming Pack execution.

### Acceptance Criteria

- Sidebar `Tasks (%)` reflects meaningful install/uninstall phases.
- Card-level progress and text remain synchronized with workflow state.
- Log records allow reconstruction of each install/uninstall attempt.
- Failure logs distinguish command failure, detection failure, and refresh mismatch.
- Host-vs-container runtime state is logged before any package action.
- No secrets or full environment dumps are logged.
- Build validation passes.
- Host GUI validation passes.

### Validation Plan

```bash
cd ~/01_projects/dev/ashgrove-welcome

cargo fmt --all
cargo check
cargo clippy
cargo test
cargo build -p forge-welcome-gui

test -f /run/.containerenv && echo "container - stop" && exit 1 || echo "host - OK"
./target/debug/forge-welcome-gui
```

### Current Blocker

Commit and push `v0.6.1` before starting implementation on `v0.6.2`.

Source refresh must also be complete before `v0.6.2` starts.
