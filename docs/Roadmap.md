---
modified: 2026-07-01
type: roadmap
project: Forge Welcome
parent_project: Forge OS
status: active
current_version: v0.6.0
current_milestone_series: v0.6.x — Production UI/UX Finalization
current_milestone: v0.6.0 — Production UI/UX Foundation
---

# Roadmap

## Purpose

This document defines the development roadmap for **Forge Welcome**, the desktop welcome and setup application for the Forge OS ecosystem.

The roadmap organizes the project into practical, incremental milestones that move Forge Welcome from a safe dry-run setup interface toward a production-ready first-run setup and onboarding application.

---

# Project Vision

Forge Welcome should become the primary first-run and post-install setup experience for Forge OS.

It should guide users through:

- System introduction
- Development setup
- Gaming setup
- Productivity setup
- Cloud and sync setup
- Forge ecosystem setup
- System information review
- Package profile installation
- Future update and maintenance guidance

Forge Welcome should remain safe by default, clear in behavior, visually polished, and aligned with Forge OS architecture.

---

# Roadmap Principles

## Production UI/UX Before Expansion

v0.6.x now prioritizes production-ready UI/UX before broader pack expansion.

## Inline Install Experience

Installation should happen on the pack page itself instead of relying on sub-dialogs for the main install process.

## Dry-Run Safety First

Real package installation must remain explicit, visible, and boundary-gated.

## Reusable Components

UI patterns should be built as reusable Slint components rather than duplicated per page.

## Documentation-First Development

Every completed milestone should update:

- `Project Status.md`
- `Milestones.md`
- `Current Sprint.md`
- `Development Journal.md`
- `CHANGELOG.md`, if applicable
- `Decisions.md`, when architecture direction changes

---

# Current Roadmap Position

| Area | Status |
|---|---|
| Welcome application foundation | In Progress |
| Pack-aware workflow | Complete |
| Installation progress engine | Complete |
| Transaction result model | Complete |
| Pack refresh engine | Complete |
| Real Development Pack execution | Complete / Guarded |
| Installation workflow stabilization | Complete |
| Production UI/UX finalization | Current |
| Multi-pack UI expansion | Deferred until shared UI foundation is complete |
| Multi-pack real execution | Future |
| Forge OS integration | Future |

---

# Completed Milestones

## v0.5.2 — Pack-Aware Installation Dialog

**Status:** ✅ Complete

Established pack-aware dialog behavior with active pack tracking.

## v0.5.3 — Installation Progress Engine

**Status:** ✅ Complete

Added installation progress state, messages, and current/total tracking.

## v0.5.4 — Transaction Result Model

**Status:** ✅ Complete

Added structured transaction result modeling.

## v0.5.5 — Pack Refresh Engine

**Status:** ✅ Complete

Added reusable pack-aware refresh behavior.

## v0.5.6 — Real Installation Execution Preparation

**Status:** ✅ Complete

Prepared execution mode, execution boundary, command result, permission, failure, and reboot handling.

## v0.5.7 — Real Development Pack Installation

**Status:** ✅ Complete

Implemented guarded real execution for Development Pack only.

## v0.5.8 — Installation Error Handling

**Status:** ✅ Complete

Improved failure classification, warning handling, retry guidance, and recovery text.

## v0.5.9 — Installation Workflow Stabilization

**Status:** ✅ Complete

Stabilized guarded Development Pack installation workflow before UI expansion.

---

# Current Roadmap Series

## v0.6.x — Production UI/UX Finalization

**Status:** Current

### Objective

Make Forge Welcome's install pages production-ready before expanding additional pack functionality.

### Production UI/UX Requirements

- Discover-style installable item cards.
- Reusable `ForgeScrollArea`.
- Inline item selection.
- `Install Selected` action.
- No install sub-dialogs in the final production flow.
- Per-item progress/status.
- Lower-left `Tasks (%)` global progress indicator.
- Persistent workflow logging.
- Reusable pattern across all installable pack pages.

---

## v0.6.0 — Production UI/UX Foundation

**Status:** Current / Ready

### Objective

Create the reusable Slint components and initial Development Pack item-card layout.

### Planned Work

- `ForgeScrollArea.slint`
- `PackItemCard.slint`
- `TaskProgressBar.slint`
- Scrollable Development Pack item list
- Kate-only production-style validation item
- `Install Selected` button label

---

## v0.6.1 — Inline Pack Install Workflow

**Status:** Planned

### Objective

Move the main installation process from a sub-dialog workflow to an inline page workflow.

---

## v0.6.2 — Task Progress and Logging

**Status:** Planned

### Objective

Add global Tasks (%) progress, staged item progress, and persistent log file output.

---

## v0.6.3 — Multi-Pack UI Application

**Status:** Planned

### Objective

Apply the shared UI pattern to Gaming, Productivity, Cloud & Sync, and Forge Ecosystem pages.

---

## v0.6.4 — UI/UX Stabilization and Polish

**Status:** Planned

### Objective

Stabilize visual spacing, scroll behavior, empty states, failure states, accessibility labels, and manual GUI validation.

---

# v0.7.x Roadmap — Forge OS Integration

## v0.7.0 — Forge OS Package Profile Integration

**Status:** Future

Define package profile source format and connect profiles to UI.

## v0.7.1 — rpm-ostree Integration Planning

**Status:** Future

Prepare controlled rpm-ostree workflows.

## v0.7.2 — Flatpak Integration Planning

**Status:** Future

Prepare Flatpak metadata, detection, installation, and refresh workflows.

## v0.7.3 — Distrobox Integration Planning

**Status:** Future

Prepare Distrobox-based developer environment workflows.

---

# v0.8.x Roadmap — First-Run Experience and Settings

- v0.8.0 — First-Run Experience
- v0.8.1 — System Status Dashboard
- v0.8.2 — Settings and Preferences

---

# v0.9.x Roadmap — Pre-Release Stabilization

- v0.9.0 — Feature Freeze
- v0.9.1 — Documentation Pass
- v0.9.2 — Release Candidate Preparation

---

# v1.0.0 — Forge Welcome Stable

Forge Welcome becomes safe, usable, documented, production-ready, and integrated enough to serve as the main setup application for Forge OS.

---

# Recommended Next Prompt

```text
IMPLEMENT PROJECT

Project:
Forge Welcome

Current milestone:
v0.6.x — Production UI/UX Finalization

Current sprint:
v0.6.0 Sprint 1 — Production UI/UX Foundation

Current task:
Implement ForgeScrollArea, PackItemCard, TaskProgressBar, and inline Development Pack item layout.
```
