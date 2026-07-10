# AshGrove Welcome

AshGrove Welcome is the desktop welcome and setup application for the AshGrove OS / Forge OS ecosystem.

The project was originally developed as Forge Welcome. Internal Rust crate names currently remain `forge-welcome-*`.

## Current Status

| Field | Value |
|---|---|
| Last completed milestone | `v0.6.1 — Inline Pack Install Workflow` |
| Final accepted revision | `v0.6.1.9` |
| Latest validation artifact | `ashgrove_welcome_v0.6.1.9.zip` |
| Active next milestone | `v0.6.2 — Task Progress and Logging` |

## Purpose

AshGrove Welcome provides a first-run and post-install setup experience for the desktop system.

Current validated scope:

- Development Pack Kate install.
- Development Pack Kate uninstall.
- Inline install/uninstall workflow.
- Sidebar task status.
- Host-only package detection.
- Source-aware red trash uninstall action.

## Repository Layout

```text
ashgrove-welcome/
├── Cargo.toml
├── Cargo.lock
├── crates/
│   ├── forge-welcome-cli/
│   ├── forge-welcome-core/
│   └── forge-welcome-gui/
├── manifests/
├── assets/
├── scripts/
└── docs/
```

## Build

```bash
cargo fmt --all
cargo check
cargo clippy
cargo test
cargo build -p forge-welcome-gui
```

Build validation may run inside `forge-dev` if Cargo is unavailable on the host.

## Run GUI

GUI runtime package detection and package actions must run from the host.

```bash
cd ~/01_projects/dev/ashgrove-welcome
test -f /run/.containerenv && echo "container - stop" && exit 1 || echo "host - OK"
./target/debug/forge-welcome-gui
```

## Safety Model

Standing rule:

```text
ExecutionMode is intent.
ExecutionBoundary is permission.
```

No package command may execute unless:

```rust
execution_plan.command_boundary.commands_allowed == true
```

## Versioning

Roadmap milestones use three-part versions:

```text
v0.6.1
v0.6.2
```

Implementation revisions use four-part versions:

```text
v0.6.1.0
v0.6.1.9
```

Validation artifacts use lowercase names, underscores, and the full four-part revision:

```text
ashgrove_welcome_v0.6.1.9.zip
```

## Documentation

Key documents:

- `docs/Project Status.md`
- `docs/Milestones.md`
- `docs/Current Sprint.md`
- `docs/Architecture.md`
- `docs/Master Knowledge Base.md`
- `docs/Development Journal.md`
- `docs/Decisions.md`
- `docs/CHANGELOG.md`
