use crate::display::{
    format_bytes, print_dry_run_header, print_error, print_info, print_nuclear_warning,
    print_space_saved, print_success, print_warning,
};
use crate::docker::Docker;
use colored::Colorize;
use dialoguer::Confirm;

pub fn run(force: bool, dry_run: bool) -> Result<(), String> {
    if !Docker::is_available() {
        print_error("Docker is not available. Is Docker running?");
        return Err("Docker not available".to_string());
    }

    if dry_run {
        print_dry_run_header();
    }

    print_nuclear_warning();

    let before = Docker::get_disk_usage()?;
    let containers = Docker::list_containers(true)?;
    let images = Docker::list_images()?;
    let volumes = Docker::list_volumes()?;
    let networks = Docker::list_networks()?;
    let custom_networks: Vec<_> = networks.iter().filter(|n| !n.is_default()).collect();

    println!("This will remove:");
    print_info(&format!("{} containers", containers.len()));
    print_info(&format!("{} images", images.len()));
    print_info(&format!("{} volumes", volumes.len()));
    print_info(&format!("{} custom networks", custom_networks.len()));
    print_info("All build cache");
    println!();
    println!(
        "Total space to free: {}",
        format_bytes(before.total_size()).green().bold()
    );
    println!();

    if dry_run {
        print_warning("Dry run - no changes made");
        return Ok(());
    }

    if !force {
        println!(
            "{}",
            "Type 'yes' to confirm complete Docker cleanup:"
                .red()
                .bold()
        );
        let confirm = Confirm::new()
            .with_prompt("Are you absolutely sure?")
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;

        if !confirm {
            print_warning("Aborted - no changes made");
            return Ok(());
        }
    }

    println!();

    // Stop running containers first
    let running: Vec<_> = containers.iter().filter(|c| c.is_running()).collect();
    if !running.is_empty() {
        print_info(&format!("Stopping {} running containers...", running.len()));
        Docker::stop_all_containers()?;
        print_success("Containers stopped");
    }

    // Remove all containers
    if !containers.is_empty() {
        print_info(&format!("Removing {} containers...", containers.len()));
        Docker::remove_all_containers()?;
        print_success("Containers removed");
    }

    // Remove all images
    if !images.is_empty() {
        print_info(&format!("Removing {} images...", images.len()));
        Docker::remove_all_images()?;
        print_success("Images removed");
    }

    // Remove all volumes
    if !volumes.is_empty() {
        print_info(&format!("Removing {} volumes...", volumes.len()));
        Docker::remove_all_volumes()?;
        print_success("Volumes removed");
    }

    // Remove custom networks
    if !custom_networks.is_empty() {
        print_info(&format!(
            "Removing {} custom networks...",
            custom_networks.len()
        ));
        Docker::remove_custom_networks()?;
        print_success("Networks removed");
    }

    // Clear all build cache
    print_info("Clearing all build cache...");
    Docker::prune_build_cache(true)?;
    print_success("Build cache cleared");

    let after = Docker::get_disk_usage()?;
    print_space_saved(before.total_size(), after.total_size());

    println!();
    print_success("Nuclear cleanup complete. Docker is now empty.");

    Ok(())
}
