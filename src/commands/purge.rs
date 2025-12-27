use crate::display::{
    format_bytes, print_dry_run_header, print_error, print_info, print_space_saved, print_success,
    print_warning,
};
use crate::docker::Docker;
use dialoguer::Confirm;

pub fn run(force: bool, dry_run: bool) -> Result<(), String> {
    if !Docker::is_available() {
        print_error("Docker is not available. Is Docker running?");
        return Err("Docker not available".to_string());
    }

    if dry_run {
        print_dry_run_header();
    }

    let before = Docker::get_disk_usage()?;
    let reclaimable = before.total_reclaimable();

    if reclaimable == 0 {
        print_success("Nothing to clean up. Docker is already tidy!");
        return Ok(());
    }

    println!("Found {} of reclaimable space:", format_bytes(reclaimable));
    println!();

    let unused_images = before.images_count.saturating_sub(before.images_active);
    let stopped_containers = before
        .containers_count
        .saturating_sub(before.containers_active);
    let unused_volumes = before.volumes_count.saturating_sub(before.volumes_active);

    if unused_images > 0 {
        print_info(&format!(
            "{} dangling images ({})",
            unused_images,
            format_bytes(before.images_reclaimable)
        ));
    }
    if stopped_containers > 0 {
        print_info(&format!(
            "{} stopped containers ({})",
            stopped_containers,
            format_bytes(before.containers_reclaimable)
        ));
    }
    if unused_volumes > 0 {
        print_info(&format!(
            "{} unused volumes ({})",
            unused_volumes,
            format_bytes(before.volumes_reclaimable)
        ));
    }
    if before.build_cache_reclaimable > 0 {
        print_info(&format!(
            "Build cache ({})",
            format_bytes(before.build_cache_reclaimable)
        ));
    }

    println!();

    if dry_run {
        print_warning("Dry run - no changes made");
        return Ok(());
    }

    if !force {
        let confirm = Confirm::new()
            .with_prompt("Proceed with cleanup?")
            .default(false)
            .interact()
            .map_err(|e| e.to_string())?;

        if !confirm {
            print_warning("Aborted");
            return Ok(());
        }
    }

    println!();
    print_info("Removing stopped containers...");
    Docker::prune_containers()?;
    print_success("Containers cleaned");

    print_info("Removing dangling images...");
    Docker::prune_images(false)?;
    print_success("Images cleaned");

    print_info("Removing unused volumes...");
    Docker::prune_volumes()?;
    print_success("Volumes cleaned");

    print_info("Removing unused networks...");
    Docker::prune_networks()?;
    print_success("Networks cleaned");

    print_info("Clearing build cache...");
    Docker::prune_build_cache(false)?;
    print_success("Build cache cleared");

    let after = Docker::get_disk_usage()?;
    print_space_saved(before.total_size(), after.total_size());

    Ok(())
}
