use clap::{Parser, Subcommand};
use forge_welcome_core::detect_development_pack_status;
use forge_welcome_core::{
    Pack, RuntimeEnvironment, create_execution_plan, create_install_plan, detect_system_dashboard,
    load_pack_from_file, load_packs_from_dir,
};
//ToDo: remove use forge_welcome_core::create_install_plan;

const MANIFEST_DIR: &str = "manifests";

#[derive(Debug, Parser)]
#[command(name = "forge-welcome")]
#[command(about = "Forge OS Welcome Center CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// List all available Forge OS packs
    ListPacks,

    /// Show details for a specific pack
    ShowPack {
        /// Pack id, for example: developer-pack
        id: String,
    },

    InstallPlan {
        /// Pack id, for example: developer-pack
        id: String,
    },

    /// Preview execution steps for installing a pack
    Install {
        /// Pack id, for example: developer-pack
        id: String,

        /// Preview commands without executing them
        #[arg(long)]
        dry_run: bool,
    },

    /// Show development pack status
    DevStatus,

    Status,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::ListPacks => list_packs(),
        Commands::ShowPack { id } => show_pack(&id),
        Commands::Status => show_status(),
        Commands::InstallPlan { id } => show_install_plan(&id),
        Commands::Install { id, dry_run } => install_pack(&id, dry_run),
        Commands::DevStatus => show_dev_status(),
    };

    if let Err(error) = result {
        eprintln!("Error: {error}");
        std::process::exit(1);
    }
}

fn list_packs() -> Result<(), Box<dyn std::error::Error>> {
    let packs = load_packs_from_dir(MANIFEST_DIR)?;

    for pack in packs {
        println!("{} ({})", pack.name, pack.id);
        println!("  {}", pack.description);
        println!("  Category: {:?}", pack.category);
        println!();
    }

    Ok(())
}

fn show_pack(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{MANIFEST_DIR}/{id}.yaml");
    let pack = load_pack_from_file(path)?;

    print_pack_details(&pack);

    Ok(())
}

fn print_pack_details(pack: &Pack) {
    println!("{} ({})", pack.name, pack.id);
    println!("{}", pack.description);
    println!("Category: {:?}", pack.category);
    println!("Requires reboot: {}", pack.requires_reboot);
    println!();

    println!("Host packages:");
    print_list(&pack.host_packages);

    println!();
    println!("Flatpaks:");
    print_list(&pack.flatpaks);

    println!();
    println!("Distrobox packages:");
    print_list(&pack.distrobox_packages);
}

fn print_list(items: &[String]) {
    if items.is_empty() {
        println!("  none");
        return;
    }

    for item in items {
        println!("  - {item}");
    }
}

fn show_status() -> Result<(), Box<dyn std::error::Error>> {
    let dashboard = detect_system_dashboard();

    println!("Forge Welcome System Dashboard");
    println!();

    println!("System Overview");
    println!(
        "  Forge OS Version: {}",
        dashboard.overview.forge_os_version
    );
    println!(
        "  Fedora Version: {}",
        dashboard
            .overview
            .fedora_version
            .unwrap_or_else(|| "Unknown".to_string())
    );
    println!(
        "  Runtime Environment: {}",
        runtime_environment_label(dashboard.overview.runtime_environment)
    );
    println!(
        "  Deployment Status: {:?}",
        dashboard.overview.deployment_status
    );
    println!();

    println!("Atomic Status");
    println!(
        "  rpm-ostree: {}",
        available(dashboard.atomic.rpm_ostree_available)
    );
    println!(
        "  Rollback Available: {}",
        yes_no(dashboard.atomic.rollback_available)
    );
    println!(
        "  Pending Reboot: {}",
        yes_no(dashboard.atomic.pending_reboot)
    );
    println!(
        "  Updates Available: {}",
        optional_count(dashboard.atomic.updates_available)
    );
    println!();

    println!("Development Environments");
    println!(
        "  forge-dev: {}",
        available(dashboard.development.forge_dev_available)
    );
    println!(
        "  rust-dev: {}",
        available(dashboard.development.rust_dev_available)
    );
    println!(
        "  web-dev: {}",
        available(dashboard.development.web_dev_available)
    );
    println!(
        "  ai-dev: {}",
        available(dashboard.development.ai_dev_available)
    );
    println!();

    println!("Applications");
    println!(
        "  Flatpak: {}",
        available(dashboard.applications.flatpak_available)
    );
    println!(
        "  Flatpak Count: {}",
        optional_count(dashboard.applications.flatpak_count)
    );
    println!(
        "  Flatpak Updates: {}",
        optional_count(dashboard.applications.flatpak_updates_available)
    );
    println!(
        "  Podman: {}",
        available(dashboard.applications.podman_available)
    );
    println!(
        "  Distrobox: {}",
        available(dashboard.applications.distrobox_available)
    );
    println!(
        "  Distrobox Count: {}",
        optional_count(dashboard.applications.distrobox_count)
    );

    Ok(())
}

fn available(value: bool) -> &'static str {
    if value { "Available" } else { "Not Available" }
}

fn runtime_environment_label(environment: RuntimeEnvironment) -> &'static str {
    match environment {
        RuntimeEnvironment::Host => "Host",
        RuntimeEnvironment::Container => "Container",
        RuntimeEnvironment::Unknown => "Unknown",
    }
}

fn yes_no(value: bool) -> &'static str {
    if value { "Yes" } else { "No" }
}

fn optional_count(value: Option<u32>) -> String {
    value
        .map(|count| count.to_string())
        .unwrap_or_else(|| "Unknown".to_string())
}

fn show_install_plan(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = format!("{MANIFEST_DIR}/{id}.yaml");
    let pack = load_pack_from_file(path)?;
    let plan = create_install_plan(&pack);

    println!("Install Plan: {} ({})", plan.pack_name, plan.pack_id);
    println!();

    println!("Host packages:");
    print_list(&plan.host_packages);

    println!();
    println!("Flatpaks:");
    print_list(&plan.flatpaks);

    println!();
    println!("Distrobox packages:");
    print_list(&plan.distrobox_packages);

    println!();
    println!("Requires reboot: {}", plan.requires_reboot);

    Ok(())
}

fn install_pack(id: &str, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    if !dry_run {
        println!("Real installation is not implemented yet.");
        println!("Use --dry-run to preview the install commands.");
        return Ok(());
    }

    let path = format!("{MANIFEST_DIR}/{id}.yaml");
    let pack = load_pack_from_file(path)?;
    let install_plan = create_install_plan(&pack);
    let execution_plan = create_execution_plan(&install_plan, dry_run);

    println!(
        "Execution Plan: {} ({})",
        execution_plan.pack_name, execution_plan.pack_id
    );
    println!("Dry run: {}", execution_plan.dry_run);
    println!();

    if execution_plan.steps.is_empty() {
        println!("No installation steps required.");
    } else {
        for step in execution_plan.steps {
            println!("{}", step.description);
            println!("  {}", step.command);
            println!();
        }
    }

    println!("Requires reboot: {}", execution_plan.requires_reboot);

    Ok(())
}

fn show_dev_status() -> Result<(), Box<dyn std::error::Error>> {
    let status = detect_development_pack_status();

    println!("Forge Welcome Development Pack Status");
    println!();

    for tool in status.tools {
        let marker = if tool.installed {
            "Available"
        } else {
            "Missing"
        };
        let version = tool.version.unwrap_or_else(|| "Unknown".to_string());

        println!("{}: {}", tool.name, marker);
        println!("  Command: {}", tool.command);
        println!("  Version: {}", version);
        println!();
    }

    Ok(())
}
