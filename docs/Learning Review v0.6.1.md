# Learning Review — v0.6.1 Inline Pack Install Workflow

## Summary

`v0.6.1` moved AshGrove Welcome from a dialog-centered install model to a direct inline Development Pack workflow.

The milestone reinforced Rust GUI workflow management, Slint state rendering, host/package detection, rpm-ostree behavior, and safety-first command execution.

## Rust Concepts Reinforced

- Enum-driven install-source modeling.
- Exhaustive pattern matching.
- Background worker execution to avoid blocking the UI thread.
- `Result`-based command handling.
- Host/container runtime checks.
- Separating active runtime state from pending deployment state.
- Handling command failure and crash-prone external tools.

## Slint Concepts Reinforced

- Card-based application state rendering.
- Checkbox state management.
- Per-item progress display.
- Sidebar status display.
- Callback wiring between Slint and Rust.
- Avoiding misleading visual affordances, such as a fallback icon that looked like a checkbox.

## Linux / rpm-ostree Concepts Reinforced

- rpm-ostree installs may stage a deployment requiring reboot.
- Active package state and pending deployment state are different.
- `rpm -q` and `command -v` reflect active runtime state.
- `rpm-ostree status` is needed to inspect deployment state.
- Container package state is not host package state.

## Software Engineering Concepts Reinforced

- Documentation-first milestone closure.
- Versioned fix revisions.
- Validation evidence capture.
- Host-only runtime validation.
- Guarding dangerous actions from incorrect environments.
- Keeping implementation scope narrow.

## Architecture Lessons

- Execution should never happen on the UI thread.
- Detection state must distinguish:
  - not installed
  - installed and removable
  - installed but managed
  - pending reboot
  - unknown/non-actionable
- UI state should be driven by source-of-truth package detection, not icon presence.
- Container and host runtime behavior must be explicitly separated.

## C++ / Qt Comparison

| Rust / Slint Concept | C++ / Qt Equivalent |
|---|---|
| Background worker thread | `QThread` / `QtConcurrent` |
| Slint callbacks | Qt signals and slots |
| `Result` error handling | `std::expected` or explicit result structs |
| Exhaustive enum matching | `enum class` plus switch/default discipline |
| State-driven card rendering | QML property bindings |
| Command execution guard | `QProcess` with controlled args and privilege boundary |

## Portfolio Value

This milestone demonstrates:

- Rust GUI development.
- Linux package workflow integration.
- rpm-ostree system tooling awareness.
- Production UI workflow iteration.
- Debugging host/container issues.
- Safety-first command execution.
- Documentation-driven engineering process.

## Content Opportunities

- Building direct install workflows in a Rust + Slint desktop app.
- Why GUI applications should not run package operations on the UI thread.
- rpm-ostree active state vs pending deployment state.
- Designing safe uninstall buttons.
- Structuring milestone and fix revisions in a software project.
