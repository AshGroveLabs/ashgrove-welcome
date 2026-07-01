---
modified: 2026-07-01
type: decisions
project: Forge Welcome
parent_project: Forge OS
status: active
---

# Decisions

## DEC-2026-07-01-001 — Reframe v0.6.x as Production UI/UX Finalization

### Status

Accepted

### Context

The prior v0.6.0 plan focused on Gaming Pack UI expansion. During Kate installation validation and UI review, the current interface was found to need production-level UI/UX work before expanding additional pack pages.

### Decision

Reframe v0.6.x as:

```text
v0.6.x — Production UI/UX Finalization
```

The first implementation milestone becomes:

```text
v0.6.0 — Production UI/UX Foundation
```

Gaming Pack expansion is deferred until the shared production UI foundation is implemented.

### Consequences

- Project documents must replace the Gaming Pack-first v0.6.0 direction.
- New v0.6.0 work starts with reusable UI components.
- Future pack pages will use the same installable item card and progress model.

---

## DEC-2026-07-01-002 — Implement ForgeScrollArea During v0.6.x

### Status

Accepted

### Context

`ForgeScrollArea` was previously deferred until later v0.6.x work. The new production UI/UX milestone requires scrollable installable item sections immediately.

### Decision

Implement `ForgeScrollArea.slint` during the active v0.6.x milestone.

### Consequences

- Scrollable UI becomes part of the current milestone scope.
- Development Pack item list should use the new scroll component first.
- Future pack pages should reuse this component.

---

## DEC-2026-07-01-003 — Move Main Install UX Inline

### Status

Accepted

### Context

The current install flow uses an installation review dialog. The desired production UX uses the main pack page as the review and action surface.

### Decision

The production install flow should be inline:

```text
Pack Page
    ↓
Selectable item list
    ↓
Install Selected
    ↓
Inline progress/status
    ↓
Inline result summary
```

Installation sub-dialogs are not part of the final production install UX.

### Consequences

- `InstallReviewDialog.slint` may remain temporarily during migration.
- Future implementation should migrate command execution and result display into the page workflow.
- The visible selected item list becomes the user's review surface.

---

## DEC-2026-07-01-004 — Use Discover-Inspired Pack Item Cards

### Status

Accepted

### Context

KDE Discover provides a clear pattern for installable application lists and update progress.

### Decision

Forge Welcome pack pages should use Discover-inspired item cards containing:

- checkbox
- icon
- app name
- description
- source/package metadata
- right-side action/status area

Installed items may show a red trash/remove action only where semantically valid.

### Consequences

- Pack pages should not use plain text lists for installable applications.
- UI state must support available, selected, installed, installing, failed, skipped, and blocked states.
- The pattern should be reusable across Development, Gaming, Productivity, Cloud & Sync, and Forge Ecosystem pages.

---

## DEC-2026-07-01-005 — Preserve Existing Safety Model

### Status

Accepted

### Context

v0.5.9 stabilized the guarded execution workflow.

### Decision

The production UI/UX work must not weaken the installation safety model.

Required rule:

```rust
execution_plan.command_boundary.commands_allowed == true
```

The following remains insufficient:

```rust
dry_run == false
```

### Consequences

- UI changes must preserve dry-run safety.
- Real execution still requires explicit user action.
- Real Gaming Pack execution remains disabled.
- Arbitrary shell execution remains forbidden.

---

## DEC-2026-07-01-006 — Add Persistent Workflow Logging

### Status

Accepted

### Context

Kate installation took a long time and required better diagnostics.

### Decision

Forge Welcome should write workflow logs to:

```text
~/.local/state/forge-welcome/forge-welcome.log
```

Logs should include workflow events, command start/end, exit code, duration, and refresh result.

### Consequences

- Logging becomes part of v0.6.x production UI/UX work.
- Logs must not contain passwords, tokens, full environment dumps, or secrets.
