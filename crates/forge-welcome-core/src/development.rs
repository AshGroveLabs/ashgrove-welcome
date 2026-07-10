use std::{env, path::Path, process::Command};

const DEVELOPMENT_VALIDATION_TOOL_NAME: &str = "Kate";
const DEVELOPMENT_VALIDATION_TOOL_COMMAND: &str = "kate";
const KATE_PACKAGE_NAME: &str = "kate";
const KATE_FLATPAK_APP_ID: &str = "org.kde.kate";

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
    HostOstreeLayered,
    HostBaseImage,
    FlatpakSystem,
    FlatpakUser,
    Unknown,
}

impl InstallSource {
    pub fn is_installed(self) -> bool {
        !matches!(self, Self::NotInstalled)
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
            Self::HostOstreeLayered => "Host rpm-ostree layered package",
            Self::HostBaseImage => "Host base image package",
            Self::FlatpakSystem => "System Flatpak",
            Self::FlatpakUser => "User Flatpak",
            Self::Unknown => "Installed source unknown",
        }
    }

    pub fn ui_metadata(self) -> &'static str {
        match self {
            Self::NotInstalled => "Host application · kate · Development Pack validation item",
            Self::HostOstreeLayered => "Installed via host rpm-ostree layered package · removable",
            Self::HostBaseImage => "Installed in host base image · OS-owned package",
            Self::FlatpakSystem => "Installed via system Flatpak · removable",
            Self::FlatpakUser => "Installed via user Flatpak · removable",
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
        version: install_source
            .is_installed()
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

    if flatpak_system_probe.success {
        details.push("decision=FlatpakSystem via flatpak info --system".to_string());
        return (InstallSource::FlatpakSystem, details.join(" | "), probe_log);
    }

    if flatpak_user_probe.success {
        details.push("decision=FlatpakUser via flatpak info --user".to_string());
        return (InstallSource::FlatpakUser, details.join(" | "), probe_log);
    }

    let current_runtime_has_kate = rpm_probe.success && executable_probe.success;

    // Guard against stale rpm-ostree deployment evidence. The app must not show
    // Kate as installed unless the current runtime can also prove that the Kate
    // RPM package exists and the Kate executable is available on PATH.
    if !current_runtime_has_kate {
        if rpm_probe.success && !executable_probe.success {
            details.push(
                "rpm_query_kate succeeded, but the kate executable was not found on PATH; treating Kate as NotInstalled"
                    .to_string(),
            );
        }

        if ostree_host
            && rpm_ostree_any_deployment_mentions_package(&json_probe.stdout, KATE_PACKAGE_NAME)
        {
            details.push(
                "rpm-ostree status mentions kate in a deployment, but current host rpm/path evidence does not show Kate installed; treating Kate as NotInstalled"
                    .to_string(),
            );
        }

        probe_log.push(DetectionProbeLogEntry::not_installed_fallback(7));
        details.push("decision=NotInstalled current_runtime_has_kate=false".to_string());
        return (InstallSource::NotInstalled, details.join(" | "), probe_log);
    }

    if ostree_host
        && rpm_ostree_current_deployment_mentions_package(
            &json_probe.stdout,
            &text_probe.stdout,
            KATE_PACKAGE_NAME,
        )
    {
        details.push(
            "decision=HostOstreeLayered via current runtime RPM evidence plus current rpm-ostree deployment evidence"
                .to_string(),
        );
        return (
            InstallSource::HostOstreeLayered,
            details.join(" | "),
            probe_log,
        );
    }

    if ostree_host {
        details.push(
            "decision=HostBaseImage via current rpm -q kate plus executable path confirmation without layered rpm-ostree evidence"
                .to_string(),
        );
        return (InstallSource::HostBaseImage, details.join(" | "), probe_log);
    }

    details.push(
        "decision=Unknown via current rpm -q kate plus executable path confirmation outside an ostree host"
            .to_string(),
    );
    (InstallSource::Unknown, details.join(" | "), probe_log)
}

fn is_ostree_host() -> bool {
    Path::new("/run/ostree-booted").exists()
}

fn rpm_ostree_current_deployment_mentions_package(
    json_output: &str,
    text_output: &str,
    package_name: &str,
) -> bool {
    rpm_ostree_current_json_deployment_mentions_package(json_output, package_name)
        || rpm_ostree_text_mentions_layered_package(text_output, package_name)
}

fn rpm_ostree_any_deployment_mentions_package(output: &str, package_name: &str) -> bool {
    json_array_contains_string(output, "requested-packages", package_name)
        || json_array_contains_string(output, "requested-local-packages", package_name)
        || json_array_contains_string(output, "base-layered-packages", package_name)
}

fn rpm_ostree_current_json_deployment_mentions_package(output: &str, package_name: &str) -> bool {
    let Some(deployment) = extract_booted_deployment_object(output) else {
        return false;
    };

    json_array_contains_string(&deployment, "requested-packages", package_name)
        || json_array_contains_string(&deployment, "requested-local-packages", package_name)
        || json_array_contains_string(&deployment, "base-layered-packages", package_name)
}

fn extract_booted_deployment_object(json_text: &str) -> Option<String> {
    for (key_start, _) in json_text.match_indices("\"booted\"") {
        if !json_bool_key_is_true(&json_text[key_start..]) {
            continue;
        }

        let object_start = json_text[..key_start].rfind('{')?;
        let object_end = find_matching_json_object_end(json_text, object_start)?;
        return Some(json_text[object_start..=object_end].to_string());
    }

    None
}

fn json_bool_key_is_true(text_from_key: &str) -> bool {
    let Some(colon_index) = text_from_key.find(':') else {
        return false;
    };

    text_from_key[colon_index + 1..]
        .trim_start()
        .starts_with("true")
}

fn find_matching_json_object_end(json_text: &str, object_start: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, character) in json_text[object_start..].char_indices() {
        if in_string {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_string = false;
            }
            continue;
        }

        match character {
            '"' => in_string = true,
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(object_start + offset);
                }
            }
            _ => {}
        }
    }

    None
}

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

fn json_array_contains_string(json_text: &str, key: &str, value: &str) -> bool {
    let key_pattern = format!("\"{key}\"");
    let value_pattern = format!("\"{value}\"");
    let mut search_start = 0;

    while let Some(relative_key_start) = json_text[search_start..].find(&key_pattern) {
        let key_start = search_start + relative_key_start;
        let after_key = &json_text[key_start + key_pattern.len()..];
        let Some(array_start_relative) = after_key.find('[') else {
            search_start = key_start + key_pattern.len();
            continue;
        };

        let array_text = &after_key[array_start_relative..];
        let Some(array_end_relative) = array_text.find(']') else {
            return false;
        };

        let array_body = &array_text[..=array_end_relative];
        if array_body.contains(&value_pattern) {
            return true;
        }

        search_start = key_start + key_pattern.len();
    }

    false
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
            "Installed via host rpm-ostree layered package · removable"
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
