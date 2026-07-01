use std::{
    cell::Cell,
    env, fs,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};

use forge_welcome_core::{
    CommandResult, CommandSpec, CommandStatus, DetectionProbeLogEntry, DevelopmentToolStatus,
    ExecutionBoundary, ExecutionMode, ExecutionPlan, ExecutionReport, ExecutionStep,
    ExecutionWorkflowStatus, InstallSource, Pack, create_confirmed_development_execution_plan,
    create_execution_plan, create_execution_plan_with_mode, create_install_plan,
    detect_development_pack_status, execute_execution_plan, load_pack_from_file,
};

slint::include_modules!();

const ITEM_STATE_AVAILABLE: i32 = 0;
const ITEM_STATE_SELECTED: i32 = 1;
const ITEM_STATE_PENDING: i32 = 2;
const ITEM_STATE_INSTALLING: i32 = 3;
const ITEM_STATE_INSTALLED: i32 = 4;
const ITEM_STATE_FAILED: i32 = 5;

const KATE_PACKAGE_NAME: &str = "kate";
const KATE_FLATPAK_APP_ID: &str = "org.kde.kate";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PendingAction {
    InstallDevelopment,
    UninstallKate,
}

#[derive(Debug, Clone, Copy)]
enum PackId {
    Development,
}

impl PackId {
    fn manifest_paths(self) -> &'static [&'static str] {
        match self {
            PackId::Development => &[
                "manifests/developer-pack.yaml",
                "manifests/development-pack.yaml",
            ],
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            PackId::Development => "Development Pack",
        }
    }
}

#[derive(Debug, Clone)]
struct PackRefreshResult {
    pack_id: PackId,
    status_text: String,
    summary_text: String,
    item_installed: bool,
    item_removable: bool,
    item_selected: bool,
    item_install_source: InstallSource,
    item_state: i32,
    item_progress: i32,
    item_status_text: String,
    item_metadata: String,
    item_detection_detail: String,
    item_detection_probes: Vec<DetectionProbeLogEntry>,
}

#[derive(Debug, Clone)]
struct TransactionResult {
    pack_name: String,
    dry_run: bool,
    succeeded: Vec<TransactionItem>,
    failed: Vec<TransactionItem>,
    warnings: Vec<String>,
    reboot_required: bool,
    summary: String,
}

#[derive(Debug, Clone)]
struct TransactionItem {
    name: String,
    detail: String,
}

fn main() -> Result<(), slint::PlatformError> {
    append_log_event("app_start", "AshGrove Welcome GUI starting");

    let app = AppWindow::new()?;

    app.set_last_action_text("Last action: None.".into());
    app.set_active_pack_name(PackId::Development.display_name().into());

    app.set_install_preview_text("Preview Install.".into());
    app.set_install_progress_current(0);
    app.set_install_progress_total(0);
    app.set_install_progress_message("Ready.".into());

    reset_task_progress(&app);
    set_development_item_icon(&app);
    append_log_event("log_path", &format!("{}", log_file_path().display()));
    refresh_pack_status(&app, PackId::Development);

    let pending_action = Rc::new(Cell::new(PendingAction::InstallDevelopment));

    let app_weak = app.as_weak();

    app.on_toggle_development_item_selection(move || {
        if let Some(app) = app_weak.upgrade() {
            if app.get_dev_item_installed() || app.get_dev_item_state() == ITEM_STATE_INSTALLING {
                return;
            }

            let selected = !app.get_dev_item_selected();
            app.set_dev_item_selected(selected);

            if selected {
                app.set_dev_item_state(ITEM_STATE_SELECTED);
                app.set_dev_item_status_text("Selected".into());
                app.set_last_action_text("Last action: Kate selected for installation.".into());
            } else {
                app.set_dev_item_state(ITEM_STATE_AVAILABLE);
                app.set_dev_item_status_text("Available".into());
                app.set_last_action_text(
                    "Last action: Kate removed from the install selection.".into(),
                );
            }
        }
    });

    let app_weak = app.as_weak();
    let pending_action_for_install = Rc::clone(&pending_action);

    app.on_install_development_selected(move || {
        if let Some(app) = app_weak.upgrade() {
            pending_action_for_install.set(PendingAction::InstallDevelopment);

            // Re-read host state before starting the workflow. Kate may have been
            // installed outside the current UI session, and installed state must
            // always override selected state.
            append_log_event(
                "install_selected_clicked",
                "Development Pack Install Selected clicked",
            );
            refresh_pack_status(&app, PackId::Development);

            if app.get_dev_item_installed() {
                app.set_dev_item_selected(false);
                app.set_dev_item_status_text("Installed".into());
                app.set_dev_item_state(ITEM_STATE_INSTALLED);
                app.set_dev_item_progress(100);
                app.set_task_progress_active(false);
                app.set_last_action_text("Last action: Kate is already installed.".into());
                append_log_event(
                    "install_skipped",
                    "Kate is already installed; selected install request ignored",
                );
                return;
            }

            if !app.get_dev_item_selected() {
                app.set_last_action_text("Last action: No Development Pack items selected.".into());
                app.set_task_progress_active(false);
                append_log_event("install_skipped", "No Development Pack items selected");
                return;
            }

            app.set_dev_item_state(ITEM_STATE_PENDING);
            app.set_dev_item_progress(10);
            app.set_dev_item_status_text("Preparing".into());
            set_task_progress(&app, 10, "Preparing Kate");

            // Temporary compatibility path for v0.6.0 Sprint 1:
            // The new inline page is now the visible review surface, while the existing
            // guarded dialog workflow remains available until v0.6.1 wires selected-item
            // execution fully inline.
            let pack_id = PackId::Development;
            let preview = build_install_preview(pack_id);

            app.set_active_pack_name(pack_id.display_name().into());
            app.set_install_preview_text(preview.into());
            app.set_install_progress_current(0);
            app.set_install_progress_total(0);
            app.set_install_progress_message("Ready.".into());
            app.set_install_state(0);
            app.set_show_install_dialog(true);

            app.set_last_action_text(
                "Last action: Install Selected prepared the Kate validation workflow.".into(),
            );
        }
    });

    let app_weak = app.as_weak();
    let pending_action_for_remove = Rc::clone(&pending_action);

    app.on_remove_development_item(move || {
        if let Some(app) = app_weak.upgrade() {
            append_log_event("remove_clicked", "Kate remove action clicked");
            refresh_pack_status(&app, PackId::Development);

            let Some(kate_status) = current_kate_tool_status() else {
                app.set_last_action_text(
                    "Last action: Unable to determine Kate install source.".into(),
                );
                append_log_event("remove_blocked", "Kate status unavailable");
                return;
            };

            if !kate_status.installed {
                app.set_last_action_text("Last action: Kate is not installed.".into());
                append_log_event("remove_blocked", "Kate is not installed");
                refresh_pack_status(&app, PackId::Development);
                return;
            }

            if !kate_status.removable {
                app.set_last_action_text(
                    format!(
                        "Last action: Kate is installed from {}; uninstall is disabled.",
                        kate_status.install_source.label()
                    )
                    .into(),
                );
                append_log_event(
                    "remove_blocked",
                    &format!(
                        "Kate source is not removable: {:?}",
                        kate_status.install_source
                    ),
                );
                refresh_pack_status(&app, PackId::Development);
                return;
            }

            let preview = build_kate_uninstall_confirmation(kate_status.install_source);
            pending_action_for_remove.set(PendingAction::UninstallKate);

            app.set_active_pack_name("Kate Uninstall".into());
            app.set_install_preview_text(preview.into());
            app.set_install_progress_current(0);
            app.set_install_progress_total(0);
            app.set_install_progress_message("Waiting for uninstall confirmation.".into());
            app.set_install_state(2);
            app.set_show_install_dialog(true);
            app.set_dev_item_state(ITEM_STATE_PENDING);
            app.set_dev_item_progress(0);
            app.set_dev_item_status_text("Remove?".into());
            reset_task_progress(&app);
            app.set_last_action_text("Last action: Kate uninstall confirmation opened.".into());
        }
    });

    let app_weak = app.as_weak();
    let pending_action_for_preview = Rc::clone(&pending_action);

    app.on_preview_development_install(move || {
        if let Some(app) = app_weak.upgrade() {
            pending_action_for_preview.set(PendingAction::InstallDevelopment);
            let pack_id = PackId::Development;
            let preview = build_install_preview(pack_id);

            app.set_active_pack_name(pack_id.display_name().into());
            app.set_install_preview_text(preview.into());
            app.set_install_progress_current(0);
            app.set_install_progress_total(0);
            app.set_install_progress_message("Ready.".into());
            app.set_install_state(0);
            app.set_show_install_dialog(true);
        }
    });

    let app_weak = app.as_weak();
    let pending_action_for_close = Rc::clone(&pending_action);

    app.on_close_install_dialog(move || {
        if let Some(app) = app_weak.upgrade() {
            app.set_show_install_dialog(false);

            if pending_action_for_close.get() == PendingAction::UninstallKate {
                pending_action_for_close.set(PendingAction::InstallDevelopment);
                refresh_pack_status(&app, PackId::Development);
                reset_task_progress(&app);
                return;
            }

            if app.get_dev_item_state() == ITEM_STATE_PENDING {
                app.set_dev_item_state(ITEM_STATE_SELECTED);
                app.set_dev_item_progress(0);
                app.set_dev_item_status_text("Selected".into());
                reset_task_progress(&app);
            }
        }
    });

    let app_weak = app.as_weak();
    let pending_action_for_confirm = Rc::clone(&pending_action);

    app.on_confirm_installation(move || {
        if let Some(app) = app_weak.upgrade() {
            let state = app.get_install_state();

            if pending_action_for_confirm.get() == PendingAction::UninstallKate {
                handle_kate_uninstall_confirmation(&app, state);
                if matches!(state, 2 | 4 | 5) {
                    pending_action_for_confirm.set(PendingAction::InstallDevelopment);
                }
                return;
            }

            match state {
                0 => {
                    let pack_id = PackId::Development;

                    app.set_install_state(3);
                    app.set_install_progress_current(1);
                    app.set_install_progress_total(3);
                    app.set_install_progress_message("Preparing dry-run execution.".into());

                    app.set_dev_item_state(ITEM_STATE_INSTALLING);
                    app.set_dev_item_progress(35);
                    app.set_dev_item_status_text("Dry-run".into());
                    set_task_progress(&app, 35, "Dry-run");

                    let result = build_dry_run_result(pack_id);

                    app.set_install_progress_current(3);
                    app.set_install_progress_message("Dry-run completed.".into());
                    app.set_install_preview_text(result.into());
                    app.set_install_state(1);

                    app.set_dev_item_state(ITEM_STATE_SELECTED);
                    app.set_dev_item_progress(0);
                    app.set_dev_item_status_text("Selected".into());
                    set_task_progress(&app, 100, "Dry-run complete");

                    app.set_last_action_text(
                        "Last action: Dry-run completed successfully. No system changes were made."
                            .into(),
                    );

                    refresh_pack_status(&app, pack_id);
                }

                1 => {
                    let pack_id = PackId::Development;
                    let confirmation_preview = build_real_install_confirmation(pack_id);

                    app.set_install_preview_text(confirmation_preview.into());
                    app.set_install_progress_current(0);
                    app.set_install_progress_total(0);
                    app.set_install_progress_message("Waiting for final confirmation.".into());
                    app.set_install_state(2);

                    app.set_dev_item_state(ITEM_STATE_PENDING);
                    app.set_dev_item_progress(0);
                    app.set_dev_item_status_text("Confirm".into());
                    set_task_progress(&app, 0, "Waiting");
                }

                2 => {
                    let pack_id = PackId::Development;

                    app.set_install_state(3);
                    app.set_install_progress_current(1);
                    app.set_install_progress_total(3);
                    app.set_install_progress_message(
                        "Running guarded Development Pack installation.".into(),
                    );

                    app.set_dev_item_state(ITEM_STATE_INSTALLING);
                    app.set_dev_item_progress(50);
                    app.set_dev_item_status_text("Installing".into());
                    set_task_progress(&app, 50, "Installing Kate");

                    let result = build_real_install_result(pack_id);
                    let success = result.failed.is_empty();
                    let rendered_result = render_transaction_result(&result);

                    app.set_install_progress_current(3);
                    app.set_install_progress_message("Installation workflow completed.".into());
                    app.set_install_preview_text(rendered_result.into());
                    app.set_install_state(if success { 4 } else { 5 });

                    if success {
                        app.set_dev_item_selected(false);
                        app.set_dev_item_installed(true);
                        app.set_dev_item_state(ITEM_STATE_INSTALLED);
                        app.set_dev_item_progress(100);
                        app.set_dev_item_status_text("Installed".into());
                        set_task_progress(&app, 100, "Complete");

                        app.set_last_action_text(
                            "Last action: Development Pack installation completed.".into(),
                        );
                    } else {
                        app.set_dev_item_state(ITEM_STATE_FAILED);
                        app.set_dev_item_progress(100);
                        app.set_dev_item_status_text("Failed".into());
                        set_task_progress(&app, 100, "Failed");

                        app.set_last_action_text(
                            "Last action: Development Pack installation completed with failures."
                                .into(),
                        );
                    }

                    refresh_pack_status(&app, pack_id);
                }

                _ => {
                    app.set_show_install_dialog(false);
                }
            }
        }
    });

    app.run()
}

fn log_file_path() -> PathBuf {
    if let Some(xdg_state_home) = env::var_os("XDG_STATE_HOME") {
        return PathBuf::from(xdg_state_home)
            .join("ashgrove-welcome")
            .join("ashgrove-welcome.log");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("ashgrove-welcome")
            .join("ashgrove-welcome.log");
    }

    PathBuf::from("/tmp/ashgrove-welcome.log")
}

fn append_log_event(event: &str, detail: &str) {
    let path = log_file_path();

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            eprintln!(
                "AshGrove Welcome: unable to create log directory {}: {error}",
                parent.display()
            );
            return;
        }
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();

    let line = format!("{timestamp} [{event}] {detail}\n");

    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(mut file) => {
            if let Err(error) = file.write_all(line.as_bytes()) {
                eprintln!(
                    "AshGrove Welcome: unable to write log file {}: {error}",
                    path.display()
                );
            }
        }
        Err(error) => {
            eprintln!(
                "AshGrove Welcome: unable to open log file {}: {error}",
                path.display()
            );
        }
    }
}

fn set_development_item_icon(app: &AppWindow) {
    let icon_names = kde_kate_icon_names();

    if let Some(icon_path) = find_application_icon(&icon_names) {
        match slint::Image::load_from_path(&icon_path) {
            Ok(image) => {
                app.set_dev_item_icon_image(image);
                app.set_dev_item_has_icon_image(true);
                append_log_event(
                    "icon_loaded",
                    &format!("Kate icon loaded from {}", icon_path.display()),
                );
            }
            Err(error) => {
                eprintln!(
                    "Forge Welcome: unable to load Kate icon from {}: {error}",
                    icon_path.display()
                );
                app.set_dev_item_has_icon_image(false);
            }
        }
    } else {
        app.set_dev_item_has_icon_image(false);
        append_log_event("icon_missing", "Kate icon not found in system icon folders");
    }
}

fn kde_kate_icon_names() -> Vec<String> {
    let mut icon_names = Vec::new();

    for app_id in ["org.kde.kate", "kate"] {
        push_unique(&mut icon_names, app_id.to_string());
    }

    for desktop_file in ["org.kde.kate.desktop", "kate.desktop"] {
        if let Some(icon_name) = read_icon_name_from_desktop_file(desktop_file) {
            push_unique(&mut icon_names, icon_name);
        }
    }

    icon_names
}

fn read_icon_name_from_desktop_file(desktop_file_name: &str) -> Option<String> {
    for data_root in icon_data_roots() {
        let path = data_root.join("applications").join(desktop_file_name);

        let Ok(contents) = fs::read_to_string(&path) else {
            continue;
        };

        for line in contents.lines() {
            let line = line.trim();

            if let Some(value) = line.strip_prefix("Icon=") {
                let icon_name = value.trim();

                if !icon_name.is_empty() {
                    return Some(icon_name.to_string());
                }
            }
        }
    }

    None
}

fn find_application_icon(icon_names: &[String]) -> Option<PathBuf> {
    let data_roots = icon_data_roots();
    let themes = ["hicolor", "breeze", "breeze-dark", "Adwaita"];
    let sizes = [
        "scalable", "symbolic", "512x512", "256x256", "128x128", "64x64", "48x48", "32x32",
        "24x24", "22x22", "16x16",
    ];
    let extensions = ["svg", "png", "xpm"];

    for root in &data_roots {
        for theme in themes {
            for size in sizes {
                for icon_name in icon_names {
                    for extension in extensions {
                        let candidate = root
                            .join("icons")
                            .join(theme)
                            .join(size)
                            .join("apps")
                            .join(format!("{icon_name}.{extension}"));

                        if candidate.is_file() {
                            return Some(candidate);
                        }
                    }
                }
            }
        }
    }

    for root in &data_roots {
        for icon_name in icon_names {
            for extension in extensions {
                let candidate = root
                    .join("pixmaps")
                    .join(format!("{icon_name}.{extension}"));

                if candidate.is_file() {
                    return Some(candidate);
                }
            }
        }
    }

    None
}

fn icon_data_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Some(xdg_data_dirs) = env::var_os("XDG_DATA_DIRS") {
        for path in env::split_paths(&xdg_data_dirs) {
            push_unique_path(&mut roots, path);
        }
    }

    for path in [
        "/usr/local/share",
        "/usr/share",
        "/var/lib/flatpak/exports/share",
    ] {
        push_unique_path(&mut roots, PathBuf::from(path));
    }

    if let Some(home) = env::var_os("HOME") {
        let home = PathBuf::from(home);
        push_unique_path(&mut roots, home.join(".local/share"));
        push_unique_path(&mut roots, home.join(".local/share/flatpak/exports/share"));
    }

    roots.into_iter().filter(|path| path.is_dir()).collect()
}

fn push_unique(values: &mut Vec<String>, value: String) {
    if !values.iter().any(|existing| existing == &value) {
        values.push(value);
    }
}

fn push_unique_path(paths: &mut Vec<PathBuf>, path: PathBuf) {
    if !paths.iter().any(|existing| existing == &path) {
        paths.push(path);
    }
}

fn reset_task_progress(app: &AppWindow) {
    app.set_task_progress_active(false);
    app.set_task_progress_percent(0);
    app.set_task_progress_title("Tasks (0%)".into());
    app.set_task_progress_detail("Idle".into());
}

fn set_task_progress(app: &AppWindow, percent: i32, detail: &str) {
    app.set_task_progress_active(true);
    app.set_task_progress_percent(percent);
    app.set_task_progress_title(format!("Tasks ({percent}%)").into());
    app.set_task_progress_detail(detail.into());
}

fn refresh_pack_status(app: &AppWindow, pack_id: PackId) {
    let refresh_result = build_pack_refresh_result(pack_id, app.get_dev_item_selected());
    apply_pack_refresh_result(app, &refresh_result);
}

fn build_pack_refresh_result(pack_id: PackId, current_selection: bool) -> PackRefreshResult {
    match pack_id {
        PackId::Development => build_development_pack_refresh_result(current_selection),
    }
}

fn build_development_pack_refresh_result(current_selection: bool) -> PackRefreshResult {
    let status = detect_development_pack_status();

    let available_count = status.tools.iter().filter(|tool| tool.installed).count();
    let total_count = status.tools.len();

    let status_text = status
        .tools
        .iter()
        .map(|tool| {
            let marker = if tool.installed { "✓" } else { "✗" };
            let version = tool
                .version
                .clone()
                .unwrap_or_else(|| "Missing".to_string());

            format!("{marker} {} — {}", tool.name, version)
        })
        .collect::<Vec<_>>()
        .join("\n");

    let summary_text = format!("Development tools available: {available_count}/{total_count}");

    let kate_status = status
        .tools
        .iter()
        .find(|tool| tool.name.eq_ignore_ascii_case("Kate"))
        .or_else(|| status.tools.first());

    let item_installed = kate_status.map(|tool| tool.installed).unwrap_or(false);
    let item_removable = kate_status.map(|tool| tool.removable).unwrap_or(false);
    let item_install_source = kate_status
        .map(|tool| tool.install_source)
        .unwrap_or(InstallSource::NotInstalled);
    let item_selected = if item_installed {
        false
    } else {
        current_selection
    };
    let item_state = if item_installed {
        ITEM_STATE_INSTALLED
    } else if item_selected {
        ITEM_STATE_SELECTED
    } else {
        ITEM_STATE_AVAILABLE
    };
    let item_progress = if item_installed { 100 } else { 0 };
    let item_status_text = if item_installed && item_removable {
        "Installed".to_string()
    } else if item_installed {
        "Managed".to_string()
    } else if item_selected {
        "Selected".to_string()
    } else {
        "Available".to_string()
    };

    let item_metadata = if item_installed {
        kate_status
            .and_then(|tool| tool.version.clone())
            .unwrap_or_else(|| "Installed".to_string())
    } else {
        "Host application · kate · Development Pack validation item".to_string()
    };

    let item_detection_detail = kate_status
        .map(|tool| tool.detection_detail.clone())
        .unwrap_or_else(|| "Kate status unavailable".to_string());

    let item_detection_probes = kate_status
        .map(|tool| tool.detection_probes.clone())
        .unwrap_or_default();

    PackRefreshResult {
        pack_id: PackId::Development,
        status_text,
        summary_text,
        item_installed,
        item_removable,
        item_selected,
        item_install_source,
        item_state,
        item_progress,
        item_status_text,
        item_metadata,
        item_detection_detail,
        item_detection_probes,
    }
}

fn apply_pack_refresh_result(app: &AppWindow, refresh_result: &PackRefreshResult) {
    match refresh_result.pack_id {
        PackId::Development => {
            app.set_dev_status_text(refresh_result.status_text.clone().into());
            app.set_dev_summary_text(refresh_result.summary_text.clone().into());
            app.set_dev_item_installed(refresh_result.item_installed);
            app.set_dev_item_removable(refresh_result.item_removable);
            app.set_dev_item_selected(refresh_result.item_selected);
            app.set_dev_item_state(refresh_result.item_state);
            app.set_dev_item_progress(refresh_result.item_progress);
            app.set_dev_item_status_text(refresh_result.item_status_text.clone().into());
            app.set_dev_item_metadata(refresh_result.item_metadata.clone().into());

            log_pack_item_detection_plan(PackId::Development, "Kate");

            for probe in &refresh_result.item_detection_probes {
                append_log_event(
                    "pack_item_detection_probe",
                    &format!(
                        "pack={} item=Kate step={} probe={} command='{}' found={} success={} stdout='{}' stderr='{}'",
                        refresh_result.pack_id.display_name(),
                        probe.step,
                        probe.probe_name,
                        probe.command_line,
                        probe.command_found,
                        probe.success,
                        probe.stdout_hint,
                        probe.stderr_hint
                    ),
                );
            }

            append_log_event(
                "pack_item_detection_decision",
                &format!(
                    "pack={} item=Kate installed={} selected={} state={} source={:?} removable={} metadata={} decision_detail={}",
                    refresh_result.pack_id.display_name(),
                    refresh_result.item_installed,
                    refresh_result.item_selected,
                    refresh_result.item_state,
                    refresh_result.item_install_source,
                    refresh_result.item_removable,
                    refresh_result.item_metadata,
                    refresh_result.item_detection_detail
                ),
            );

            append_log_event(
                "development_refresh",
                &format!(
                    "installed={} selected={} state={} source={:?} removable={} metadata={} summary={} detection={}",
                    refresh_result.item_installed,
                    refresh_result.item_selected,
                    refresh_result.item_state,
                    refresh_result.item_install_source,
                    refresh_result.item_removable,
                    refresh_result.item_metadata,
                    refresh_result.summary_text,
                    refresh_result.item_detection_detail
                ),
            );
        }
    }
}

fn log_pack_item_detection_plan(pack_id: PackId, item_name: &str) {
    append_log_event(
        "pack_item_detection_plan",
        &format!(
            "pack={} item={} order=1:'rpm-ostree status --json' order=2:'rpm-ostree status' order=3:'rpm -q kate' order=4:'flatpak info --system org.kde.kate' order=5:'flatpak info --user org.kde.kate' order=6:'NotInstalled'",
            pack_id.display_name(),
            item_name
        ),
    );
}

fn build_install_preview(pack_id: PackId) -> String {
    let pack_result = load_first_available_pack(pack_id);

    let Ok(pack) = pack_result else {
        return format!("Unable to load {} manifest.", pack_id.display_name());
    };

    let install_plan = create_install_plan(&pack);
    let execution_plan = create_execution_plan(&install_plan, true);

    if execution_plan.steps.is_empty() {
        return format!("No install steps required for {}.", pack_id.display_name());
    }

    let mut output = format!(
        "Pack: {} ({})\nMode: {}\nBoundary allows commands: {}\n\n",
        execution_plan.pack_name,
        execution_plan.pack_id,
        execution_plan.execution_mode.label(),
        execution_plan.command_boundary.commands_allowed
    );

    output.push_str("Dry-run preview only. No system changes will be made.\n\n");

    for step in execution_plan.steps {
        output.push_str(&format!("{}\n{}\n\n", step.description, step.command));
    }

    output.push_str(&format!(
        "Requires reboot: {}",
        execution_plan.requires_reboot
    ));

    output
}

fn build_real_install_confirmation(pack_id: PackId) -> String {
    let pack_result = load_first_available_pack(pack_id);

    let Ok(pack) = pack_result else {
        return format!("Unable to load {} manifest.", pack_id.display_name());
    };

    let install_plan = create_install_plan(&pack);
    let execution_plan =
        create_execution_plan_with_mode(&install_plan, ExecutionMode::RealExecution);

    if execution_plan.steps.is_empty() {
        return format!("No install steps required for {}.", pack_id.display_name());
    }

    let mut output = format!(
        "Confirm Real Installation\n\nPack: {} ({})\nMode requested: {}\nBoundary currently allows commands: {}\n\n",
        execution_plan.pack_name,
        execution_plan.pack_id,
        execution_plan.execution_mode.label(),
        execution_plan.command_boundary.commands_allowed
    );

    output.push_str("Safety rule:\n");
    output.push_str("ExecutionMode is intent. ExecutionBoundary is permission.\n\n");
    output.push_str(
        "Press Install only if you want Forge Welcome to enable the guarded Development Pack execution boundary and run the listed controlled commands.\n\n",
    );

    for step in execution_plan.steps {
        output.push_str(&format!("{}\n{}\n\n", step.description, step.command));
    }

    output.push_str(&format!(
        "Requires reboot: {}\n\nThis action may modify the system.",
        execution_plan.requires_reboot
    ));

    output
}

fn build_dry_run_result(pack_id: PackId) -> String {
    let result = build_transaction_result(pack_id, true);
    render_transaction_result(&result)
}

fn build_real_install_result(pack_id: PackId) -> TransactionResult {
    match pack_id {
        PackId::Development => build_development_real_install_result(),
    }
}

fn build_transaction_result(pack_id: PackId, dry_run: bool) -> TransactionResult {
    match pack_id {
        PackId::Development => build_development_transaction_result(dry_run),
    }
}

fn build_development_transaction_result(dry_run: bool) -> TransactionResult {
    let status = detect_development_pack_status();

    let mut succeeded = Vec::new();
    let mut failed = Vec::new();
    let mut warnings = Vec::new();

    for tool in status.tools {
        if tool.installed {
            let detail = tool.version.unwrap_or_else(|| "Installed".to_string());

            succeeded.push(TransactionItem {
                name: tool.name,
                detail,
            });
        } else {
            failed.push(TransactionItem {
                name: tool.name.clone(),
                detail: "Missing".to_string(),
            });

            warnings.push(format!("{} is missing.", tool.name));
        }
    }

    let total = succeeded.len() + failed.len();

    TransactionResult {
        pack_name: PackId::Development.display_name().to_string(),
        dry_run,
        reboot_required: false,
        summary: format!(
            "{} of {} development components are currently available.",
            succeeded.len(),
            total
        ),
        succeeded,
        failed,
        warnings,
    }
}

fn build_development_real_install_result() -> TransactionResult {
    let pack_result = load_first_available_pack(PackId::Development);

    let Ok(pack) = pack_result else {
        return TransactionResult {
            pack_name: PackId::Development.display_name().to_string(),
            dry_run: false,
            reboot_required: false,
            summary: "Unable to load Development Pack manifest. Installation was not started."
                .to_string(),
            succeeded: Vec::new(),
            failed: vec![TransactionItem {
                name: "Development Pack manifest".to_string(),
                detail: "Manifest could not be loaded.".to_string(),
            }],
            warnings: vec!["Real installation was blocked before command execution.".to_string()],
        };
    };

    let install_plan = create_install_plan(&pack);
    let execution_plan = create_confirmed_development_execution_plan(&install_plan, true);
    let execution_report = execute_execution_plan(&execution_plan);

    transaction_result_from_execution_report(execution_report)
}

fn transaction_result_from_execution_report(report: ExecutionReport) -> TransactionResult {
    let mut transaction_result = TransactionResult {
        pack_name: report.pack_name.clone(),
        dry_run: report.dry_run,
        reboot_required: report.requires_reboot,
        summary: build_transaction_summary(&report),
        succeeded: Vec::new(),
        failed: Vec::new(),
        warnings: Vec::new(),
    };

    for command_result in &report.results {
        add_command_result_to_transaction_result(command_result, &mut transaction_result);
    }

    append_workflow_notes(&report, &mut transaction_result);

    transaction_result
}

fn build_transaction_summary(report: &ExecutionReport) -> String {
    report.summary_line()
}

fn add_command_result_to_transaction_result(
    command_result: &CommandResult,
    transaction_result: &mut TransactionResult,
) {
    let item = TransactionItem {
        name: command_result.description.clone(),
        detail: build_command_result_detail(command_result),
    };

    match command_result.status {
        CommandStatus::Succeeded => transaction_result.succeeded.push(item),
        CommandStatus::Failed | CommandStatus::Blocked => transaction_result.failed.push(item),
        CommandStatus::Skipped | CommandStatus::Planned => {
            transaction_result.warnings.push(format!(
                "{}: {}",
                command_result.description,
                command_result
                    .message
                    .clone()
                    .unwrap_or_else(|| command_result.status.label().to_string())
            ));
        }
    }

    if command_result.has_warning_classification() {
        transaction_result.warnings.push(format!(
            "{}: {}",
            command_result.description,
            command_result.guidance_summary()
        ));
    }
}

fn append_workflow_notes(report: &ExecutionReport, transaction_result: &mut TransactionResult) {
    match report.workflow_status() {
        ExecutionWorkflowStatus::PartialSuccess => transaction_result.warnings.push(
            "Partial success detected. Some commands completed, but one or more commands failed. Review failed items before retrying."
                .to_string(),
        ),
        ExecutionWorkflowStatus::Blocked => transaction_result.failed.push(TransactionItem {
            name: "Execution boundary".to_string(),
            detail: report.command_boundary.safety_note.clone(),
        }),
        ExecutionWorkflowStatus::SucceededWithWarnings
        | ExecutionWorkflowStatus::Succeeded
        | ExecutionWorkflowStatus::Planned
        | ExecutionWorkflowStatus::DryRun
        | ExecutionWorkflowStatus::Failed => {}
    }

    if report.command_boundary.commands_allowed {
        transaction_result
            .warnings
            .push(report.command_boundary.safety_note.clone());
    }

    if report.requires_reboot {
        transaction_result.warnings.push(
            "A reboot is required before all Development Pack changes should be considered active."
                .to_string(),
        );
    }
}

fn build_command_result_detail(command_result: &CommandResult) -> String {
    let mut detail = String::new();

    detail.push_str(&format!("Status: {}\n", command_result.status.label()));
    detail.push_str(&format!(
        "Classification: {}\n",
        command_result.classification.kind.label()
    ));
    detail.push_str(&format!("Command: {}\n", command_result.command));
    detail.push_str(&format!(
        "Explanation: {}\n",
        &command_result.classification.explanation
    ));
    detail.push_str(&format!(
        "Guidance: {}\n",
        &command_result.classification.guidance
    ));
    detail.push_str(&format!(
        "Retry safe: {}\n",
        if command_result.classification.retry_safe {
            "yes"
        } else {
            "no"
        }
    ));
    detail.push_str(&format!(
        "User action required: {}\n",
        if command_result.classification.user_action_required {
            "yes"
        } else {
            "no"
        }
    ));

    if let Some(exit_code) = command_result.exit_code {
        detail.push_str(&format!("Exit code: {exit_code}\n"));
    }

    if let Some(duration_ms) = command_result.duration_ms {
        detail.push_str(&format!("Duration: {duration_ms} ms\n"));
    }

    if let Some(message) = &command_result.message {
        detail.push_str(&format!("Message: {message}\n"));
    }

    if !command_result.stdout.trim().is_empty() {
        detail.push_str("Stdout:\n");
        detail.push_str(command_result.stdout.trim());
        detail.push('\n');
    }

    if !command_result.stderr.trim().is_empty() {
        detail.push_str("Stderr:\n");
        detail.push_str(command_result.stderr.trim());
        detail.push('\n');
    }

    detail
}

fn render_transaction_result(result: &TransactionResult) -> String {
    let mut output = String::new();

    output.push_str(&format!("{} Transaction Results\n\n", result.pack_name));
    let is_uninstall = result.pack_name.to_lowercase().contains("uninstall");

    if result.dry_run {
        output.push_str("Mode: Dry Run\n");
        output.push_str("No system changes were made.\n\n");
    } else {
        if is_uninstall {
            output.push_str("Mode: Real Uninstall\n\n");
        } else {
            output.push_str("Mode: Real Installation\n\n");
        }
    }

    output.push_str("Summary\n");
    output.push_str("────────────────────────────\n\n");
    output.push_str(&result.summary);
    output.push_str("\n\n");

    output.push_str(&format!(
        "Succeeded: {}\nFailed: {}\nWarnings: {}\nRequires reboot: {}\n\n",
        result.succeeded.len(),
        result.failed.len(),
        result.warnings.len(),
        result.reboot_required
    ));

    output.push_str("Succeeded\n");
    output.push_str("────────────────────────────\n\n");

    if result.succeeded.is_empty() {
        output.push_str("No successful items reported.\n\n");
    } else {
        for item in &result.succeeded {
            output.push_str(&format!("✓ {}\n  {}\n\n", item.name, item.detail));
        }
    }

    output.push_str("Failed / Missing\n");
    output.push_str("────────────────────────────\n\n");

    if result.failed.is_empty() {
        output.push_str("No failed or missing items reported.\n\n");
    } else {
        for item in &result.failed {
            output.push_str(&format!("⚠ {}\n  {}\n\n", item.name, item.detail));
        }
    }

    output.push_str("Warnings\n");
    output.push_str("────────────────────────────\n\n");

    if result.warnings.is_empty() {
        output.push_str("No warnings reported.\n\n");
    } else {
        for warning in &result.warnings {
            output.push_str(&format!("• {}\n", warning));
        }

        output.push('\n');
    }

    output.push_str("Result\n");
    output.push_str("────────────────────────────\n\n");

    if result.dry_run {
        output.push_str("Dry-run completed successfully.\n");
        output.push_str("Installation execution was not performed.");
    } else if result.failed.is_empty() && result.reboot_required && is_uninstall {
        output.push_str("Uninstall transaction completed successfully. Reboot is required before all changes are fully applied.");
    } else if result.failed.is_empty() && result.reboot_required {
        output.push_str("Installation transaction completed successfully. Reboot is required before all changes are active.");
    } else if result.failed.is_empty() && is_uninstall {
        output.push_str("Uninstall transaction completed successfully.");
    } else if result.failed.is_empty() {
        output.push_str("Installation transaction completed successfully.");
    } else if is_uninstall {
        output.push_str("Uninstall transaction completed with failures. Review the guidance for each failed item before retrying.");
    } else {
        output.push_str("Installation transaction completed with failures. Review the guidance for each failed item before retrying.");
    }

    output
}

fn current_kate_tool_status() -> Option<DevelopmentToolStatus> {
    detect_development_pack_status()
        .tools
        .into_iter()
        .find(|tool| tool.name.eq_ignore_ascii_case("Kate"))
}

fn build_kate_uninstall_confirmation(source: InstallSource) -> String {
    let Some((description, command_spec, requires_reboot)) = kate_uninstall_command(source) else {
        return format!(
            "Kate is installed from {}.\n\nThis source is not removable through AshGrove Welcome yet.",
            source.label()
        );
    };

    let mut output = String::new();
    output.push_str("Confirm Kate Uninstall\n\n");
    output.push_str(&format!("Detected source: {}\n", source.label()));
    output.push_str(&format!("Action: {}\n", description));
    output.push_str(&format!("Command: {}\n", command_spec.display_command()));
    output.push_str(&format!("Requires reboot: {}\n\n", requires_reboot));
    output.push_str("Safety rule:\n");
    output.push_str("ExecutionMode is intent. ExecutionBoundary is permission.\n\n");
    output.push_str(
        "Press Install in this temporary confirmation dialog only if you want AshGrove Welcome to run the guarded uninstall command for Kate.\n",
    );

    output
}

fn handle_kate_uninstall_confirmation(app: &AppWindow, state: i32) {
    match state {
        2 => {
            app.set_install_state(3);
            app.set_install_progress_current(1);
            app.set_install_progress_total(3);
            app.set_install_progress_message("Running guarded Kate uninstall.".into());
            app.set_dev_item_state(ITEM_STATE_INSTALLING);
            app.set_dev_item_progress(50);
            app.set_dev_item_status_text("Removing".into());
            set_task_progress(app, 50, "Removing Kate");

            let result = build_kate_real_uninstall_result();
            let success = result.failed.is_empty();
            let rendered_result = render_transaction_result(&result);

            app.set_install_progress_current(3);
            app.set_install_progress_message("Kate uninstall workflow completed.".into());
            app.set_install_preview_text(rendered_result.into());
            app.set_install_state(if success { 4 } else { 5 });
            app.set_dev_item_progress(100);

            if success {
                set_task_progress(app, 100, "Uninstall complete");
                app.set_last_action_text("Last action: Kate uninstall command completed.".into());
                append_log_event("uninstall_completed", "Kate uninstall command completed");
            } else {
                app.set_dev_item_state(ITEM_STATE_FAILED);
                app.set_dev_item_status_text("Failed".into());
                set_task_progress(app, 100, "Uninstall failed");
                app.set_last_action_text("Last action: Kate uninstall command failed.".into());
                append_log_event("uninstall_failed", "Kate uninstall command failed");
            }

            refresh_pack_status(app, PackId::Development);
        }
        _ => {
            app.set_show_install_dialog(false);
            refresh_pack_status(app, PackId::Development);
            reset_task_progress(app);
        }
    }
}

fn build_kate_real_uninstall_result() -> TransactionResult {
    let Some(kate_status) = current_kate_tool_status() else {
        return failed_uninstall_result("Kate status", "Unable to determine Kate install source.");
    };

    if !kate_status.installed {
        return failed_uninstall_result("Kate", "Kate is not installed.");
    }

    let Some(execution_plan) =
        build_kate_uninstall_execution_plan(kate_status.install_source, true)
    else {
        return failed_uninstall_result(
            "Kate uninstall source",
            &format!(
                "Kate is installed from {}; uninstall is disabled for this source.",
                kate_status.install_source.label()
            ),
        );
    };

    append_log_event(
        "uninstall_started",
        &format!(
            "source={:?} command_count={}",
            kate_status.install_source,
            execution_plan.steps.len()
        ),
    );

    let execution_report = execute_execution_plan(&execution_plan);
    transaction_result_from_execution_report(execution_report)
}

fn failed_uninstall_result(name: &str, detail: &str) -> TransactionResult {
    TransactionResult {
        pack_name: "Kate Uninstall".to_string(),
        dry_run: false,
        reboot_required: false,
        summary: detail.to_string(),
        succeeded: Vec::new(),
        failed: vec![TransactionItem {
            name: name.to_string(),
            detail: detail.to_string(),
        }],
        warnings: Vec::new(),
    }
}

fn build_kate_uninstall_execution_plan(
    source: InstallSource,
    user_confirmed: bool,
) -> Option<ExecutionPlan> {
    let (description, command_spec, requires_reboot) = kate_uninstall_command(source)?;
    let command = command_spec.display_command();

    Some(ExecutionPlan {
        pack_id: "development".to_string(),
        pack_name: "Kate Uninstall".to_string(),
        dry_run: false,
        execution_mode: ExecutionMode::RealExecution,
        command_boundary: ExecutionBoundary::for_confirmed_development_pack(
            "development",
            user_confirmed,
        ),
        steps: vec![ExecutionStep {
            description,
            command,
            command_spec,
        }],
        requires_reboot,
    })
}

fn kate_uninstall_command(source: InstallSource) -> Option<(String, CommandSpec, bool)> {
    match source {
        InstallSource::HostOstreeLayered => Some((
            "Uninstall Kate host layered package".to_string(),
            CommandSpec::new("rpm-ostree", ["uninstall", KATE_PACKAGE_NAME]),
            true,
        )),
        InstallSource::FlatpakSystem => Some((
            "Uninstall Kate system Flatpak".to_string(),
            CommandSpec::new(
                "flatpak",
                ["uninstall", "--system", "-y", KATE_FLATPAK_APP_ID],
            ),
            false,
        )),
        InstallSource::FlatpakUser => Some((
            "Uninstall Kate user Flatpak".to_string(),
            CommandSpec::new(
                "flatpak",
                ["uninstall", "--user", "-y", KATE_FLATPAK_APP_ID],
            ),
            false,
        )),
        InstallSource::HostBaseImage | InstallSource::Unknown | InstallSource::NotInstalled => None,
    }
}

fn load_first_available_pack(pack_id: PackId) -> Result<Pack, String> {
    for path in pack_id.manifest_paths() {
        if let Ok(pack) = load_pack_from_file(path) {
            return Ok(pack);
        }
    }

    Err(format!(
        "Unable to load {} manifest.",
        pack_id.display_name()
    ))
}
