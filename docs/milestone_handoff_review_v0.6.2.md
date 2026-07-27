# Milestone Handoff Review — v0.6.2 Task Progress and Logging

## 1. Executive Summary

`v0.6.2 — Task Progress and Logging` made the Development Pack Kate workflow easier to understand, validate, and support while preserving the package-operation safety rules from earlier milestones. The milestone improved live task progress during real `rpm-ostree` work, captured command stdout/stderr for diagnostics, kept compact UI status text readable, corrected Kate source classification, preserved source-aware uninstall behavior, and added persistent structured runtime log events for post-validation support.

The final validated implementation revision for this handoff is `v0.6.2.8`. This document is a maintainer and owner handoff; it does not mark the milestone complete, commit changes, push changes, or advance the workflow beyond handoff review.

## 2. Milestone Goal

The original `v0.6.2` objective was to improve task progress, workflow logging, and diagnostics for the already validated inline Development Pack install/uninstall workflow.

The milestone goal included:

- Improve visible task progress so the sidebar `Tasks (%)` and Kate card communicate real workflow phases instead of coarse jumps.
- Improve workflow logging so package operations leave enough evidence to diagnose failures after the GUI exits.
- Improve diagnostics for command start failures, authorization failures, package-manager failures, warnings, reboot-required states, and refresh outcomes.
- Preserve source-aware install/remove behavior for Kate.
- Preserve safety gates, including the Development Pack-only execution boundary, no arbitrary shell execution, container runtime blocking, and no real Gaming Pack execution.

## 3. What Changed

### Task progress model

The implementation added explicit rpm-ostree-oriented progress phases in `installer.rs` and GUI progress handling in `main.rs`. Progress now has named phases such as starting, preparing, resolving, downloading, applying, finalizing, writing, staging, refreshing, reboot required, completed, and failed.

The GUI maps those phases into concise sidebar text and direction-aware card text:

- Install workflow: `Installing...`, `Waiting for reboot...`, or `Failed`.
- Uninstall workflow: `Removing...`, `Waiting for reboot...`, or `Failed`.
- Sidebar detail: normalized short labels such as `Downloading`, `Writing deployment`, `Refreshing`, or `Reboot required`.

### rpm-ostree stdout/stderr streaming

Command execution now captures child stdout and stderr with pipes instead of relying on inherited terminal I/O. Reader threads drain both streams while the command runs, append captured lines to the final command result, emit progress events when output can be parsed, and write structured `command_output_line` log events.

This is important for KDE app-menu launches where the GUI may not have a usable terminal to inherit and where persistent logs are needed after the process exits.

### Progress parsing and progress clamping

`installer.rs` parses `rpm-ostree` output lines into `CommandProgressEvent` values. Each event carries a source stream, phase, display status, and percent. `main.rs` dispatches those events back to the Slint event loop and clamps live progress to `0..98`.

The UI only reaches `100%` after command completion and post-command state refresh/fallback logic. This prevents the previous failed validation pattern where progress jumped `0 -> 55 -> 100` without representing the real transaction.

### Structured runtime logging

`v0.6.2.8` added persistent structured log events in the GUI runtime log file. The log path is:

- `$XDG_STATE_HOME/ashgrove-welcome/ashgrove-welcome.log`
- `$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log`
- `/tmp/ashgrove-welcome.log` when no home directory is available

The required structured events are:

- `progress_transition`
- `command_output_line`
- `command_result`
- `refresh_result`
- `final_ui_state`

Structured fields are formatted as escaped key/value pairs such as `event="final_ui_state" workflow="install" package="kate" ...`.

### Kate source classification

Kate detection now combines several probes:

- `rpm-ostree status --json`
- `rpm-ostree status`
- `rpm -q kate`
- `flatpak info --system org.kde.kate`
- `flatpak info --user org.kde.kate`
- PATH lookup for `kate`

Flatpak system/user sources are recognized first. Host rpm-ostree layered source is recognized only when the current runtime has `rpm -q kate` and `kate` on PATH and structured rpm-ostree JSON shows Kate in the booted deployment layered/requested package evidence.

### Source unknown / Managed product-rule correction

The Development Pack no longer treats `Managed` as a normal fallback state for active Kate. If Kate is active but source evidence is incomplete or conflicting, the card fails closed as source unknown: installed state remains visible, the checkbox is hidden, and uninstall is disabled.

This prevents the UI from implying that Kate is safely managed or removable when the app cannot prove that.

### Red trash uninstall preservation

The red trash action remains available only for removable sources:

- `HostOstreeLayered`
- `FlatpakSystem`
- `FlatpakUser`

It remains hidden for not installed, pending reboot, base-image, and unknown-source states. This preserves source-aware uninstall behavior and prevents unsupported removal attempts.

### Pack page UI/UX cleanup

The pack page layout was compacted:

- `Development` became `Development Pack`.
- Redundant subtitle and summary card were removed.
- `Install Selected` was reduced in size.
- Placeholder pack pages now follow the `<Name> Pack` title pattern and can hide subtitles.

### Application card compaction

The Kate card was tightened:

- Card height was reduced.
- Checkbox appears only for not-installed items.
- Icon size was reduced.
- Description and metadata are shorter.
- Progress meter is compact and does not display raw package-manager text.
- Status/trash/blocked affordances remain driven by item state, installed state, and removability.

### Documentation updates

The milestone documentation was corrected after review cycles. Stale references to older active states were updated, `v0.6.2.8` was recorded as the current corrective revision, and deferred scope was clarified across architecture, changelog, sprint, project status, milestone, and sprint-plan documents.

## 4. Why It Changed

Validation found that the first implementation did not provide meaningful progress during real `rpm-ostree` work. The visible state moved too coarsely, initially only `0 -> 55 -> 100`, and did not reflect long-running transaction phases.

Inherited terminal I/O was also not sufficient for a production KDE app-menu launch. A GUI process may not inherit a useful terminal, and terminal-only output disappears when the process exits. Capturing stdout/stderr inside the app gives both live progress and persistent evidence.

Long `rpm-ostree` messages were too noisy for card and sidebar UI. The fix keeps raw output in logs and command results while showing normalized, short UI text.

Kate was incorrectly classified as `Managed` or host base-image in cases where the app could not prove that it was a normal OS-owned package or safely removable. That conflicted with the product rule for Development Pack cards: unknown source evidence must fail closed rather than becoming a normal-looking final state.

Earlier JSON handling scanned strings in `rpm-ostree status --json`, which was brittle and missed real deployment evidence. The milestone replaced that with `serde_json` parsing and structured deployment checks.

Before `v0.6.2.8`, persistent structured log evidence was missing for required runtime events. That made CODE REVIEW block the milestone because validation could not rely on durable grep-friendly traces.

Documentation had also become stale after several implementation and review cycles. The docs needed to reflect `v0.6.2.8`, the active corrective workflow state, and the actual deferred work.

## 5. How the Code Works

Command planning and gating live in `crates/forge-welcome-core/src/installer.rs`. `create_install_plan` scopes the Development Pack validation workflow to Kate. `create_confirmed_development_execution_plan` creates a real execution plan only after user confirmation, then applies `ExecutionBoundary::for_confirmed_development_pack`. Execution is allowed only when `ExecutionBoundary.commands_allowed == true`.

Commands are represented as `CommandSpec` with a program and argument vector. They are not executed through arbitrary shell strings.

`execute_execution_plan_with_progress` is the main execution entry point. It skips commands for dry runs, blocks commands when the execution boundary is closed, and otherwise executes each step while sending progress events through a callback.

For `rpm-ostree`, command execution pipes stdout and stderr. Separate readers drain both streams, preserve the captured text, and emit progress/log events per line. Parsed rpm-ostree output becomes `CommandProgressEvent` values, which include a phase, status, percent, and output source.

`main.rs` receives progress callbacks from worker threads. It uses `slint::invoke_from_event_loop` before mutating Slint properties, so UI state updates happen on the Slint event loop. Live progress is clamped below `100%` with `bounded_live_progress_percent`; final `100%` is applied only after command completion, refresh, scheduled-update fallback, and final UI state logging.

Structured runtime logging is handled in `main.rs` through `append_log_event`, `append_structured_log_event`, `format_log_fields`, and `json_log_string`. The milestone logs command output, command result, refresh result, progress transitions, and final UI state without dumping the full environment.

Kate source classification lives in `crates/forge-welcome-core/src/development.rs`. `RpmOstreeStatusJson::parse` uses `serde_json::Value` to read the `deployments` array. Helper methods find booted or staged deployments and check accepted layered/requested package keys, including `requested-packages`, `requested-local-packages`, `base-layered-packages`, and several rpm-ostree key variants.

Classification rules are intentionally fail-closed:

- Flatpak system/user probes win when present.
- Current runtime must prove Kate exists with both `rpm -q kate` and PATH evidence before active host install is trusted.
- Staged deployment with Kate but no active runtime evidence becomes pending rpm-ostree install.
- Booted deployment with Kate and staged deployment without Kate becomes pending rpm-ostree removal.
- Booted layered/requested Kate evidence plus active runtime evidence becomes `HostOstreeLayered`.
- Active Kate without usable structured source evidence becomes `Unknown`.

Application card state in `main.rs` and `PackItemCard.slint` drives checkbox, trash, status, and progress behavior. The checkbox is rendered only when the item is not installed. The red trash is rendered only when actions are enabled, no operation is active, the item is installed, and the source is removable. Unknown-source and managed/non-removable states hide both install selection and uninstall action.

The compact pack layout is rendered in `DevelopmentPage.slint`, `PlaceholderPage.slint`, `PackItemCard.slint`, and `ForgeButton.slint`. The layout uses a smaller page title, a larger usable list area, a compact button, short card text, a smaller icon, and a compact progress bar.

## 6. Files Changed

- `crates/forge-welcome-core/src/installer.rs`: Added task progress phases, command output capture, progress parsing, command result classification, persistent command logging, and execution-boundary-preserving command execution.
- `crates/forge-welcome-core/src/development.rs`: Reworked Kate detection and source classification with structured `serde_json` parsing, fail-closed unknown-source behavior, pending rpm-ostree state handling, and tests.
- `crates/forge-welcome-core/Cargo.toml`: Added `serde_json`.
- `Cargo.lock`: Recorded the new `serde_json` dependency for `forge-welcome-core`.
- `crates/forge-welcome-gui/src/main.rs`: Added transaction gate handling, worker-thread execution, Slint event-loop dispatch, progress clamping, concise UI status normalization, structured runtime logging, final UI state logging, refresh result logging, and source-aware install/remove state application.
- `crates/forge-welcome-gui/ui/app.slint`: Updated default metadata and pack page titles to compact `<Name> Pack` naming.
- `crates/forge-welcome-gui/ui/pages/DevelopmentPage.slint`: Compact Development Pack page title, removed redundant text/card, expanded the list section, shortened Kate description, and reduced `Install Selected`.
- `crates/forge-welcome-gui/ui/pages/PlaceholderPage.slint`: Added optional subtitle rendering and compact placeholder pack layout.
- `crates/forge-welcome-gui/ui/components/PackItemCard.slint`: Compacted card height, icon, text layout, checkbox behavior, progress bar, and source-aware status/trash rendering.
- `crates/forge-welcome-gui/ui/components/ForgeButton.slint`: Added configurable padding and font/icon sizing for compact buttons.
- `manifests/applications.yaml`: Added or updated application catalog data used by the pack/application model.
- `docs/Architecture.md`: Updated active architecture notes for `v0.6.2.8`, structured logging, progress boundary, source-state rules, and deferred scope.
- `docs/CHANGELOG.md`: Recorded `v0.6.2` revision history through `v0.6.2.8`.
- `docs/Current Sprint.md`: Updated active sprint state, corrective revision, and next legal workflow action.
- `docs/Next Sprint Plan v0.6.2.md`: Reframed the plan around the active corrective sequence and deferred scope.
- `docs/Project Status.md`: Updated project state to `v0.6.2.8`, not committed, not pushed, with BUILD AND VALIDATE next.
- `docs/Milestones.md`: Updated milestone history and active `v0.6.2` corrective revision sequence.

## 7. Validation Evidence

### Automated validation

Latest reported automated validation for `v0.6.2.8`:

- `cargo fmt --all --check`: passed
- `cargo check`: passed
- `cargo clippy`: passed
- `cargo test`: passed
- `cargo build -p forge-welcome-gui`: passed
- `git diff --check`: passed

Latest reported test counts:

- `forge-welcome-core`: 111 passed
- `forge-welcome-gui`: 39 passed

### Manual validation

Latest reported manual validation for `v0.6.2.8`:

- Host GUI launched.
- Development Pack opened.
- Compact UI remained intact.
- Kate layered/removable detection worked.
- Red trash was visible when appropriate.
- Checkbox was hidden when Kate was installed.
- Kate install/remove workflow was validated.
- Progress remained below `100%` until completion/refresh.
- Final state reached reboot required when expected.
- Runtime log file contained:
  - `progress_transition`
  - `command_output_line`
  - `command_result`
  - `refresh_result`
  - `final_ui_state`

## 8. Support and Troubleshooting Guide

### Kate shows Source unknown

Run:

```bash
rpm -q kate
command -v kate
rpm-ostree status
rpm-ostree status --json
```

If `rpm -q kate` and `command -v kate` succeed but structured `rpm-ostree status --json` does not show Kate in the booted deployment layered/requested package evidence, the app intentionally fails closed as source unknown. Do not force-enable uninstall without confirming the package source.

### Kate does not show red trash

Check whether the detected source is removable. Red trash appears only for `HostOstreeLayered`, `FlatpakSystem`, or `FlatpakUser`.

Run:

```bash
rpm -q kate
command -v kate
rpm-ostree status --json
```

If Kate is pending reboot, source unknown, base-image, or not installed, trash is intentionally hidden.

### Checkbox appears when Kate is installed

This should not happen after `v0.6.2.8`. Confirm the runtime probes:

```bash
rpm -q kate
command -v kate
rpm-ostree status --json
```

Then inspect the final UI state log:

```bash
grep 'final_ui_state' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
```

Look for `installed`, `selected`, `state`, `status`, and `metadata`.

### Progress appears stuck

Long `rpm-ostree` phases can produce uneven output. Confirm that command output is still being logged:

```bash
grep 'command_output_line' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
grep 'progress_transition' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
```

Also check `rpm-ostree status` in another terminal to see whether a transaction or staged deployment exists.

### Progress jumps to 100 too early

Live progress should clamp below `100%` until command completion and refresh. Check the event order:

```bash
grep -E 'progress_transition|command_result|refresh_result|final_ui_state' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
```

`command_result` and `refresh_result` should appear before final `100%` UI state.

### rpm-ostree auth fails

Check for permission/auth text in command results and stderr:

```bash
grep 'command_result' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
rpm-ostree status
```

Retry from a host session with a working PolicyKit or terminal authentication path. Do not run the workflow from a container.

### Log file is missing

Check the expected locations:

```bash
ls -l "$XDG_STATE_HOME/ashgrove-welcome/ashgrove-welcome.log"
ls -l "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
ls -l /tmp/ashgrove-welcome.log
```

If none exist, confirm the GUI was launched and had permission to create the state directory.

### Expected log events are missing

Check each required event:

```bash
grep 'progress_transition' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
grep 'command_output_line' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
grep 'command_result' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
grep 'refresh_result' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
grep 'final_ui_state' "$HOME/.local/state/ashgrove-welcome/ashgrove-welcome.log"
```

If `command_output_line` is missing, the command may not have started, may have failed before producing output, or the workflow may have been blocked before execution.

### App is accidentally run in a container

Run the host preflight check:

```bash
test -f /run/.containerenv && echo "container - stop" && exit 1 || echo "host - OK"
```

If it reports `container - stop`, close that GUI and launch from the host. Package actions are intentionally blocked in `forge-dev` or container-like environments.

### rpm-ostree status --json format changes

Capture the current output:

```bash
rpm-ostree status --json
```

Compare deployment keys against the accepted package arrays in `development.rs`: `requested-packages`, `requested-local-packages`, `base-layered-packages`, `layered-packages`, `layeredPackages`, and `LayeredPackages`. If rpm-ostree changes its schema, add focused parser tests before changing classification behavior.

## 9. Safety and Risk Review

- `ExecutionBoundary.commands_allowed == true` remains the execution gate.
- No arbitrary shell execution was introduced.
- Command vectors are used instead of shell strings.
- The container runtime guard remains and blocks package actions from container-like environments.
- Real execution remains limited to confirmed Development Pack workflows.
- No real Gaming Pack execution was introduced.
- Unknown or conflicting source evidence fails closed.
- Logging avoids full environment dumps and secrets. It records structured workflow, command, progress, refresh, and UI-state fields needed for support.

## 10. Remaining Risks and Deferred Work

- rpm-ostree progress parsing may still be line-fragment fragile because real command output can vary by rpm-ostree version and transaction path.
- Workflow orchestration remains concentrated in `crates/forge-welcome-gui/src/main.rs`.
- Typed rpm-ostree structs are deferred; the current parser uses `serde_json::Value`.
- A full rpm-ostree D-Bus transaction client is deferred.
- Installation Page / Multi-Pack Installation Queue is deferred.
- Per-application multi-pack progress is deferred.

## 11. Learning Review

### Rust process management

The milestone shows why GUI package workflows need explicit process management. Child processes must be started with known command vectors, captured output, clear result classification, and duration/exit-status reporting.

### Rust stdout/stderr pipe draining

Capturing only one stream can deadlock or lose diagnostics. The implementation drains stdout and stderr concurrently, records both, and treats both as possible sources of progress and support evidence.

### Rust channels and worker-thread communication

Long package work runs off the UI thread. Worker code communicates progress through callbacks and final results through Slint event-loop dispatch. This keeps the GUI responsive while preserving a single UI mutation boundary.

### serde_json and structured parsing

Structured parsing is more maintainable than scanning raw JSON strings. The rpm-ostree parser now looks at actual deployment objects and package arrays instead of relying on brittle text positions.

### Fail-closed state modeling

Unknown package source is not a cosmetic state. It changes allowed actions. The milestone reinforces that destructive package UI must hide unsafe actions unless source evidence is strong enough.

### Slint event-loop dispatch

Slint properties are updated from the event loop, not directly from worker threads. This pattern is essential for stable GUI behavior during background package operations.

### Slint component layout and compact UI design

The compact card work shows the value of shortening source labels, removing redundant summaries, and keeping raw technical output out of cramped UI surfaces.

### rpm-ostree deployment model

rpm-ostree can have booted and staged deployments. Active runtime evidence and staged deployment evidence are different. The UI must distinguish active installed state from pending reboot state.

### rpm-ostree layered packages vs active runtime availability

A deployment mentioning Kate is not enough by itself. The current runtime must also show `rpm -q kate` and `command -v kate` before the app treats Kate as active.

### Linux host vs container validation

Build/test validation can happen in a dev container, but real package workflow validation must happen on the host. The milestone keeps that boundary explicit.

### Persistent runtime logging design

Durable logs need stable event names, escaped fields, and grep-friendly structure. They should capture enough context for support without dumping secrets or full environment state.

### Product-state modeling for package manager UI

Package manager UI states need product rules, not only technical probes. The Development Pack rule is that `Managed` is not an acceptable normal fallback for Kate; source uncertainty must be visible and non-actionable.

## 12. Tech with Dj Content Opportunities

- Building a Linux package workflow UI in Rust and Slint.
- Why command progress is hard in GUI apps.
- Capturing rpm-ostree output safely.
- Source-aware uninstall buttons.
- Debugging package state on Fedora Atomic.
- Designing fail-closed UI states.
- Compacting a Slint application card layout.
- Turning validation failures into better architecture.
- Using `serde_json` to replace brittle command-output scanning.
- Keeping raw logs and friendly UI text separate.
- Designing reboot-required UX for rpm-ostree systems.
- Validating host workflows separately from container builds.

## 13. Handoff Decision

`v0.6.2 — Task Progress and Logging` is ready to proceed to `MILESTONE COMPLETE` if this handoff document is reviewed and accepted by the user.

This document does not mark the milestone complete automatically. It also does not commit, push, or advance to COMMIT / PUSH.
