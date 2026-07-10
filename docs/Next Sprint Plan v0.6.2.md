# Next Sprint Plan — v0.6.2 Task Progress and Logging

## Sprint

```text
v0.6.2 Sprint 1 — Task Progress and Logging Foundation
```

## Objective

Improve progress visibility and diagnostic logging for the validated inline Development Pack install/uninstall workflow.

## Context

`v0.6.1 — Inline Pack Install Workflow` is complete at final accepted revision `v0.6.1.9`.

The validated workflow now installs and uninstalls Kate through the Development Pack page.

## First Implementation Task

```text
Define and implement clearer install/uninstall task phases for sidebar Tasks (%) and item-card status messages.
```

## Planned Work

- Define install task phases:
  - idle
  - preparing
  - executing
  - refreshing
  - completed
  - failed
  - reboot required
- Define uninstall task phases:
  - idle
  - preparing
  - executing
  - refreshing
  - completed
  - failed
- Improve sidebar `Tasks (%)` accuracy.
- Improve card-level progress/status messages.
- Improve persistent log messages.
- Record command duration and exit status when available.
- Record refresh decisions after package operations.
- Preserve direct install behavior.
- Preserve red trash per-application uninstall behavior.
- Preserve container action guard.
- Preserve `ExecutionBoundary.commands_allowed == true`.

## Non-Goals

- Do not add Gaming Pack real execution.
- Do not add multi-pack execution.
- Do not rename crates.
- Do not perform a broad architecture rewrite.
- Do not remove safety gates.

## Validation

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

## Risks

| Risk | Mitigation |
|---|---|
| Progress UI may become too complex | Keep phase model small and explicit. |
| Logs may leak too much environment detail | Log only safe structured events, not full environment dumps. |
| More workflow code in `main.rs` | Defer broad refactor; extract only if needed. |
| rpm-ostree behavior varies by host state | Record command result and refresh state separately. |
