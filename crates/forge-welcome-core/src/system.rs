use std::process::Command;

#[derive(Debug, Clone)]
pub struct SystemDashboard {
    pub overview: SystemOverview,
    pub atomic: AtomicStatus,
    pub development: DevelopmentEnvironmentStatus,
    pub applications: ApplicationStatus,
}

#[derive(Debug, Clone)]
pub struct SystemOverview {
    pub forge_os_version: String,
    pub fedora_version: Option<String>,
    pub deployment_status: DeploymentStatus,
    pub runtime_environment: RuntimeEnvironment,
}

#[derive(Debug, Clone)]
pub struct AtomicStatus {
    pub rpm_ostree_available: bool,
    pub rollback_available: bool,
    pub pending_reboot: bool,
    pub updates_available: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct DevelopmentEnvironmentStatus {
    pub forge_dev_available: bool,
    pub rust_dev_available: bool,
    pub web_dev_available: bool,
    pub ai_dev_available: bool,
}

#[derive(Debug, Clone)]
pub struct ApplicationStatus {
    pub flatpak_available: bool,
    pub flatpak_count: Option<u32>,
    pub flatpak_updates_available: Option<u32>,
    pub podman_available: bool,
    pub distrobox_available: bool,
    pub distrobox_count: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    Host,
    Container,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeploymentStatus {
    Healthy,
    Unknown,
}

pub fn detect_system_dashboard() -> SystemDashboard {
    SystemDashboard {
        overview: SystemOverview {
            forge_os_version: "0.2.0".to_string(),
            fedora_version: read_fedora_version(),
            deployment_status: DeploymentStatus::Healthy,
            runtime_environment: detect_runtime_environment(),
        },
        atomic: AtomicStatus {
            rpm_ostree_available: command_exists("rpm-ostree"),
            rollback_available: rollback_available(),
            pending_reboot: pending_reboot(),
            updates_available: rpm_ostree_updates_available(),
        },
        development: DevelopmentEnvironmentStatus {
            forge_dev_available: distrobox_exists("forge-dev"),
            rust_dev_available: distrobox_exists("rust-dev"),
            web_dev_available: distrobox_exists("web-dev"),
            ai_dev_available: distrobox_exists("ai-dev"),
        },
        applications: ApplicationStatus {
            flatpak_available: command_exists("flatpak"),
            flatpak_count: flatpak_count(),
            flatpak_updates_available: flatpak_updates_available(),
            podman_available: command_exists("podman"),
            distrobox_available: command_exists("distrobox"),
            distrobox_count: distrobox_count(),
        },
    }
}

pub fn detect_system_status() -> SystemStatus {
    let dashboard = detect_system_dashboard();

    SystemStatus {
        fedora_version: dashboard.overview.fedora_version,
        runtime_environment: dashboard.overview.runtime_environment,
        rpm_ostree_available: dashboard.atomic.rpm_ostree_available,
        flatpak_available: dashboard.applications.flatpak_available,
        podman_available: dashboard.applications.podman_available,
        distrobox_available: dashboard.applications.distrobox_available,
    }
}

#[derive(Debug, Clone)]
pub struct SystemStatus {
    pub fedora_version: Option<String>,
    pub runtime_environment: RuntimeEnvironment,
    pub rpm_ostree_available: bool,
    pub flatpak_available: bool,
    pub podman_available: bool,
    pub distrobox_available: bool,
}

fn command_exists(command: &str) -> bool {
    Command::new("which")
        .arg(command)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

fn read_fedora_version() -> Option<String> {
    let contents = std::fs::read_to_string("/etc/os-release").ok()?;

    contents
        .lines()
        .find(|line| line.starts_with("PRETTY_NAME="))
        .map(|line| {
            line.trim_start_matches("PRETTY_NAME=")
                .trim_matches('"')
                .to_string()
        })
}
fn pending_reboot() -> bool {
    std::path::Path::new("/run/rpm-ostree/reboot-required").exists()
        || rpm_ostree_status_contains("staged")
}

fn rollback_available() -> bool {
    let output = Command::new("rpm-ostree").arg("status").output();

    let Ok(output) = output else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    stdout
        .lines()
        .filter(|line| line.trim_start().starts_with("Version:"))
        .count()
        > 1
}

fn rpm_ostree_updates_available() -> Option<u32> {
    let output = Command::new("rpm-ostree").arg("status").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    for line in stdout.lines() {
        let trimmed = line.trim();

        if let Some(diff) = trimmed.strip_prefix("Diff:") {
            let count = diff
                .split(',')
                .filter_map(|part| {
                    part.trim()
                        .split_whitespace()
                        .next()
                        .and_then(|value| value.parse::<u32>().ok())
                })
                .sum();

            return Some(count);
        }
    }

    Some(0)
}

fn rpm_ostree_status_contains(needle: &str) -> bool {
    let output = Command::new("rpm-ostree").arg("status").output();

    let Ok(output) = output else {
        return false;
    };

    if !output.status.success() {
        return false;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.to_lowercase().contains(&needle.to_lowercase())
}

fn flatpak_updates_available() -> Option<u32> {
    let output = Command::new("flatpak")
        .args(["remote-ls", "--updates"])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    Some(
        stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .count() as u32,
    )
}

fn distrobox_exists(name: &str) -> bool {
    let output = Command::new("distrobox").arg("list").output();

    let Ok(output) = output else {
        return false;
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().any(|line| line.contains(name))
}

fn distrobox_count() -> Option<u32> {
    let output = Command::new("distrobox").arg("list").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let count = stdout
        .lines()
        .filter(|line| line.contains('|'))
        .filter(|line| !line.contains("NAME"))
        .count();

    Some(count as u32)
}

fn flatpak_count() -> Option<u32> {
    let output = Command::new("flatpak").arg("list").output().ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().count() as u32)
}

fn detect_runtime_environment() -> RuntimeEnvironment {
    if std::path::Path::new("/run/.containerenv").exists()
        || std::path::Path::new("/.dockerenv").exists()
    {
        return RuntimeEnvironment::Container;
    }

    if command_exists("rpm-ostree") {
        return RuntimeEnvironment::Host;
    }

    RuntimeEnvironment::Unknown
}
