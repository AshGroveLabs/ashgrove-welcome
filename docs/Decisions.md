---
modified: 2026-07-10
type: decisions
project: AshGrove Welcome
status: active
---

# Decisions

## D-0001 — Keep Internal Crate Names for Now

**Status:** Accepted

The public project name is AshGrove Welcome, but internal Rust crate names remain `forge-welcome-*` until a dedicated rename milestone.

## D-0002 — v0.6.x Focuses on Production UI/UX

**Status:** Accepted

`v0.6.x` focuses on production-ready install page behavior before broader multi-pack expansion.

## D-0003 — Build Validation and Runtime Validation Are Separate

**Status:** Accepted

Build validation may run in `forge-dev`. GUI runtime package detection and package actions must run from the host.

## D-0004 — Three-Part Milestones and Four-Part Implementation Revisions

**Status:** Accepted

### Decision

Three-part versions identify roadmap milestones:

```text
v0.6.1
v0.6.2
```

Four-part versions identify implementation revisions:

```text
v0.6.1.0
v0.6.1.1
v0.6.1.9
```

### Rules

- Revision zero is the initial implementation.
- Fixes increment only the final revision number.
- Fixes do not consume the next roadmap milestone.
- Code review uses only the latest implementation revision.
- Validation and release artifacts never overwrite earlier revisions.
- Validation artifacts must use lowercase project names, underscores, and full four-part revisions.

### Examples

```text
ashgrove_welcome_v0.6.1.0.zip
ashgrove_welcome_v0.6.1.1.zip
ashgrove_welcome_v0.6.1.9.zip
```

## D-0005 — v0.6.1 Final Accepted Revision

**Status:** Accepted

`v0.6.1 — Inline Pack Install Workflow` completed at:

```text
v0.6.1.9
```

with final artifact:

```text
ashgrove_welcome_v0.6.1.9.zip
```

## D-0006 — Container Runtime Actions Are Blocked

**Status:** Accepted

The GUI may be visually inspected in a container, but install/uninstall actions must be blocked there. Host package detection must use the host runtime.

## D-0007 — Source-Aware Per-Item Uninstall

**Status:** Accepted**

Installed/removable applications show a red trash action. Trash is per application and does not uninstall the entire pack.
