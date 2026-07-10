# Next Sprint Plan — v0.6.1 Inline Pack Install Workflow

## Sprint

```text
v0.6.1 Sprint 1 — Inline Development Pack Workflow
```

## Objective

Replace the temporary legacy dialog path with inline Development Pack install/uninstall behavior.

---

# First Implementation Task

```text
Wire Install Selected and Kate trash action into inline page state while preserving execution gates.
```

---

# Planned Work

- Add inline confirmation/status area.
- Route selected Kate install through the inline page.
- Ignore installed Kate during install.
- Route Kate trash action through source-aware uninstall.
- Disable uninstall for non-removable sources.
- Update sidebar `Tasks (%)` during workflow.
- Refresh Kate state after workflow completion.
- Log install/uninstall workflow events.

---

# Validation

Run from host:

```bash
cd ~/Documents/AshGrove/ashgrove-welcome

test -f /run/.containerenv && echo "container - stop" || echo "host - OK"

cargo fmt --all
cargo check
cargo clippy
cargo test
cargo build -p forge-welcome-gui
./target/debug/forge-welcome-gui
```

---

# Risks

| Risk | Mitigation |
|---|---|
| Uninstall can modify host state | Require explicit action and execution boundary. |
| Inline workflow may duplicate legacy dialog logic | Extract only after behavior is stable. |
| Source detection edge cases | Keep detection logs and disable uninstall when source is unknown. |
| Accidentally testing from forge-dev | Use host preflight command before every GUI run. |
