---
modified: 2026-07-10
type: master-knowledge-base
project: AshGrove Welcome
status: active
last_completed_milestone: v0.6.1 — Inline Pack Install Workflow
active_roadmap_milestone: v0.6.2 — Task Progress and Logging
---

# Master Knowledge Base

## Project Identity

| Field | Value |
|---|---|
| Project | AshGrove Welcome |
| Legacy source name | Forge Welcome |
| Internal crate names | `forge-welcome-*` |
| Repository | `https://github.com/AshGroveLabs/ashgrove-welcome` |
| Last completed milestone | `v0.6.1 — Inline Pack Install Workflow` |
| Final accepted revision | `v0.6.1.9` |
| Active roadmap milestone | `v0.6.2 — Task Progress and Logging` |

## Versioning Knowledge

Roadmap milestones use three-part versions:

```text
v0.6.1
v0.6.2
```

Implementation revisions use four-part versions:

```text
v0.6.1.0
v0.6.1.1
v0.6.1.9
```

Rules:

- Revision zero is the initial implementation.
- Fixes increment only the final revision number.
- Fixes do not consume the next roadmap milestone.
- Code review uses the latest revision only.
- Artifacts never overwrite earlier revisions.
- Validation artifact filenames use lowercase names, underscores, and full four-part revisions.

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

## Active Next Focus

`v0.6.2 — Task Progress and Logging`

Goals:

- Improve sidebar task progress state.
- Improve item-level workflow text.
- Improve persistent logging.
- Improve failure and recovery diagnostics.
