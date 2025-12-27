use crate::display::{format_bytes, print_error, print_info, print_space_saved, print_success, print_warning};
use crate::docker::Docker;
use colored::Colorize;
use dialoguer::MultiSelect;

pub fn run(force: bool, dry_run: bool) -> Result<(), String> {
    if !Docker::is_available() {
        print_error("Docker is not available. Is Docker running?");
        return Err("Docker not available".to_string());
    }

    let before = Docker::get_disk_usage()?;

    // Gather all purgeable items
    let mut items: Vec<PurgeItem> = Vec::new();

    // Stopped containers
    let containers = Docker::list_containers(true)?;
    let stopped: Vec<_> = containers.iter().filter(|c| !c.is_running()).collect();
    if !stopped.is_empty() {
        items.push(PurgeItem {
            label: format!(
                "Stopped containers ({} containers)",
                stopped.len()
            ),
            category: Category::Containers,
        });
    }

    // Dangling images
    let images = Docker::list_images()?;
    let dangling_count = before.images_count.saturating_sub(before.images_active);
    if dangling_count > 0 || before.images_reclaimable > 0 {
        items.push(PurgeItem {
            label: format!(
                "Dangling images ({} images, {})",
                dangling_count,
                format_bytes(before.images_reclaimable)
            ),
            category: Category::Images,
        });
    }

    // All images (for more aggressive cleanup)
    if !images.is_empty() {
        items.push(PurgeItem {
            label: format!(
                "ALL images ({} images, {})",
                images.len(),
                format_bytes(before.images_size)
            ),
            category: Category::AllImages,
        });
    }

    // Unused volumes
    let volumes = Docker::list_volumes()?;
    let unused_volumes = before.volumes_count.saturating_sub(before.volumes_active);
    if unused_volumes > 0 || before.volumes_reclaimable > 0 {
        items.push(PurgeItem {
            label: format!(
                "Unused volumes ({} volumes, {})",
                unused_volumes,
                format_bytes(before.volumes_reclaimable)
            ),
            category: Category::Volumes,
        });
    }

    // All volumes
    if !volumes.is_empty() {
        items.push(PurgeItem {
            label: format!(
                "ALL volumes ({} volumes, {})",
                volumes.len(),
                format_bytes(before.volumes_size)
            ),
            category: Category::AllVolumes,
        });
    }

    // Unused networks
    let networks = Docker::list_networks()?;
    let custom_networks: Vec<_> = networks.iter().filter(|n| !n.is_default()).collect();
    if !custom_networks.is_empty() {
        items.push(PurgeItem {
            label: format!("Custom networks ({} networks)", custom_networks.len()),
            category: Category::Networks,
        });
    }

    // Build cache
    if before.build_cache_size > 0 {
        items.push(PurgeItem {
            label: format!("Build cache ({})", format_bytes(before.build_cache_size)),
            category: Category::BuildCache,
        });
    }

    if items.is_empty() {
        print_success("Nothing to clean up. Docker is already tidy!");
        return Ok(());
    }

    println!("{}", "Select items to purge:".bold());
    println!("{}", "(Use space to select, enter to confirm)".dimmed());
    println!();

    let labels: Vec<&str> = items.iter().map(|i| i.label.as_str()).collect();

    let selections = if force {
        // If force, select all by default
        (0..items.len()).collect()
    } else {
        MultiSelect::new()
            .items(&labels)
            .interact()
            .map_err(|e| e.to_string())?
    };

    if selections.is_empty() {
        print_warning("Nothing selected. Aborting.");
        return Ok(());
    }

    println!();
    println!("{}", "Selected for removal:".bold());
    for &idx in &selections {
        print_info(&items[idx].label);
    }
    println!();

    if dry_run {
        print_warning("Dry run - no changes made");
        return Ok(());
    }

    // Execute selected purges
    let selected_categories: Vec<Category> = selections.iter().map(|&i| items[i].category).collect();

    // Check for conflicts (can't do both dangling and all images)
    let has_all_images = selected_categories.contains(&Category::AllImages);
    let has_all_volumes = selected_categories.contains(&Category::AllVolumes);

    if selected_categories.contains(&Category::Containers) {
        print_info("Removing stopped containers...");
        Docker::prune_containers()?;
        print_success("Containers removed");
    }

    if has_all_images {
        print_info("Removing ALL images...");
        Docker::prune_images(true)?;
        print_success("All images removed");
    } else if selected_categories.contains(&Category::Images) {
        print_info("Removing dangling images...");
        Docker::prune_images(false)?;
        print_success("Dangling images removed");
    }

    if has_all_volumes {
        print_info("Removing ALL volumes...");
        Docker::remove_all_volumes()?;
        print_success("All volumes removed");
    } else if selected_categories.contains(&Category::Volumes) {
        print_info("Removing unused volumes...");
        Docker::prune_volumes()?;
        print_success("Unused volumes removed");
    }

    if selected_categories.contains(&Category::Networks) {
        print_info("Removing custom networks...");
        Docker::prune_networks()?;
        print_success("Networks removed");
    }

    if selected_categories.contains(&Category::BuildCache) {
        print_info("Clearing build cache...");
        Docker::prune_build_cache(true)?;
        print_success("Build cache cleared");
    }

    let after = Docker::get_disk_usage()?;
    print_space_saved(before.total_size(), after.total_size());

    Ok(())
}

#[derive(Clone, Copy, PartialEq)]
enum Category {
    Containers,
    Images,
    AllImages,
    Volumes,
    AllVolumes,
    Networks,
    BuildCache,
}

struct PurgeItem {
    label: String,
    category: Category,
}
