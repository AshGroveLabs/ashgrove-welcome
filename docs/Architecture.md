---
modified: 2026-07-27
type: architecture
project: AshGrove Welcome
status: active
active_roadmap_milestone: v0.6.2 — Task Progress and Logging
current_implementation_revision: v0.6.2.8 — Persistent Runtime Logging Fix
---

# Architecture

## Current Architecture State

| Area | State |
|---|---|
| Active roadmap milestone | `v0.6.2 — Task Progress and Logging` |
| Current corrective revision | `v0.6.2.8 — Persistent Runtime Logging Fix` |
| Corrective revision status | IMPLEMENT FIX; BUILD AND VALIDATE next |
| Failed review revision | `v0.6.2.7` |
| GUI framework | Slint |
| Language | Rust |
| Package workflow target | Host rpm-ostree / Flatpak workflows |
| Runtime validation | Host-only |
| Build validation | May run in `forge-dev` |

## Active v0.6.2 Architecture

### Persistent Runtime Logging

Runtime troubleshooting uses the existing AshGrove Welcome log path:

- `$XDG_STATE_HOME/ashgrove-welcome/ashgrove-welcome.log`
- `$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log`
- `/tmp/ashgrove-welcome.log` when no home directory is available

The runtime log must persist structured, grep-friendly events for:

- `progress_transition`
- `command_output_line`
- `command_result`
- `refresh_result`
- `final_ui_state`

rpm-ostree stdout/stderr lines are logged as `command_output_line` events for troubleshooting. Compact card/sidebar UI remains concise and must not surface raw package-manager output in cramped status fields.

### Task Progress Boundary

Live command progress is clamped below 100% until process completion and post-command refresh. Final 100% UI state is applied only after command completion and refreshed package state/fallback state have been applied.

## Accepted Foundation Architecture

### Application Catalog

The application catalog is the canonical source for stable application identity and install metadata. Runtime state must not define catalog identity. A pack refers to applications by catalog application ID instead of carrying independent package definitions.

### Pack Membership

Pack manifests describe membership by application ID. Pack membership answers which catalog applications belong to a pack; application definitions answer what those applications are and how they may be installed.

### Typed Install Variants

Trusted install configuration uses typed variants instead of loosely interpreted backend strings. Variants describe supported installation sources such as Flatpak and rpm-ostree with explicit backend-specific identifiers.

### Flatpak-First Source Resolution

Source resolution prefers Flatpak when a valid Flatpak variant is configured. rpm-ostree is used only when configured and selected by policy. The resolver must not silently fall back to another backend when the selected or preferred source is invalid.

The production Flatpak default is system installation unless a later approved revision changes that policy.

### Validated Trusted Configuration

Manifest data is wrapped in validated application and pack types before use by install planning, display state, or runtime detection. Invalid trusted configuration must fail before becoming accessible as usable catalog state.

### Trusted Manifest Runtime Search

Runtime manifest discovery searches complete trusted manifest directories. It must not depend on install-time unsafe current-directory assumptions.

### Identifier Validation

Application IDs, display identifiers, Flatpak refs, and rpm-ostree package names are validated before they are accepted into trusted runtime configuration or detection paths.

### Enum-Driven Lifecycle

Runtime lifecycle state is represented through explicit enum variants instead of contradictory booleans. UI and planning code must derive behavior from the lifecycle variant and associated evidence.

### Installed-Source Invariant

An application may enter `Installed` only with active detected-source evidence. Pending rpm-ostree deployment evidence is not the same as active installed-source evidence.

### Development Pack Visible State

For Development Pack application cards, `Managed` is not a normal final visible state. Kate is removable only when active host evidence is paired with structured rpm-ostree booted layered/requested package evidence. Active Kate without confirmed removable/layered evidence must fail closed as a detection issue such as source unknown or source conflict: checkbox hidden, trash hidden, and exact probe evidence logged.

### Planning Records

Planning records separate catalog configuration, selected install source, detected source, and lifecycle state. This foundation supports later multi-item and grouped execution work without implementing grouped execution in the current `v0.6.2.8` fix.

### Compatibility Boundary

Kate remains the compatibility validation item, but it is no longer the permanent singular architecture. Kate resolves through stable catalog identity under the same catalog and pack-membership model that later applications will use.

## Deferred Scope

Clearly deferred beyond `v0.6.2.8`:

- Multi-item Slint model and source selector.
- Grouped execution.
- Generic removal.
- D-Bus progress and the Installation screen.
- Future Installation Page / Multi-Pack Installation Queue: list selected applications across packs as install cards with per-application progress bars while the sidebar keeps overall `Tasks (%)` progress.

## Runtime Environment Rule

Package detection and package actions must not run from `forge-dev`.

Host preflight:

```bash
test -f /run/.containerenv && echo "container - stop" && exit 1 || echo "host - OK"
```

## Safety Rules

1. Preserve explicit user action before package operations.
2. Preserve `ExecutionBoundary.commands_allowed == true` as the execution gate.
3. Do not introduce arbitrary shell execution.
4. Do not enable unapproved package execution.
5. Do not execute package actions inside `forge-dev`.
6. Do not log secrets, tokens, passwords, or full environment dumps.
