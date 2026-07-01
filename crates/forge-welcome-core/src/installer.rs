use crate::Pack;
use std::env;
use std::fs;
use std::process::{Command, Stdio};
use std::time::Instant;

const DEVELOPMENT_VALIDATION_PACKAGE: &str = "kate";

#[derive(Debug, Clone)]
pub struct InstallPlan {
    pub pack_id: String,
    pub pack_name: String,
    pub host_packages: Vec<String>,
    pub flatpaks: Vec<String>,
    pub distrobox_packages: Vec<String>,
    pub requires_reboot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeInstallEnvironment {
    Host,
    Distrobox,
    Unknown,
}

impl RuntimeInstallEnvironment {
    pub fn label(self) -> &'static str {
        match self {
            Self::Host => "Host",
            Self::Distrobox => "Distrobox / forge-dev",
            Self::Unknown => "Unknown",
        }
    }

    pub fn is_container_like(self) -> bool {
        matches!(self, Self::Distrobox)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageInstallStrategy {
    RpmOstree,
    Flatpak,
    Dnf,
}

impl PackageInstallStrategy {
    pub fn label(self) -> &'static str {
        match self {
            Self::RpmOstree => "rpm-ostree",
            Self::Flatpak => "Flatpak",
            Self::Dnf => "DNF",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrivilegeEscalationMethod {
    Direct,
    Pkexec,
    SudoAskpass,
    SudoInteractive,
}

impl PrivilegeEscalationMethod {
    pub fn label(self) -> &'static str {
        match self {
            Self::Direct => "direct root dnf",
            Self::Pkexec => "PolicyKit authentication prompt",
            Self::SudoAskpass => "sudo askpass",
            Self::SudoInteractive => "terminal sudo prompt",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallTarget {
    SystemPackage,
    FlatpakApplication,
    ContainerPackage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    DryRun,
    RealExecution,
}

impl ExecutionMode {
    pub fn from_dry_run(dry_run: bool) -> Self {
        if dry_run {
            Self::DryRun
        } else {
            Self::RealExecution
        }
    }

    pub fn is_dry_run(self) -> bool {
        matches!(self, Self::DryRun)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::DryRun => "Dry Run",
            Self::RealExecution => "Real Execution",
        }
    }
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::DryRun
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionBoundary {
    pub mode: ExecutionMode,
    pub commands_allowed: bool,
    pub safety_note: String,
}

impl ExecutionBoundary {
    pub fn for_mode(mode: ExecutionMode) -> Self {
        match mode {
            ExecutionMode::DryRun => Self {
                mode,
                commands_allowed: false,
                safety_note: "Dry-run mode only previews commands. No system commands are executed."
                    .to_string(),
            },
            ExecutionMode::RealExecution => Self {
                mode,
                commands_allowed: false,
                safety_note: "Real execution mode requires explicit confirmation and an enabled execution boundary."
                    .to_string(),
            },
        }
    }

    pub fn for_confirmed_development_pack(pack_id: &str, user_confirmed: bool) -> Self {
        let commands_allowed = user_confirmed && is_development_pack_id(pack_id);

        if commands_allowed {
            Self {
                mode: ExecutionMode::RealExecution,
                commands_allowed: true,
                safety_note:
                    "Real execution is enabled for the confirmed Development Pack workflow."
                        .to_string(),
            }
        } else if !user_confirmed {
            Self {
                mode: ExecutionMode::RealExecution,
                commands_allowed: false,
                safety_note:
                    "Real execution was requested, but user confirmation was not provided."
                        .to_string(),
            }
        } else {
            Self {
                mode: ExecutionMode::RealExecution,
                commands_allowed: false,
                safety_note: format!(
                    "Real execution is only enabled for the Development Pack during v0.5.7. Pack '{}' is blocked.",
                    pack_id
                ),
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandSpec {
    pub program: String,
    pub args: Vec<String>,
    pub requires_terminal_interaction: bool,
}

impl CommandSpec {
    pub fn new<I, S>(program: impl Into<String>, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
            requires_terminal_interaction: false,
        }
    }

    pub fn with_terminal_interaction(mut self) -> Self {
        self.requires_terminal_interaction = true;
        self
    }

    pub fn display_command(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(display_token(&self.program));
        parts.extend(self.args.iter().map(|arg| display_token(arg)));
        parts.join(" ")
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionStep {
    pub description: String,
    pub command: String,
    pub command_spec: CommandSpec,
}

impl ExecutionStep {
    fn new(description: impl Into<String>, command_spec: CommandSpec) -> Self {
        let command = command_spec.display_command();

        Self {
            description: description.into(),
            command,
            command_spec,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub pack_id: String,
    pub pack_name: String,
    pub dry_run: bool,
    pub execution_mode: ExecutionMode,
    pub command_boundary: ExecutionBoundary,
    pub steps: Vec<ExecutionStep>,
    pub requires_reboot: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandStatus {
    Planned,
    Succeeded,
    Failed,
    Skipped,
    Blocked,
}

impl CommandStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::Succeeded => "Succeeded",
            Self::Failed => "Failed",
            Self::Skipped => "Skipped",
            Self::Blocked => "Blocked",
        }
    }

    pub fn is_success(self) -> bool {
        matches!(self, Self::Succeeded)
    }

    pub fn is_failure(self) -> bool {
        matches!(self, Self::Failed | Self::Blocked)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallationErrorKind {
    None,
    Planned,
    DryRunSkipped,
    BoundaryBlocked,
    CommandStartFailure,
    PermissionFailure,
    PackageManagerFailure,
    NetworkOrRepositoryFailure,
    Warning,
    RebootRequired,
    UnknownFailure,
}

impl InstallationErrorKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "No Issue",
            Self::Planned => "Planned",
            Self::DryRunSkipped => "Dry-Run Skipped",
            Self::BoundaryBlocked => "Execution Boundary Blocked",
            Self::CommandStartFailure => "Command Start Failure",
            Self::PermissionFailure => "Permission Failure",
            Self::PackageManagerFailure => "Package Manager Failure",
            Self::NetworkOrRepositoryFailure => "Network or Repository Failure",
            Self::Warning => "Warning",
            Self::RebootRequired => "Reboot Required",
            Self::UnknownFailure => "Unknown Failure",
        }
    }

    pub fn is_failure(self) -> bool {
        matches!(
            self,
            Self::BoundaryBlocked
                | Self::CommandStartFailure
                | Self::PermissionFailure
                | Self::PackageManagerFailure
                | Self::NetworkOrRepositoryFailure
                | Self::UnknownFailure
        )
    }

    pub fn is_warning(self) -> bool {
        matches!(self, Self::Warning | Self::RebootRequired)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallationErrorClassification {
    pub kind: InstallationErrorKind,
    pub title: String,
    pub explanation: String,
    pub guidance: String,
    pub retry_safe: bool,
    pub user_action_required: bool,
}

impl InstallationErrorClassification {
    pub fn for_kind(kind: InstallationErrorKind) -> Self {
        match kind {
            InstallationErrorKind::None => Self::new(
                kind,
                "No issue detected.",
                "The command completed without an installation error classification.",
                "No action is required.",
                false,
                false,
            ),
            InstallationErrorKind::Planned => Self::new(
                kind,
                "Command planned.",
                "The command was generated for review but was not executed.",
                "Review the planned command. Use dry-run first before real execution.",
                false,
                false,
            ),
            InstallationErrorKind::DryRunSkipped => Self::new(
                kind,
                "Dry-run skipped execution.",
                "Dry-run mode intentionally avoids system modification.",
                "No action is required. Use the final confirmation flow only when real execution is intended.",
                false,
                false,
            ),
            InstallationErrorKind::BoundaryBlocked => Self::new(
                kind,
                "Execution boundary blocked the command.",
                "Forge Welcome refused to execute because the execution boundary did not allow commands.",
                "Return to the confirmation flow and verify that this is the Development Pack workflow. Do not bypass the boundary manually.",
                false,
                true,
            ),
            InstallationErrorKind::CommandStartFailure => Self::new(
                kind,
                "Command could not start.",
                "The operating system could not launch the configured command program.",
                "Verify that the required tool is installed and available in PATH, then retry from Forge Welcome.",
                true,
                true,
            ),
            InstallationErrorKind::PermissionFailure => Self::new(
                kind,
                "Permission was denied.",
                "The command appears to require authorization that was not granted or the password prompt was cancelled, unavailable, or timed out.",
                "Retry from Forge Welcome and complete the terminal sudo prompt. If no prompt appears, launch Forge Welcome from the same forge-dev terminal session or configure SUDO_ASKPASS for GUI authentication.",
                true,
                true,
            ),
            InstallationErrorKind::PackageManagerFailure => Self::new(
                kind,
                "Package manager reported a failure.",
                "The command started, but rpm-ostree, Flatpak, Distrobox, or DNF reported an installation failure.",
                "Review stdout and stderr for the package name or repository error. Retry after resolving the package manager issue.",
                true,
                true,
            ),
            InstallationErrorKind::NetworkOrRepositoryFailure => Self::new(
                kind,
                "Network or repository failure.",
                "The output suggests that the package source, repository metadata, or network connection was unavailable.",
                "Check network connectivity and repository availability, then retry from Forge Welcome.",
                true,
                true,
            ),
            InstallationErrorKind::Warning => Self::new(
                kind,
                "Command completed with warnings.",
                "The command succeeded, but output contained warning text or stderr content.",
                "Review the warning output. Retry is not required unless the installed tool does not work as expected.",
                false,
                true,
            ),
            InstallationErrorKind::RebootRequired => Self::new(
                kind,
                "Reboot required.",
                "The command completed and the output indicates that a reboot or new deployment activation is required.",
                "Finish any active work, then reboot before relying on the installed components.",
                false,
                true,
            ),
            InstallationErrorKind::UnknownFailure => Self::new(
                kind,
                "Unknown installation failure.",
                "The command failed, but Forge Welcome could not classify the failure more specifically.",
                "Review stdout and stderr. If the cause is unclear, retry after checking system logs and package-manager status.",
                true,
                true,
            ),
        }
    }

    fn new(
        kind: InstallationErrorKind,
        title: impl Into<String>,
        explanation: impl Into<String>,
        guidance: impl Into<String>,
        retry_safe: bool,
        user_action_required: bool,
    ) -> Self {
        Self {
            kind,
            title: title.into(),
            explanation: explanation.into(),
            guidance: guidance.into(),
            retry_safe,
            user_action_required,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub description: String,
    pub command: String,
    pub status: CommandStatus,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub message: Option<String>,
    pub classification: InstallationErrorClassification,
    pub reboot_required: bool,
    pub duration_ms: Option<u128>,
}

impl CommandResult {
    pub fn planned(step: &ExecutionStep) -> Self {
        Self {
            description: step.description.clone(),
            command: step.command.clone(),
            status: CommandStatus::Planned,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            message: Some("Command was planned but not executed.".to_string()),
            classification: InstallationErrorClassification::for_kind(
                InstallationErrorKind::Planned,
            ),
            reboot_required: false,
            duration_ms: None,
        }
    }

    pub fn blocked(step: &ExecutionStep, reason: impl Into<String>) -> Self {
        Self {
            description: step.description.clone(),
            command: step.command.clone(),
            status: CommandStatus::Blocked,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            message: Some(reason.into()),
            classification: InstallationErrorClassification::for_kind(
                InstallationErrorKind::BoundaryBlocked,
            ),
            reboot_required: false,
            duration_ms: None,
        }
    }

    pub fn skipped(step: &ExecutionStep, reason: impl Into<String>) -> Self {
        Self {
            description: step.description.clone(),
            command: step.command.clone(),
            status: CommandStatus::Skipped,
            exit_code: None,
            stdout: String::new(),
            stderr: String::new(),
            message: Some(reason.into()),
            classification: InstallationErrorClassification::for_kind(
                InstallationErrorKind::DryRunSkipped,
            ),
            reboot_required: false,
            duration_ms: None,
        }
    }

    pub fn succeeded(
        step: &ExecutionStep,
        exit_code: i32,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        reboot_required: bool,
        duration_ms: u128,
    ) -> Self {
        let stdout = stdout.into();
        let stderr = stderr.into();
        let classification = classify_success_output(&stdout, &stderr, reboot_required);

        Self {
            description: step.description.clone(),
            command: step.command.clone(),
            status: CommandStatus::Succeeded,
            exit_code: Some(exit_code),
            stdout,
            stderr,
            message: Some("Command completed successfully.".to_string()),
            classification,
            reboot_required,
            duration_ms: Some(duration_ms),
        }
    }

    pub fn failed(
        step: &ExecutionStep,
        exit_code: Option<i32>,
        stdout: impl Into<String>,
        stderr: impl Into<String>,
        message: impl Into<String>,
        reboot_required: bool,
        duration_ms: Option<u128>,
    ) -> Self {
        let stdout = stdout.into();
        let stderr = stderr.into();
        let message = message.into();
        let classification =
            classify_failed_output(step, exit_code, &stdout, &stderr, &message, reboot_required);

        Self {
            description: step.description.clone(),
            command: step.command.clone(),
            status: CommandStatus::Failed,
            exit_code,
            stdout,
            stderr,
            message: Some(message),
            classification,
            reboot_required,
            duration_ms,
        }
    }

    pub fn has_warning_classification(&self) -> bool {
        self.classification.kind.is_warning()
    }

    pub fn is_actionable_failure(&self) -> bool {
        self.status.is_failure() || self.classification.kind.is_failure()
    }

    pub fn guidance_summary(&self) -> String {
        format!(
            "{}: {}",
            self.classification.kind.label(),
            self.classification.guidance
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionWorkflowStatus {
    Planned,
    DryRun,
    Blocked,
    Succeeded,
    SucceededWithWarnings,
    PartialSuccess,
    Failed,
}

impl ExecutionWorkflowStatus {
    pub fn label(self) -> &'static str {
        match self {
            Self::Planned => "Planned",
            Self::DryRun => "Dry Run",
            Self::Blocked => "Blocked",
            Self::Succeeded => "Succeeded",
            Self::SucceededWithWarnings => "Succeeded With Warnings",
            Self::PartialSuccess => "Partial Success",
            Self::Failed => "Failed",
        }
    }

    pub fn is_terminal_success(self) -> bool {
        matches!(self, Self::Succeeded | Self::SucceededWithWarnings)
    }

    pub fn is_failure(self) -> bool {
        matches!(self, Self::Blocked | Self::PartialSuccess | Self::Failed)
    }
}

#[derive(Debug, Clone)]
pub struct ExecutionReport {
    pub pack_id: String,
    pub pack_name: String,
    pub dry_run: bool,
    pub execution_mode: ExecutionMode,
    pub command_boundary: ExecutionBoundary,
    pub results: Vec<CommandResult>,
    pub requires_reboot: bool,
}

impl ExecutionReport {
    pub fn result_count(&self) -> usize {
        self.results.len()
    }

    pub fn has_failures(&self) -> bool {
        self.results
            .iter()
            .any(CommandResult::is_actionable_failure)
    }

    pub fn succeeded_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status.is_success())
            .count()
    }

    pub fn failed_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == CommandStatus::Failed)
            .count()
    }

    pub fn blocked_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == CommandStatus::Blocked)
            .count()
    }

    pub fn skipped_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == CommandStatus::Skipped)
            .count()
    }

    pub fn planned_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.status == CommandStatus::Planned)
            .count()
    }

    pub fn warning_count(&self) -> usize {
        self.results
            .iter()
            .filter(|result| result.has_warning_classification())
            .count()
    }

    pub fn is_partial_success(&self) -> bool {
        self.succeeded_count() > 0 && self.has_failures()
    }

    pub fn workflow_status(&self) -> ExecutionWorkflowStatus {
        if self.result_count() > 0 && self.planned_count() == self.result_count() {
            ExecutionWorkflowStatus::Planned
        } else if self.dry_run {
            ExecutionWorkflowStatus::DryRun
        } else if self.blocked_count() > 0 && self.succeeded_count() == 0 {
            ExecutionWorkflowStatus::Blocked
        } else if self.is_partial_success() {
            ExecutionWorkflowStatus::PartialSuccess
        } else if self.has_failures() {
            ExecutionWorkflowStatus::Failed
        } else if self.warning_count() > 0 || self.requires_reboot {
            ExecutionWorkflowStatus::SucceededWithWarnings
        } else {
            ExecutionWorkflowStatus::Succeeded
        }
    }

    pub fn summary_line(&self) -> String {
        format!(
            "{}: {} of {} command(s) succeeded, {} failed, {} blocked, {} skipped, {} warning classification(s).",
            self.workflow_status().label(),
            self.succeeded_count(),
            self.result_count(),
            self.failed_count(),
            self.blocked_count(),
            self.skipped_count(),
            self.warning_count()
        )
    }
}

pub fn create_install_plan(pack: &Pack) -> InstallPlan {
    let mut plan = InstallPlan {
        pack_id: pack.id.clone(),
        pack_name: pack.name.clone(),
        host_packages: pack.host_packages.clone(),
        flatpaks: pack.flatpaks.clone(),
        distrobox_packages: pack.distrobox_packages.clone(),
        requires_reboot: pack.requires_reboot,
    };

    apply_development_validation_package_scope(&mut plan);

    plan
}

fn apply_development_validation_package_scope(plan: &mut InstallPlan) {
    if !is_development_pack_id(&plan.pack_id) {
        return;
    }

    plan.host_packages = vec![DEVELOPMENT_VALIDATION_PACKAGE.to_string()];
    plan.flatpaks.clear();
    plan.distrobox_packages.clear();
    plan.requires_reboot = false;
}

pub fn create_execution_plan(plan: &InstallPlan, dry_run: bool) -> ExecutionPlan {
    create_execution_plan_with_mode(plan, ExecutionMode::from_dry_run(dry_run))
}

pub fn create_execution_plan_with_mode(
    plan: &InstallPlan,
    execution_mode: ExecutionMode,
) -> ExecutionPlan {
    let runtime_environment = detect_runtime_install_environment();
    let steps = build_execution_steps_for_environment(plan, runtime_environment);

    ExecutionPlan {
        pack_id: plan.pack_id.clone(),
        pack_name: plan.pack_name.clone(),
        dry_run: execution_mode.is_dry_run(),
        execution_mode,
        command_boundary: ExecutionBoundary::for_mode(execution_mode),
        steps,
        requires_reboot: plan.requires_reboot,
    }
}

pub fn create_confirmed_development_execution_plan(
    plan: &InstallPlan,
    user_confirmed: bool,
) -> ExecutionPlan {
    let mut execution_plan = create_execution_plan_with_mode(plan, ExecutionMode::RealExecution);
    execution_plan.command_boundary =
        ExecutionBoundary::for_confirmed_development_pack(&plan.pack_id, user_confirmed);
    execution_plan
}

pub fn create_planned_command_results(plan: &ExecutionPlan) -> Vec<CommandResult> {
    plan.steps.iter().map(CommandResult::planned).collect()
}

pub fn create_execution_report(plan: &ExecutionPlan) -> ExecutionReport {
    ExecutionReport {
        pack_id: plan.pack_id.clone(),
        pack_name: plan.pack_name.clone(),
        dry_run: plan.dry_run,
        execution_mode: plan.execution_mode,
        command_boundary: plan.command_boundary.clone(),
        results: create_planned_command_results(plan),
        requires_reboot: plan.requires_reboot,
    }
}

pub fn execute_execution_plan(plan: &ExecutionPlan) -> ExecutionReport {
    if plan.dry_run {
        return ExecutionReport {
            pack_id: plan.pack_id.clone(),
            pack_name: plan.pack_name.clone(),
            dry_run: plan.dry_run,
            execution_mode: plan.execution_mode,
            command_boundary: plan.command_boundary.clone(),
            results: plan
                .steps
                .iter()
                .map(|step| {
                    CommandResult::skipped(step, "Dry-run mode does not execute system commands.")
                })
                .collect(),
            requires_reboot: false,
        };
    }

    if !plan.command_boundary.commands_allowed {
        return ExecutionReport {
            pack_id: plan.pack_id.clone(),
            pack_name: plan.pack_name.clone(),
            dry_run: plan.dry_run,
            execution_mode: plan.execution_mode,
            command_boundary: plan.command_boundary.clone(),
            results: plan
                .steps
                .iter()
                .map(|step| CommandResult::blocked(step, plan.command_boundary.safety_note.clone()))
                .collect(),
            requires_reboot: false,
        };
    }

    let results = plan
        .steps
        .iter()
        .map(execute_execution_step)
        .collect::<Vec<_>>();

    let requires_reboot = plan.requires_reboot
        || results
            .iter()
            .any(|result| result.reboot_required && result.status.is_success());

    ExecutionReport {
        pack_id: plan.pack_id.clone(),
        pack_name: plan.pack_name.clone(),
        dry_run: plan.dry_run,
        execution_mode: plan.execution_mode,
        command_boundary: plan.command_boundary.clone(),
        results,
        requires_reboot,
    }
}

fn classify_success_output(
    stdout: &str,
    stderr: &str,
    reboot_required: bool,
) -> InstallationErrorClassification {
    if reboot_required {
        InstallationErrorClassification::for_kind(InstallationErrorKind::RebootRequired)
    } else if output_mentions_warning(stdout)
        || output_mentions_warning(stderr)
        || !stderr.trim().is_empty()
    {
        InstallationErrorClassification::for_kind(InstallationErrorKind::Warning)
    } else {
        InstallationErrorClassification::for_kind(InstallationErrorKind::None)
    }
}

fn classify_failed_output(
    step: &ExecutionStep,
    _exit_code: Option<i32>,
    stdout: &str,
    stderr: &str,
    message: &str,
    _reboot_required: bool,
) -> InstallationErrorClassification {
    let combined = format!("{}\n{}\n{}", stdout, stderr, message).to_lowercase();

    if combined.contains("failed to start command")
        || combined.contains("no such file or directory")
        || combined.contains("command not found")
    {
        InstallationErrorClassification::for_kind(InstallationErrorKind::CommandStartFailure)
    } else if contains_any(
        &combined,
        &[
            "permission denied",
            "not authorized",
            "authorization required",
            "authentication required",
            "polkit",
            "pkexec",
            "error getting authority",
            "could not connect",
            "sudo: a password is required",
            "sudo: a terminal is required",
            "password is required",
            "authentication failed",
            "authentication failure",
            "authorization failed",
            "authorization was cancelled",
            "authorization canceled",
            "not authorized",
            "not granted",
            "conversation failed",
            "no authentication agent",
            "incorrect password",
            "sorry, try again",
            "polkit authentication agent",
        ],
    ) {
        InstallationErrorClassification::for_kind(InstallationErrorKind::PermissionFailure)
    } else if contains_any(
        &combined,
        &[
            "network",
            "connection timed out",
            "timeout",
            "could not resolve",
            "couldn't resolve",
            "cannot download",
            "failed to download",
            "repository",
            "repo",
            "metadata",
            "flathub",
            "mirror",
        ],
    ) {
        InstallationErrorClassification::for_kind(InstallationErrorKind::NetworkOrRepositoryFailure)
    } else if is_package_manager_program(&step.command_spec.program)
        || contains_any(
            &combined,
            &[
                "rpm-ostree",
                "flatpak",
                "distrobox",
                "dnf",
                "package",
                "dependency",
                "transaction",
            ],
        )
    {
        InstallationErrorClassification::for_kind(InstallationErrorKind::PackageManagerFailure)
    } else {
        InstallationErrorClassification::for_kind(InstallationErrorKind::UnknownFailure)
    }
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn is_package_manager_program(program: &str) -> bool {
    matches!(
        program,
        "rpm-ostree" | "flatpak" | "distrobox" | "dnf" | "sudo" | "pkexec"
    )
}

pub fn detect_runtime_install_environment() -> RuntimeInstallEnvironment {
    if let Some(environment) = runtime_environment_from_override() {
        return environment;
    }

    if is_distrobox_like_environment() {
        RuntimeInstallEnvironment::Distrobox
    } else if is_host_like_environment() {
        RuntimeInstallEnvironment::Host
    } else {
        RuntimeInstallEnvironment::Unknown
    }
}

fn runtime_environment_from_override() -> Option<RuntimeInstallEnvironment> {
    let value = env::var("FORGE_WELCOME_INSTALL_ENV").ok()?;
    runtime_environment_from_label(&value)
}

fn runtime_environment_from_label(value: &str) -> Option<RuntimeInstallEnvironment> {
    match value.trim().to_lowercase().as_str() {
        "host" | "forge-os" | "forgeos" => Some(RuntimeInstallEnvironment::Host),
        "distrobox" | "forge-dev" | "container" | "toolbox" => {
            Some(RuntimeInstallEnvironment::Distrobox)
        }
        "unknown" => Some(RuntimeInstallEnvironment::Unknown),
        _ => None,
    }
}

fn is_distrobox_like_environment() -> bool {
    env::var_os("DISTROBOX_ENTER_PATH").is_some()
        || env::var_os("DISTROBOX_HOST_HOME").is_some()
        || env::var_os("TOOLBOX_PATH").is_some()
        || env::var_os("container").is_some()
        || fs::metadata("/run/.containerenv").is_ok()
        || fs::metadata("/.dockerenv").is_ok()
}

fn is_host_like_environment() -> bool {
    fs::metadata("/run/ostree-booted").is_ok() || command_exists("rpm-ostree")
}

fn command_exists(program: &str) -> bool {
    env::var_os("PATH")
        .and_then(|paths| {
            env::split_paths(&paths).find_map(|path| {
                let candidate = path.join(program);
                if candidate.is_file() { Some(()) } else { None }
            })
        })
        .is_some()
}

fn package_install_strategy_for_environment(
    target: InstallTarget,
    runtime_environment: RuntimeInstallEnvironment,
) -> PackageInstallStrategy {
    match target {
        InstallTarget::FlatpakApplication => PackageInstallStrategy::Flatpak,
        InstallTarget::SystemPackage => {
            if runtime_environment.is_container_like() {
                PackageInstallStrategy::Dnf
            } else {
                PackageInstallStrategy::RpmOstree
            }
        }
        InstallTarget::ContainerPackage => PackageInstallStrategy::Dnf,
    }
}

fn build_execution_steps_for_environment(
    plan: &InstallPlan,
    runtime_environment: RuntimeInstallEnvironment,
) -> Vec<ExecutionStep> {
    let mut steps = Vec::new();

    if !plan.host_packages.is_empty() {
        let strategy = package_install_strategy_for_environment(
            InstallTarget::SystemPackage,
            runtime_environment,
        );
        steps.push(build_package_install_step(
            strategy,
            &plan.host_packages,
            runtime_environment,
            "system package",
        ));
    }

    if !plan.flatpaks.is_empty() {
        let strategy = package_install_strategy_for_environment(
            InstallTarget::FlatpakApplication,
            runtime_environment,
        );
        steps.push(build_package_install_step(
            strategy,
            &plan.flatpaks,
            runtime_environment,
            "Flatpak application",
        ));
    }

    if !plan.distrobox_packages.is_empty() {
        let strategy = package_install_strategy_for_environment(
            InstallTarget::ContainerPackage,
            runtime_environment,
        );
        steps.push(build_package_install_step(
            strategy,
            &plan.distrobox_packages,
            runtime_environment,
            "container package",
        ));
    }

    steps
}

fn build_package_install_step(
    strategy: PackageInstallStrategy,
    packages: &[String],
    runtime_environment: RuntimeInstallEnvironment,
    package_label: &str,
) -> ExecutionStep {
    match strategy {
        PackageInstallStrategy::RpmOstree => {
            let mut args = vec!["install".to_string()];
            args.extend(packages.iter().cloned());

            ExecutionStep::new(
                format!(
                    "Install {} host {}(s) with rpm-ostree",
                    packages.len(),
                    package_label
                ),
                CommandSpec::new("rpm-ostree", args),
            )
        }
        PackageInstallStrategy::Flatpak => {
            let mut args = vec![
                "install".to_string(),
                "-y".to_string(),
                "flathub".to_string(),
            ];
            args.extend(packages.iter().cloned());

            ExecutionStep::new(
                format!(
                    "Install {} host {}(s) with Flatpak",
                    packages.len(),
                    package_label
                ),
                CommandSpec::new("flatpak", args),
            )
        }
        PackageInstallStrategy::Dnf => {
            let privilege_method = detect_privilege_escalation_method();
            let command_spec = dnf_command_spec_for_method(packages, privilege_method);

            ExecutionStep::new(
                format!(
                    "Install {} {}(s) with DNF in {} using {}",
                    packages.len(),
                    package_label,
                    runtime_environment.label(),
                    privilege_method.label()
                ),
                command_spec,
            )
        }
    }
}

fn dnf_command_spec(packages: &[String]) -> CommandSpec {
    dnf_command_spec_for_method(packages, detect_privilege_escalation_method())
}

fn dnf_command_spec_for_method(
    packages: &[String],
    method: PrivilegeEscalationMethod,
) -> CommandSpec {
    let mut dnf_args = vec!["dnf".to_string(), "install".to_string(), "-y".to_string()];
    dnf_args.extend(packages.iter().cloned());

    match method {
        PrivilegeEscalationMethod::Direct => {
            let mut args = vec!["install".to_string(), "-y".to_string()];
            args.extend(packages.iter().cloned());
            CommandSpec::new("dnf", args)
        }
        PrivilegeEscalationMethod::Pkexec => CommandSpec::new("pkexec", dnf_args),
        PrivilegeEscalationMethod::SudoAskpass => {
            let mut args = vec!["-A".to_string()];
            args.extend(dnf_args);
            CommandSpec::new("sudo", args)
        }
        PrivilegeEscalationMethod::SudoInteractive => {
            CommandSpec::new("sudo", dnf_args).with_terminal_interaction()
        }
    }
}

pub fn detect_privilege_escalation_method() -> PrivilegeEscalationMethod {
    if let Some(method) = privilege_escalation_method_from_override() {
        return method;
    }

    if appears_to_be_running_as_root() {
        PrivilegeEscalationMethod::Direct
    } else if env::var_os("SUDO_ASKPASS").is_some() {
        PrivilegeEscalationMethod::SudoAskpass
    } else {
        // PolicyKit/pkexec is intentionally not selected automatically for forge-dev / Distrobox.
        // Container sessions often do not have access to a host PolicyKit authority, which causes
        // pkexec to fail with "Error getting authority" even when the package manager is correct.
        // The supported fallback for the validation run is a terminal-backed sudo prompt.
        PrivilegeEscalationMethod::SudoInteractive
    }
}

fn privilege_escalation_method_from_override() -> Option<PrivilegeEscalationMethod> {
    let value = env::var("FORGE_WELCOME_PRIVILEGE_METHOD").ok()?;
    privilege_escalation_method_from_label(&value)
}

fn privilege_escalation_method_from_label(value: &str) -> Option<PrivilegeEscalationMethod> {
    match value.trim().to_lowercase().as_str() {
        "direct" | "root" | "none" => Some(PrivilegeEscalationMethod::Direct),
        "pkexec" | "polkit" | "policykit" => Some(PrivilegeEscalationMethod::Pkexec),
        "sudo-askpass" | "askpass" => Some(PrivilegeEscalationMethod::SudoAskpass),
        "sudo" | "interactive-sudo" | "sudo-interactive" => {
            Some(PrivilegeEscalationMethod::SudoInteractive)
        }
        _ => None,
    }
}

fn appears_to_be_running_as_root() -> bool {
    env::var("EUID").ok().as_deref() == Some("0")
        || env::var("UID").ok().as_deref() == Some("0")
        || env::var("USER").ok().as_deref() == Some("root")
}

fn execute_execution_step(step: &ExecutionStep) -> CommandResult {
    if step.command_spec.requires_terminal_interaction {
        execute_execution_step_with_inherited_terminal(step)
    } else {
        execute_execution_step_with_captured_output(step)
    }
}

fn execute_execution_step_with_captured_output(step: &ExecutionStep) -> CommandResult {
    let started_at = Instant::now();
    let output = Command::new(&step.command_spec.program)
        .args(&step.command_spec.args)
        .output();

    match output {
        Ok(output) => {
            let duration_ms = started_at.elapsed().as_millis();
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let exit_code = output.status.code();
            let reboot_required =
                output_mentions_reboot(&stdout) || output_mentions_reboot(&stderr);

            if output.status.success() {
                CommandResult::succeeded(
                    step,
                    exit_code.unwrap_or(0),
                    stdout,
                    stderr,
                    reboot_required,
                    duration_ms,
                )
            } else {
                let message = match exit_code {
                    Some(code) => format!("Command exited with status code {code}."),
                    None => "Command terminated without a status code.".to_string(),
                };

                CommandResult::failed(
                    step,
                    exit_code,
                    stdout,
                    stderr,
                    message,
                    reboot_required,
                    Some(duration_ms),
                )
            }
        }
        Err(error) => CommandResult::failed(
            step,
            None,
            String::new(),
            String::new(),
            format!("Failed to start command: {error}"),
            false,
            None,
        ),
    }
}

fn execute_execution_step_with_inherited_terminal(step: &ExecutionStep) -> CommandResult {
    let started_at = Instant::now();
    let status = Command::new(&step.command_spec.program)
        .args(&step.command_spec.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status();

    match status {
        Ok(status) => {
            let duration_ms = started_at.elapsed().as_millis();
            let exit_code = status.code();
            let message = "Command used inherited terminal I/O for authentication and package-manager output.";

            if status.success() {
                CommandResult::succeeded(
                    step,
                    exit_code.unwrap_or(0),
                    String::new(),
                    String::new(),
                    false,
                    duration_ms,
                )
            } else {
                CommandResult::failed(
                    step,
                    exit_code,
                    String::new(),
                    String::new(),
                    match exit_code {
                        Some(code) => format!("{message} Command exited with status code {code}."),
                        None => format!("{message} Command terminated without a status code."),
                    },
                    false,
                    Some(duration_ms),
                )
            }
        }
        Err(error) => CommandResult::failed(
            step,
            None,
            String::new(),
            String::new(),
            format!("Failed to start command: {error}"),
            false,
            None,
        ),
    }
}

fn display_token(token: &str) -> String {
    if token
        .chars()
        .any(|character| character.is_whitespace() || matches!(character, '"' | '\''))
    {
        format!("\"{}\"", token.replace('"', "\\\""))
    } else {
        token.to_string()
    }
}

fn output_mentions_reboot(output: &str) -> bool {
    let normalized = output.to_lowercase();

    normalized.contains("reboot")
        || normalized.contains("restart required")
        || normalized.contains("new deployment")
        || normalized.contains("deployment complete")
}

fn output_mentions_warning(output: &str) -> bool {
    let normalized = output.to_lowercase();

    normalized.contains("warning") || normalized.contains("warn:") || normalized.contains("caution")
}

fn is_development_pack_id(pack_id: &str) -> bool {
    matches!(
        pack_id,
        "dev" | "developer" | "development" | "developer-pack" | "development-pack"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_install_plan() -> InstallPlan {
        InstallPlan {
            pack_id: "development".to_string(),
            pack_name: "Development Pack".to_string(),
            host_packages: vec!["git".to_string()],
            flatpaks: vec!["com.vscodium.codium".to_string()],
            distrobox_packages: vec!["zsh".to_string()],
            requires_reboot: true,
        }
    }

    #[test]
    fn development_install_plan_is_limited_to_kate_system_package_for_validation_patch() {
        let pack = Pack {
            id: "development".to_string(),
            name: "Development Pack".to_string(),
            description: "Development validation pack".to_string(),
            category: crate::PackCategory::Development,
            host_packages: vec!["git".to_string(), "rust".to_string()],
            flatpaks: vec!["com.vscodium.codium".to_string()],
            distrobox_packages: vec!["zsh".to_string()],
            requires_reboot: false,
        };

        let plan = create_install_plan(&pack);

        assert_eq!(plan.host_packages, vec!["kate".to_string()]);
        assert!(plan.flatpaks.is_empty());
        assert!(plan.distrobox_packages.is_empty());
        assert!(!plan.requires_reboot);
    }

    #[test]
    fn development_validation_execution_plan_installs_only_kate_with_dnf_in_distrobox_using_terminal_sudo_prompt()
     {
        let pack = Pack {
            id: "development".to_string(),
            name: "Development Pack".to_string(),
            description: "Development validation pack".to_string(),
            category: crate::PackCategory::Development,
            host_packages: vec!["git".to_string()],
            flatpaks: vec!["com.vscodium.codium".to_string()],
            distrobox_packages: vec!["zsh".to_string()],
            requires_reboot: false,
        };

        let install_plan = create_install_plan(&pack);
        let steps = build_execution_steps_for_environment(
            &install_plan,
            RuntimeInstallEnvironment::Distrobox,
        );

        assert_eq!(steps.len(), 1);
        assert!(matches!(
            steps[0].command_spec.program.as_str(),
            "sudo" | "dnf"
        ));
        assert_ne!(steps[0].command_spec.program, "pkexec");
        assert!(steps[0].command.contains("dnf"));
        assert!(steps[0].command.contains("install"));
        assert!(steps[0].command.contains("kate"));
        assert!(!steps[0].command.contains("sudo -n"));
    }

    #[test]
    fn development_validation_execution_plan_uses_rpm_ostree_on_host() {
        let pack = Pack {
            id: "development".to_string(),
            name: "Development Pack".to_string(),
            description: "Development validation pack".to_string(),
            category: crate::PackCategory::Development,
            host_packages: vec!["git".to_string()],
            flatpaks: vec!["com.vscodium.codium".to_string()],
            distrobox_packages: vec!["zsh".to_string()],
            requires_reboot: false,
        };

        let install_plan = create_install_plan(&pack);
        let steps =
            build_execution_steps_for_environment(&install_plan, RuntimeInstallEnvironment::Host);

        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].command_spec.program, "rpm-ostree");
        assert_eq!(
            steps[0].command_spec.args,
            vec!["install".to_string(), "kate".to_string()]
        );
        assert_eq!(steps[0].command, "rpm-ostree install kate");
    }

    #[test]
    fn flatpak_targets_use_flatpak_strategy_on_host() {
        let plan = InstallPlan {
            pack_id: "test".to_string(),
            pack_name: "Test Pack".to_string(),
            host_packages: Vec::new(),
            flatpaks: vec!["org.kde.kate".to_string()],
            distrobox_packages: Vec::new(),
            requires_reboot: false,
        };

        let steps = build_execution_steps_for_environment(&plan, RuntimeInstallEnvironment::Host);

        assert_eq!(steps.len(), 1);
        assert_eq!(steps[0].command_spec.program, "flatpak");
        assert_eq!(
            steps[0].command_spec.args,
            vec![
                "install".to_string(),
                "-y".to_string(),
                "flathub".to_string(),
                "org.kde.kate".to_string()
            ]
        );
    }

    #[test]
    fn runtime_environment_label_override_is_supported_for_tests_and_manual_validation() {
        assert_eq!(
            runtime_environment_from_label("forge-dev"),
            Some(RuntimeInstallEnvironment::Distrobox)
        );
        assert_eq!(
            runtime_environment_from_label("host"),
            Some(RuntimeInstallEnvironment::Host)
        );
        assert_eq!(runtime_environment_from_label("bad-value"), None);
    }

    #[test]
    fn dnf_strategy_can_be_forced_to_pkexec_for_explicit_override_only() {
        let packages = vec!["kate".to_string()];
        let command_spec =
            dnf_command_spec_for_method(&packages, PrivilegeEscalationMethod::Pkexec);

        assert_eq!(command_spec.program, "pkexec");
        assert_eq!(
            command_spec.args,
            vec![
                "dnf".to_string(),
                "install".to_string(),
                "-y".to_string(),
                "kate".to_string()
            ]
        );
        assert!(!command_spec.requires_terminal_interaction);
        assert_eq!(command_spec.display_command(), "pkexec dnf install -y kate");
    }

    #[test]
    fn dnf_strategy_uses_direct_dnf_when_already_root() {
        let packages = vec!["kate".to_string()];
        let command_spec =
            dnf_command_spec_for_method(&packages, PrivilegeEscalationMethod::Direct);

        assert_eq!(command_spec.program, "dnf");
        assert_eq!(
            command_spec.args,
            vec!["install".to_string(), "-y".to_string(), "kate".to_string()]
        );
    }

    #[test]
    fn dnf_strategy_uses_terminal_interactive_sudo_without_noninteractive_flag() {
        let packages = vec!["kate".to_string()];
        let command_spec =
            dnf_command_spec_for_method(&packages, PrivilegeEscalationMethod::SudoInteractive);

        assert_eq!(command_spec.program, "sudo");
        assert_eq!(command_spec.args[0], "dnf");
        assert!(command_spec.requires_terminal_interaction);
        assert!(!command_spec.args.iter().any(|arg| arg == "-n"));
        assert_eq!(command_spec.display_command(), "sudo dnf install -y kate");
    }

    #[test]
    fn privilege_method_label_override_is_supported_for_tests_and_manual_validation() {
        assert_eq!(
            privilege_escalation_method_from_label("pkexec"),
            Some(PrivilegeEscalationMethod::Pkexec)
        );
        assert_eq!(
            privilege_escalation_method_from_label("sudo-askpass"),
            Some(PrivilegeEscalationMethod::SudoAskpass)
        );
        assert_eq!(
            privilege_escalation_method_from_label("sudo"),
            Some(PrivilegeEscalationMethod::SudoInteractive)
        );
        assert_eq!(privilege_escalation_method_from_label("bad-value"), None);
    }

    #[test]
    fn automatic_privilege_selection_does_not_use_pkexec_for_container_validation() {
        let method = if appears_to_be_running_as_root() {
            PrivilegeEscalationMethod::Direct
        } else if env::var_os("SUDO_ASKPASS").is_some() {
            PrivilegeEscalationMethod::SudoAskpass
        } else {
            PrivilegeEscalationMethod::SudoInteractive
        };

        assert_ne!(method, PrivilegeEscalationMethod::Pkexec);
    }

    #[test]
    fn dry_run_is_default_execution_mode() {
        assert_eq!(ExecutionMode::default(), ExecutionMode::DryRun);
    }

    #[test]
    fn dry_run_plan_does_not_allow_command_execution() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::DryRun);

        assert!(execution_plan.dry_run);
        assert_eq!(execution_plan.execution_mode, ExecutionMode::DryRun);
        assert!(!execution_plan.command_boundary.commands_allowed);
        assert_eq!(execution_plan.steps.len(), 3);
        assert!(execution_plan.requires_reboot);
    }

    #[test]
    fn real_execution_mode_is_modeled_but_not_enabled_by_default() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::RealExecution);

        assert!(!execution_plan.dry_run);
        assert_eq!(execution_plan.execution_mode, ExecutionMode::RealExecution);
        assert!(!execution_plan.command_boundary.commands_allowed);
        assert!(
            execution_plan
                .command_boundary
                .safety_note
                .contains("requires explicit confirmation")
        );
    }

    #[test]
    fn confirmed_development_pack_enables_execution_boundary() {
        let plan = sample_install_plan();
        let execution_plan = create_confirmed_development_execution_plan(&plan, true);

        assert_eq!(execution_plan.execution_mode, ExecutionMode::RealExecution);
        assert!(!execution_plan.dry_run);
        assert!(execution_plan.command_boundary.commands_allowed);
    }

    #[test]
    fn unconfirmed_development_pack_keeps_execution_blocked() {
        let plan = sample_install_plan();
        let execution_plan = create_confirmed_development_execution_plan(&plan, false);

        assert!(!execution_plan.command_boundary.commands_allowed);
    }

    #[test]
    fn non_development_pack_keeps_execution_blocked() {
        let mut plan = sample_install_plan();
        plan.pack_id = "gaming".to_string();

        let execution_plan = create_confirmed_development_execution_plan(&plan, true);

        assert!(!execution_plan.command_boundary.commands_allowed);
    }

    #[test]
    fn legacy_dry_run_boolean_api_still_maps_to_execution_mode() {
        let plan = sample_install_plan();

        let dry_run_plan = create_execution_plan(&plan, true);
        let real_execution_plan = create_execution_plan(&plan, false);

        assert_eq!(dry_run_plan.execution_mode, ExecutionMode::DryRun);
        assert_eq!(
            real_execution_plan.execution_mode,
            ExecutionMode::RealExecution
        );
        assert!(!dry_run_plan.command_boundary.commands_allowed);
        assert!(!real_execution_plan.command_boundary.commands_allowed);
    }

    #[test]
    fn blocked_execution_plan_does_not_execute_commands() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::RealExecution);
        let report = execute_execution_plan(&execution_plan);

        assert!(report.has_failures());
        assert_eq!(report.results.len(), 3);
        assert!(
            report
                .results
                .iter()
                .all(|result| result.status == CommandStatus::Blocked)
        );
    }

    #[test]
    fn command_specs_use_program_and_arguments_instead_of_shell_strings() {
        let plan = sample_install_plan();
        let steps = build_execution_steps_for_environment(&plan, RuntimeInstallEnvironment::Host);

        assert_eq!(steps[0].command_spec.program, "rpm-ostree");
        assert_eq!(steps[0].command_spec.args[0], "install");
        assert_eq!(steps[1].command_spec.program, "flatpak");

        let container_steps =
            build_execution_steps_for_environment(&plan, RuntimeInstallEnvironment::Distrobox);
        assert!(matches!(
            container_steps[0].command_spec.program.as_str(),
            "sudo" | "dnf"
        ));
        assert_ne!(container_steps[0].command_spec.program, "pkexec");
        assert!(container_steps[0].command.contains("dnf"));
        assert!(!container_steps[0].command.contains("sudo -n"));
    }

    #[test]
    fn blocked_command_result_has_boundary_classification() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::RealExecution);
        let report = execute_execution_plan(&execution_plan);

        assert!(report.results.iter().all(|result| {
            result.classification.kind == InstallationErrorKind::BoundaryBlocked
        }));
    }

    #[test]
    fn failed_command_start_is_classified() {
        let step = ExecutionStep::new(
            "Start missing tool",
            CommandSpec::new("missing-forge-welcome-test-command", Vec::<String>::new()),
        );

        let result = CommandResult::failed(
            &step,
            None,
            "",
            "",
            "Failed to start command: No such file or directory",
            false,
            None,
        );

        assert_eq!(
            result.classification.kind,
            InstallationErrorKind::CommandStartFailure
        );
    }

    #[test]
    fn permission_failure_is_classified() {
        let step = ExecutionStep::new(
            "Install host package",
            CommandSpec::new("rpm-ostree", ["install", "git"]),
        );
        let result = CommandResult::failed(
            &step,
            Some(1),
            "",
            "error: Permission denied",
            "Command exited with status code 1.",
            false,
            Some(10),
        );

        assert_eq!(
            result.classification.kind,
            InstallationErrorKind::PermissionFailure
        );
    }

    #[test]
    fn network_failure_is_classified() {
        let step = ExecutionStep::new(
            "Install Flatpak application",
            CommandSpec::new("flatpak", ["install", "-y", "flathub", "com.example.App"]),
        );
        let result = CommandResult::failed(
            &step,
            Some(1),
            "",
            "Failed to download metadata from repository: connection timed out",
            "Command exited with status code 1.",
            false,
            Some(10),
        );

        assert_eq!(
            result.classification.kind,
            InstallationErrorKind::NetworkOrRepositoryFailure
        );
    }

    #[test]
    fn reboot_required_success_is_classified() {
        let step = ExecutionStep::new(
            "Install host package",
            CommandSpec::new("rpm-ostree", ["install", "git"]),
        );
        let result = CommandResult::succeeded(
            &step,
            0,
            "Staged deployment complete; reboot required",
            "",
            true,
            10,
        );

        assert_eq!(
            result.classification.kind,
            InstallationErrorKind::RebootRequired
        );
    }

    #[test]
    fn dry_run_execution_report_reports_stabilized_workflow_status() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::DryRun);
        let report = execute_execution_plan(&execution_plan);

        assert_eq!(report.workflow_status(), ExecutionWorkflowStatus::DryRun);
        assert_eq!(report.skipped_count(), 3);
        assert_eq!(report.failed_count(), 0);
        assert_eq!(report.blocked_count(), 0);
        assert!(!report.has_failures());
    }

    #[test]
    fn blocked_execution_report_reports_stabilized_workflow_status() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::RealExecution);
        let report = execute_execution_plan(&execution_plan);

        assert_eq!(report.workflow_status(), ExecutionWorkflowStatus::Blocked);
        assert_eq!(report.blocked_count(), 3);
        assert!(report.has_failures());
    }

    #[test]
    fn planned_execution_report_reports_stabilized_workflow_status() {
        let plan = sample_install_plan();
        let execution_plan = create_execution_plan_with_mode(&plan, ExecutionMode::DryRun);
        let report = create_execution_report(&execution_plan);

        assert_eq!(report.workflow_status(), ExecutionWorkflowStatus::Planned);
        assert_eq!(report.planned_count(), 3);
    }

    #[test]
    fn partial_success_report_reports_stabilized_workflow_status() {
        let success_step = ExecutionStep::new(
            "Install host package",
            CommandSpec::new("rpm-ostree", ["install", "git"]),
        );
        let failure_step = ExecutionStep::new(
            "Install Flatpak application",
            CommandSpec::new("flatpak", ["install", "-y", "flathub", "com.example.App"]),
        );

        let report = ExecutionReport {
            pack_id: "development".to_string(),
            pack_name: "Development Pack".to_string(),
            dry_run: false,
            execution_mode: ExecutionMode::RealExecution,
            command_boundary: ExecutionBoundary::for_confirmed_development_pack(
                "development",
                true,
            ),
            results: vec![
                CommandResult::succeeded(&success_step, 0, "ok", "", false, 5),
                CommandResult::failed(
                    &failure_step,
                    Some(1),
                    "",
                    "Failed to download metadata from repository: connection timed out",
                    "Command exited with status code 1.",
                    false,
                    Some(5),
                ),
            ],
            requires_reboot: false,
        };

        assert_eq!(
            report.workflow_status(),
            ExecutionWorkflowStatus::PartialSuccess
        );
        assert_eq!(report.succeeded_count(), 1);
        assert_eq!(report.failed_count(), 1);
        assert!(report.summary_line().contains("Partial Success"));
    }
}
