use serde_json::Value;
use std::{env, path::Path, process::Command};

const DEVELOPMENT_VALIDATION_TOOL_NAME: &str = "Kate";
const DEVELOPMENT_VALIDATION_TOOL_COMMAND: &str = "kate";
const KATE_PACKAGE_NAME: &str = "kate";
const KATE_FLATPAK_APP_ID: &str = "org.kde.kate";
const RPM_OSTREE_LAYERED_PACKAGE_KEYS: &[&str] = &[
    "requested-packages",
    "requested-local-packages",
    "base-layered-packages",
    "layered-packages",
    "layeredPackages",
    "LayeredPackages",
];

#[derive(Debug, Clone)]
pub struct DevelopmentPackStatus {
    pub tools: Vec<DevelopmentToolStatus>,
}

#[derive(Debug, Clone)]
pub struct DevelopmentToolStatus {
    pub name: String,
    pub command: String,
    pub installed: bool,
    pub version: Option<String>,
    pub install_source: InstallSource,
    pub removable: bool,
    pub detection_detail: String,
    pub detection_probes: Vec<DetectionProbeLogEntry>,
}

#[derive(Debug, Clone)]
pub struct DetectionProbeLogEntry {
    pub step: usize,
    pub probe_name: String,
    pub command_line: String,
    pub command_found: bool,
    pub success: bool,
    pub stdout_hint: String,
    pub stderr_hint: String,
}

impl DetectionProbeLogEntry {
    fn from_probe(step: usize, result: &ProbeResult) -> Self {
        Self {
            step,
            probe_name: result.name.to_string(),
            command_line: result.command_line.clone(),
            command_found: result.command_found,
            success: result.success,
            stdout_hint: compact_probe_text(&result.stdout),
            stderr_hint: compact_probe_text(&result.stderr),
        }
    }

    fn not_installed_fallback(step: usize) -> Self {
        Self {
            step,
            probe_name: "not_installed_fallback".to_string(),
            command_line: "NotInstalled".to_string(),
            command_found: true,
            success: true,
            stdout_hint: "No supported current host RPM/rpm-ostree or Flatpak source detected"
                .to_string(),
            stderr_hint: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallSource {
    NotInstalled,
    PendingOstreeInstall,
    PendingOstreeRemoval,
    HostOstreeLayered,
    HostBaseImage,
    FlatpakSystem,
    FlatpakUser,
    Unknown,
}

impl InstallSource {
    pub fn is_installed(self) -> bool {
        !matches!(self, Self::NotInstalled | Self::PendingOstreeInstall)
    }

    pub fn is_pending_reboot(self) -> bool {
        matches!(
            self,
            Self::PendingOstreeInstall | Self::PendingOstreeRemoval
        )
    }

    pub fn is_removable(self) -> bool {
        matches!(
            self,
            Self::HostOstreeLayered | Self::FlatpakSystem | Self::FlatpakUser
        )
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::NotInstalled => "Not installed",
            Self::PendingOstreeInstall => "Pending rpm-ostree installation",
            Self::PendingOstreeRemoval => "Pending rpm-ostree removal",
            Self::HostOstreeLayered => "Host rpm-ostree layered package",
            Self::HostBaseImage => "Host base image package",
            Self::FlatpakSystem => "System Flatpak",
            Self::FlatpakUser => "User Flatpak",
            Self::Unknown => "Installed source unknown",
        }
    }

    pub fn ui_metadata(self) -> &'static str {
        match self {
            Self::NotInstalled => "Host application",
            Self::PendingOstreeInstall => {
                "Kate installation staged by rpm-ostree · reboot required"
            }
            Self::PendingOstreeRemoval => "Kate removal staged by rpm-ostree · reboot required",
            Self::HostOstreeLayered | Self::HostBaseImage => "Host application",
            Self::FlatpakSystem | Self::FlatpakUser => "Flatpak application",
            Self::Unknown => "Installed · source unknown · uninstall disabled",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ReadOnlyProbe {
    command: &'static str,
    args: &'static [&'static str],
}

#[derive(Debug, Clone)]
struct ProbeResult {
    name: &'static str,
    command_line: String,
    command_found: bool,
    success: bool,
    stdout: String,
    stderr: String,
}

impl ProbeResult {
    fn summary(&self) -> String {
        let stdout_hint = compact_probe_text(&self.stdout);
        let stderr_hint = compact_probe_text(&self.stderr);
        format!(
            "{} command='{}' found={} success={} stdout='{}' stderr='{}'",
            self.name,
            self.command_line,
            self.command_found,
            self.success,
            stdout_hint,
            stderr_hint
        )
    }
}

struct KateProbeSet<'a> {
    json: &'a ProbeResult,
    rpm: &'a ProbeResult,
    flatpak_system: &'a ProbeResult,
    flatpak_user: &'a ProbeResult,
    executable: &'a ProbeResult,
}

fn executable_path_probe(executable_name: &str) -> ProbeResult {
    match find_executable_on_path(executable_name) {
        Some(path) => ProbeResult {
            name: "path_lookup_kate",
            command_line: format!("PATH lookup for {executable_name}"),
            command_found: true,
            success: true,
            stdout: path.display().to_string(),
            stderr: String::new(),
        },
        None => ProbeResult {
            name: "path_lookup_kate",
            command_line: format!("PATH lookup for {executable_name}"),
            command_found: true,
            success: false,
            stdout: String::new(),
            stderr: format!("{executable_name} was not found on PATH"),
        },
    }
}

fn find_executable_on_path(executable_name: &str) -> Option<std::path::PathBuf> {
    let path_value = env::var_os("PATH")?;

    env::split_paths(&path_value)
        .map(|path| path.join(executable_name))
        .find(|candidate| candidate.is_file())
}

const RPM_PACKAGE_PROBE: ReadOnlyProbe = ReadOnlyProbe {
    command: "rpm",
    args: &["-q", KATE_PACKAGE_NAME],
};

const RPM_OSTREE_STATUS_JSON_PROBE: ReadOnlyProbe = ReadOnlyProbe {
    command: "rpm-ostree",
    args: &["status", "--json"],
};

const RPM_OSTREE_STATUS_TEXT_PROBE: ReadOnlyProbe = ReadOnlyProbe {
    command: "rpm-ostree",
    args: &["status"],
};

const FLATPAK_SYSTEM_PROBE: ReadOnlyProbe = ReadOnlyProbe {
    command: "flatpak",
    args: &["info", "--system", KATE_FLATPAK_APP_ID],
};

const FLATPAK_USER_PROBE: ReadOnlyProbe = ReadOnlyProbe {
    command: "flatpak",
    args: &["info", "--user", KATE_FLATPAK_APP_ID],
};

pub fn detect_development_pack_status() -> DevelopmentPackStatus {
    let tools = vec![detect_kate_validation_status()];

    DevelopmentPackStatus { tools }
}

fn detect_kate_validation_status() -> DevelopmentToolStatus {
    let (install_source, detection_detail, detection_probes) =
        detect_kate_install_source_with_detail();
    build_kate_status(install_source, detection_detail, detection_probes)
}

fn build_kate_status(
    install_source: InstallSource,
    detection_detail: String,
    detection_probes: Vec<DetectionProbeLogEntry>,
) -> DevelopmentToolStatus {
    DevelopmentToolStatus {
        name: DEVELOPMENT_VALIDATION_TOOL_NAME.to_string(),
        command: DEVELOPMENT_VALIDATION_TOOL_COMMAND.to_string(),
        installed: install_source.is_installed(),
        version: (install_source.is_installed() || install_source.is_pending_reboot())
            .then(|| install_source.ui_metadata().to_string()),
        removable: install_source.is_removable(),
        install_source,
        detection_detail,
        detection_probes,
    }
}

fn detect_kate_install_source_with_detail() -> (InstallSource, String, Vec<DetectionProbeLogEntry>)
{
    let mut details = Vec::new();
    let mut probe_log = Vec::new();

    let ostree_host = is_ostree_host();
    details.push(format!("ostree_host={ostree_host}"));

    let json_probe = run_probe("rpm_ostree_status_json", &RPM_OSTREE_STATUS_JSON_PROBE);
    details.push(json_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(1, &json_probe));

    let text_probe = run_probe("rpm_ostree_status_text", &RPM_OSTREE_STATUS_TEXT_PROBE);
    details.push(text_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(2, &text_probe));

    let rpm_probe = run_probe("rpm_query_kate", &RPM_PACKAGE_PROBE);
    details.push(rpm_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(3, &rpm_probe));

    let flatpak_system_probe = run_probe("flatpak_system_info_kate", &FLATPAK_SYSTEM_PROBE);
    details.push(flatpak_system_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(4, &flatpak_system_probe));

    let flatpak_user_probe = run_probe("flatpak_user_info_kate", &FLATPAK_USER_PROBE);
    details.push(flatpak_user_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(5, &flatpak_user_probe));

    let executable_probe = executable_path_probe(DEVELOPMENT_VALIDATION_TOOL_COMMAND);
    details.push(executable_probe.summary());
    probe_log.push(DetectionProbeLogEntry::from_probe(6, &executable_probe));

    let probes = KateProbeSet {
        json: &json_probe,
        rpm: &rpm_probe,
        flatpak_system: &flatpak_system_probe,
        flatpak_user: &flatpak_user_probe,
        executable: &executable_probe,
    };

    let install_source = classify_kate_install_source(ostree_host, &probes, &mut details);

    if install_source == InstallSource::NotInstalled {
        probe_log.push(DetectionProbeLogEntry::not_installed_fallback(7));
    }

    (install_source, details.join(" | "), probe_log)
}

fn classify_kate_install_source(
    ostree_host: bool,
    probes: &KateProbeSet<'_>,
    details: &mut Vec<String>,
) -> InstallSource {
    if probes.flatpak_system.success {
        details.push("decision=FlatpakSystem via flatpak info --system".to_string());
        return InstallSource::FlatpakSystem;
    }

    if probes.flatpak_user.success {
        details.push("decision=FlatpakUser via flatpak info --user".to_string());
        return InstallSource::FlatpakUser;
    }

    let current_runtime_has_kate = probes.rpm.success && probes.executable.success;
    let rpm_ostree_json = ostree_host
        .then(|| RpmOstreeStatusJson::parse(&probes.json.stdout))
        .flatten();
    let rpm_ostree_status_available = rpm_ostree_json.is_some();
    let staged_deployment_present = rpm_ostree_json
        .as_ref()
        .is_some_and(|status| status.has_deployment_with_true_key("staged"));
    let staged_mentions_kate = rpm_ostree_json
        .as_ref()
        .is_some_and(|status| status.staged_deployment_mentions_package(KATE_PACKAGE_NAME));
    let booted_mentions_kate = rpm_ostree_json
        .as_ref()
        .is_some_and(|status| status.booted_deployment_mentions_package(KATE_PACKAGE_NAME));

    // Guard against stale rpm-ostree deployment evidence. The app must not show
    // Kate as installed unless the current runtime can also prove that the Kate
    // RPM package exists and the Kate executable is available on PATH.
    if !current_runtime_has_kate {
        if staged_mentions_kate {
            details.push(
                "decision=PendingOstreeInstall via staged non-booted rpm-ostree deployment"
                    .to_string(),
            );
            return InstallSource::PendingOstreeInstall;
        }

        if probes.rpm.success && !probes.executable.success {
            details.push(
                "rpm_query_kate succeeded, but the kate executable was not found on PATH; treating Kate as NotInstalled"
                    .to_string(),
            );
        }

        if rpm_ostree_json
            .as_ref()
            .is_some_and(|status| status.any_deployment_mentions_package(KATE_PACKAGE_NAME))
        {
            details.push(
                "rpm-ostree status mentions kate in a deployment, but current host rpm/path evidence does not show Kate installed; treating Kate as NotInstalled"
                    .to_string(),
            );
        }

        details.push("decision=NotInstalled current_runtime_has_kate=false".to_string());
        return InstallSource::NotInstalled;
    }

    if staged_deployment_present && booted_mentions_kate && !staged_mentions_kate {
        details.push(
            "decision=PendingOstreeRemoval via Kate present in booted deployment and absent from staged deployment"
                .to_string(),
        );
        return InstallSource::PendingOstreeRemoval;
    }

    if ostree_host
        && rpm_ostree_json
            .as_ref()
            .is_some_and(|status| status.booted_deployment_mentions_package(KATE_PACKAGE_NAME))
    {
        details.push(
            "decision=HostOstreeLayered via current runtime RPM evidence plus structured booted rpm-ostree deployment evidence"
                .to_string(),
        );
        return InstallSource::HostOstreeLayered;
    }

    if ostree_host && rpm_ostree_status_available {
        details.push(
            "decision=Unknown because Kate is active, but structured rpm-ostree status has no booted layered/requested Kate evidence"
                .to_string(),
        );
        return InstallSource::Unknown;
    }

    if ostree_host {
        details.push(
            "decision=Unknown because Kate is active but rpm-ostree status did not return usable layered/base-image evidence"
                .to_string(),
        );
        return InstallSource::Unknown;
    }

    details.push(
        "decision=Unknown via current rpm -q kate plus executable path confirmation outside an ostree host"
            .to_string(),
    );
    InstallSource::Unknown
}

fn is_ostree_host() -> bool {
    Path::new("/run/ostree-booted").exists()
}

#[cfg(test)]
fn rpm_ostree_any_deployment_mentions_package(output: &str, package_name: &str) -> bool {
    RpmOstreeStatusJson::parse(output)
        .is_some_and(|status| status.any_deployment_mentions_package(package_name))
}

#[cfg(test)]
fn rpm_ostree_staged_deployment_mentions_package(output: &str, package_name: &str) -> bool {
    RpmOstreeStatusJson::parse(output)
        .is_some_and(|status| status.staged_deployment_mentions_package(package_name))
}

#[cfg(test)]
fn rpm_ostree_current_json_deployment_mentions_package(output: &str, package_name: &str) -> bool {
    RpmOstreeStatusJson::parse(output)
        .is_some_and(|status| status.booted_deployment_mentions_package(package_name))
}

struct RpmOstreeStatusJson {
    deployments: Vec<Value>,
}

impl RpmOstreeStatusJson {
    fn parse(output: &str) -> Option<Self> {
        let status: Value = serde_json::from_str(output).ok()?;
        let deployments = status.get("deployments")?.as_array()?.clone();

        Some(Self { deployments })
    }

    fn booted_deployment_mentions_package(&self, package_name: &str) -> bool {
        self.deployment_with_true_key("booted")
            .is_some_and(|deployment| deployment_mentions_package(deployment, package_name))
    }

    fn staged_deployment_mentions_package(&self, package_name: &str) -> bool {
        self.deployment_with_true_key("staged")
            .is_some_and(|deployment| deployment_mentions_package(deployment, package_name))
    }

    fn any_deployment_mentions_package(&self, package_name: &str) -> bool {
        self.deployment_values()
            .any(|deployment| deployment_mentions_package(deployment, package_name))
    }

    fn deployment_with_true_key(&self, key: &str) -> Option<&Value> {
        self.deployment_values()
            .find(|deployment| deployment.get(key).and_then(Value::as_bool) == Some(true))
    }

    fn has_deployment_with_true_key(&self, key: &str) -> bool {
        self.deployment_with_true_key(key).is_some()
    }

    fn deployment_values(&self) -> impl Iterator<Item = &Value> {
        self.deployments
            .iter()
            .filter(|deployment| deployment.is_object())
    }
}

fn deployment_mentions_package(deployment: &Value, package_name: &str) -> bool {
    RPM_OSTREE_LAYERED_PACKAGE_KEYS
        .iter()
        .any(|key| json_array_contains_package(deployment, key, package_name))
}

fn json_array_contains_package(deployment: &Value, key: &str, package_name: &str) -> bool {
    deployment
        .get(key)
        .and_then(Value::as_array)
        .is_some_and(|packages| {
            packages.iter().any(|package| {
                package
                    .as_str()
                    .is_some_and(|name| package_name_matches(name, package_name))
            })
        })
}

fn package_name_matches(candidate: &str, package_name: &str) -> bool {
    candidate == package_name
        || candidate
            .strip_prefix(package_name)
            .is_some_and(|suffix| suffix.starts_with('-') && suffix_is_version_like(&suffix[1..]))
}

fn suffix_is_version_like(suffix: &str) -> bool {
    suffix
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
}

#[cfg(test)]
fn rpm_ostree_text_mentions_layered_package(output: &str, package_name: &str) -> bool {
    output.lines().any(|line| {
        let lower = line.to_lowercase();
        let package_line = lower.contains("layeredpackages")
            || lower.contains("layered packages")
            || lower.contains("requestedpackages")
            || lower.contains("requested packages");

        package_line
            && line.split_whitespace().any(|part| {
                part.trim_matches(|c: char| c == ',' || c == '[' || c == ']' || c == '"')
                    == package_name
            })
    })
}

#[cfg(test)]
fn json_array_contains_string(json_text: &str, key: &str, value: &str) -> bool {
    RpmOstreeStatusJson::parse(json_text).is_some_and(|status| {
        status
            .deployment_values()
            .any(|deployment| json_array_contains_package(deployment, key, value))
    })
}

fn run_probe(name: &'static str, probe: &ReadOnlyProbe) -> ProbeResult {
    let command_line = format!("{} {}", probe.command, probe.args.join(" "));

    match Command::new(probe.command).args(probe.args).output() {
        Ok(output) => ProbeResult {
            name,
            command_line,
            command_found: true,
            success: output.status.success(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        },
        Err(error) => ProbeResult {
            name,
            command_line,
            command_found: false,
            success: false,
            stdout: String::new(),
            stderr: error.to_string(),
        },
    }
}

fn compact_probe_text(text: &str) -> String {
    let compact = text
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(3)
        .collect::<Vec<_>>()
        .join(" / ");

    if compact.chars().count() > 180 {
        compact.chars().take(180).collect::<String>() + "..."
    } else {
        compact
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn successful_probe(name: &'static str, stdout: &str) -> ProbeResult {
        ProbeResult {
            name,
            command_line: name.to_string(),
            command_found: true,
            success: true,
            stdout: stdout.to_string(),
            stderr: String::new(),
        }
    }

    fn failed_probe(name: &'static str) -> ProbeResult {
        ProbeResult {
            name,
            command_line: name.to_string(),
            command_found: true,
            success: false,
            stdout: String::new(),
            stderr: String::new(),
        }
    }

    fn classify_test_kate(
        ostree_json: &str,
        _ostree_text: &str,
        rpm_success: bool,
        path_success: bool,
    ) -> InstallSource {
        let json_probe = if ostree_json.is_empty() {
            failed_probe("rpm_ostree_status_json")
        } else {
            successful_probe("rpm_ostree_status_json", ostree_json)
        };
        let rpm_probe = if rpm_success {
            successful_probe("rpm_query_kate", "kate-26.04.3-1.fc44.x86_64")
        } else {
            failed_probe("rpm_query_kate")
        };
        let executable_probe = if path_success {
            successful_probe("path_lookup_kate", "/usr/bin/kate")
        } else {
            failed_probe("path_lookup_kate")
        };
        let flatpak_system_probe = failed_probe("flatpak_system_info_kate");
        let flatpak_user_probe = failed_probe("flatpak_user_info_kate");
        let mut details = vec!["ostree_host=true".to_string()];
        let probes = KateProbeSet {
            json: &json_probe,
            rpm: &rpm_probe,
            flatpak_system: &flatpak_system_probe,
            flatpak_user: &flatpak_user_probe,
            executable: &executable_probe,
        };

        classify_kate_install_source(true, &probes, &mut details)
    }

    #[test]
    fn kate_validation_status_uses_expected_identity() {
        let status = detect_kate_validation_status();

        assert_eq!(status.name, DEVELOPMENT_VALIDATION_TOOL_NAME);
        assert_eq!(status.command, DEVELOPMENT_VALIDATION_TOOL_COMMAND);
    }

    #[test]
    fn installed_source_metadata_is_not_icon_based() {
        assert_eq!(
            InstallSource::HostOstreeLayered.ui_metadata(),
            "Host application"
        );
        assert_eq!(
            InstallSource::HostBaseImage.ui_metadata(),
            "Host application"
        );
        assert_eq!(
            InstallSource::FlatpakSystem.ui_metadata(),
            "Flatpak application"
        );
        assert_eq!(
            InstallSource::FlatpakUser.ui_metadata(),
            "Flatpak application"
        );
        assert!(
            !InstallSource::HostOstreeLayered
                .ui_metadata()
                .to_lowercase()
                .contains("icon")
        );
    }

    #[test]
    fn removable_sources_are_source_specific() {
        assert!(InstallSource::HostOstreeLayered.is_removable());
        assert!(InstallSource::FlatpakSystem.is_removable());
        assert!(InstallSource::FlatpakUser.is_removable());
        assert!(!InstallSource::HostBaseImage.is_removable());
        assert!(!InstallSource::Unknown.is_removable());
        assert!(!InstallSource::NotInstalled.is_removable());
        assert!(!InstallSource::PendingOstreeInstall.is_removable());
        assert!(!InstallSource::PendingOstreeRemoval.is_removable());
    }

    #[test]
    fn not_installed_fallback_probe_has_expected_step() {
        let fallback = DetectionProbeLogEntry::not_installed_fallback(7);

        assert_eq!(fallback.step, 7);
        assert_eq!(fallback.command_line, "NotInstalled");
        assert!(fallback.success);
    }

    #[test]
    fn read_only_probe_definitions_do_not_use_shell() {
        for probe in [
            RPM_PACKAGE_PROBE,
            RPM_OSTREE_STATUS_JSON_PROBE,
            RPM_OSTREE_STATUS_TEXT_PROBE,
            FLATPAK_SYSTEM_PROBE,
            FLATPAK_USER_PROBE,
        ] {
            assert_ne!(probe.command, "sh");
            assert_ne!(probe.command, "bash");
            assert!(!probe.args.is_empty());
        }
    }

    #[test]
    fn json_array_detection_finds_layered_package_in_first_matching_array() {
        let json = r#"{
            "deployments": [
                {
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert!(json_array_contains_string(
            json,
            "requested-packages",
            "kate"
        ));
        assert!(!json_array_contains_string(
            json,
            "requested-packages",
            "vim"
        ));
    }

    #[test]
    fn json_array_detection_checks_later_matching_arrays() {
        let json = r#"{
            "deployments": [
                { "requested-packages": ["git"] },
                { "requested-packages": ["kate"] }
            ]
        }"#;

        assert!(json_array_contains_string(
            json,
            "requested-packages",
            "kate"
        ));
    }

    #[test]
    fn current_deployment_detection_uses_booted_deployment_only() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["git"]
                },
                {
                    "booted": false,
                    "requested-packages": ["kate"]
                }
            ]
        }"#;

        assert!(!rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
        assert!(rpm_ostree_any_deployment_mentions_package(json, "kate"));
    }

    #[test]
    fn current_deployment_detection_finds_current_layered_kate() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert!(rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
    }

    #[test]
    fn current_deployment_detection_uses_full_booted_deployment_with_nested_metadata() {
        let json = r#"{
            "deployments": [
                {
                    "metadata": {
                        "rpmostree": {
                            "booted": true,
                            "requested-packages": ["not-kate"]
                        }
                    },
                    "booted": true,
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert!(rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
    }

    #[test]
    fn current_deployment_detection_accepts_version_like_package_suffix() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["kate-26.04.3-1.fc44.x86_64", "git"]
                }
            ]
        }"#;

        assert!(rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
    }

    #[test]
    fn current_deployment_detection_rejects_broad_fuzzy_package_names() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["kate-plugin", "kate-devel", "libkate"]
                }
            ]
        }"#;

        assert!(!rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
    }

    #[test]
    fn staged_deployment_detection_finds_pending_kate() {
        let json = r#"{
            "deployments": [
                {
                    "booted": false,
                    "staged": true,
                    "requested-packages": ["kate", "git"]
                },
                {
                    "booted": true,
                    "staged": false,
                    "requested-packages": ["git"]
                }
            ]
        }"#;

        assert!(rpm_ostree_staged_deployment_mentions_package(json, "kate"));
        assert!(!rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
    }

    #[test]
    fn non_staged_rollback_deployment_is_not_pending() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["git"]
                },
                {
                    "booted": false,
                    "requested-packages": ["kate"]
                }
            ]
        }"#;

        assert!(!rpm_ostree_staged_deployment_mentions_package(json, "kate"));
    }

    #[test]
    fn runtime_evidence_is_required_before_current_ostree_detection() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["kate"]
                }
            ]
        }"#;

        assert!(rpm_ostree_current_json_deployment_mentions_package(
            json, "kate"
        ));
        assert!(rpm_ostree_any_deployment_mentions_package(json, "kate"));
    }

    #[test]
    fn active_layered_kate_classifies_as_removable_host_ostree_layered() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "", true, true),
            InstallSource::HostOstreeLayered
        );
    }

    #[test]
    fn booted_deployment_with_nested_metadata_before_booted_requested_kate_is_removable() {
        let json = r#"{
            "deployments": [
                {
                    "metadata": {
                        "origin": {
                            "booted": true,
                            "requested-packages": ["not-kate"]
                        }
                    },
                    "booted": true,
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "", true, true),
            InstallSource::HostOstreeLayered
        );
    }

    #[test]
    fn booted_deployment_with_base_layered_packages_kate_is_removable() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "base-layered-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "", true, true),
            InstallSource::HostOstreeLayered
        );
    }

    #[test]
    fn booted_deployment_with_layered_packages_key_variants_kate_is_removable() {
        for key in ["layered-packages", "layeredPackages", "LayeredPackages"] {
            let json = format!(
                r#"{{
                    "deployments": [
                        {{
                            "booted": true,
                            "{key}": ["kate", "git"]
                        }}
                    ]
                }}"#
            );

            assert_eq!(
                classify_test_kate(&json, "", true, true),
                InstallSource::HostOstreeLayered,
                "expected {key} to classify Kate as layered"
            );
        }
    }

    #[test]
    fn active_kate_with_valid_booted_deployment_and_no_layered_status_fails_closed() {
        let json = r#"{
            "deployments": [
                {
                    "booted": true,
                    "requested-packages": ["git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "State: idle\n", true, true),
            InstallSource::Unknown
        );
    }

    #[test]
    fn active_kate_with_unavailable_ostree_status_fails_closed() {
        assert_eq!(
            classify_test_kate("", "", true, true),
            InstallSource::Unknown
        );
    }

    #[test]
    fn active_kate_with_unusable_ostree_json_on_ostree_host_fails_closed() {
        assert_eq!(
            classify_test_kate(
                "{ invalid json",
                "State: idle\nLayeredPackages: git kate\n",
                true,
                true
            ),
            InstallSource::Unknown
        );
    }

    #[test]
    fn pending_reboot_install_remains_non_active_source() {
        let json = r#"{
            "deployments": [
                {
                    "booted": false,
                    "staged": true,
                    "requested-packages": ["kate", "git"]
                },
                {
                    "booted": true,
                    "requested-packages": ["git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "", false, false),
            InstallSource::PendingOstreeInstall
        );
    }

    #[test]
    fn pending_reboot_removal_remains_non_actionable_source() {
        let json = r#"{
            "deployments": [
                {
                    "booted": false,
                    "staged": true,
                    "requested-packages": ["git"]
                },
                {
                    "booted": true,
                    "requested-packages": ["kate", "git"]
                }
            ]
        }"#;

        assert_eq!(
            classify_test_kate(json, "", true, true),
            InstallSource::PendingOstreeRemoval
        );
    }

    #[test]
    fn rpm_ostree_text_detection_finds_layered_package_line() {
        let text = "State: idle\nLayeredPackages: git kate zsh\n";

        assert!(rpm_ostree_text_mentions_layered_package(text, "kate"));
        assert!(!rpm_ostree_text_mentions_layered_package(text, "vim"));
    }

    #[test]
    fn rpm_ostree_text_detection_ignores_generic_package_lines() {
        let text = "State: idle\nPackages: git kate zsh\n";

        assert!(!rpm_ostree_text_mentions_layered_package(text, "kate"));
    }

    #[test]
    fn rpm_ostree_json_detection_does_not_match_generic_packages_array() {
        let json = r#"{
            "deployments": [
                {
                    "packages": ["kate", "git"]
                }
            ]
        }"#;

        assert!(!rpm_ostree_any_deployment_mentions_package(json, "kate"));
    }
}
