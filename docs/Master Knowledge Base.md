---
modified: 2026-07-27
type: master-knowledge-base
project: AshGrove Welcome
status: active
last_completed_milestone: v0.6.2 — Task Progress and Logging
final_accepted_revision: v0.6.2.8
next_planned_milestone: v0.6.3 — Multi-Item Pack Page Preparation
v0.6.3_status: Planned / not started
current_workflow_state: post-v0.6.2 documentation consistency correction pending PROJECT STATUS
---

# Master Knowledge Base

## Project Identity

| Field | Value |
|---|---|
| Project | AshGrove Welcome |
| Legacy source name | Forge Welcome |
| Internal crate names | `forge-welcome-*` |
| Repository | `https://github.com/AshGroveLabs/ashgrove-welcome` |
| Last completed milestone | `v0.6.2 — Task Progress and Logging` |
| Final accepted revision | `v0.6.2.8` |
| Next planned milestone | `v0.6.3 — Multi-Item Pack Page Preparation` |
| v0.6.3 status | Planned / not started |
| Current workflow state | Post-v0.6.2 documentation consistency correction pending PROJECT STATUS |

## Versioning Knowledge

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
v0.6.1.12
v0.6.2.8
```

Rules:

- Revision zero is the initial implementation.
- Fixes increment only the final revision number.
- Fixes do not consume the next roadmap milestone.
- Code review uses the latest revision only.
- Artifacts never overwrite earlier revisions.
- Validation artifact filenames use lowercase names, underscores, and full four-part revisions.

## Workflow Process Notes

Future code milestone prompts must include a Documentation Sync Ledger. IMPLEMENT PROJECT and IMPLEMENT FIX must state whether Project Status, Current Sprint, Milestones, Roadmap, CHANGELOG, Development Journal, Architecture, Decisions, Master Knowledge Base, and milestone reports were updated, not needed, or deferred with reason. BUILD AND VALIDATE and CODE REVIEW must include a documentation consistency check. MILESTONE HANDOFF REVIEW must include a code-symbol-level walkthrough.

## Completed v0.6.1 Knowledge

### Inline Workflow

```text
Kate checkbox
    ↓
Install Selected
    ↓
Card progress
    ↓
Sidebar Tasks (%)
    ↓
rpm-ostree install
    ↓
Refresh state
    ↓
Installed card with red trash
```

### Uninstall Workflow

```text
Red trash
    ↓
Kate-only uninstall
    ↓
rpm-ostree uninstall
    ↓
Refresh state
    ↓
Available card with checkbox
```

## Detection Knowledge

Useful checks:

```bash
rpm -q kate
command -v kate
rpm-ostree status
flatpak info --system org.kde.kate
flatpak info --user org.kde.kate
```

Important distinction:

- `rpm -q kate` and `command -v kate` show active runtime availability.
- `rpm-ostree status` shows deployment state and pending changes.
- `forge-dev` package state is not host package state.

## Safety Knowledge

Standing rule:

```text
ExecutionMode is intent.
ExecutionBoundary is permission.
```

No package command may execute unless:

```rust
execution_plan.command_boundary.commands_allowed == true
```

Runtime rule:

```text
Build validation may run in forge-dev.
GUI package detection and install/uninstall validation must run on host.
```

## Completed v0.6.2 Knowledge

`v0.6.2 — Task Progress and Logging`

Completed goals:

- Improve sidebar task progress state.
- Improve item-level workflow text.
- Improve persistent logging.
- Improve failure and recovery diagnostics.

## Active Next Focus

`v0.6.3 — Multi-Item Pack Page Preparation`

Status: Planned / not started.
