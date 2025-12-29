use crate::display::{format_bytes, print_error, print_info, print_success, print_warning};
use crate::system::{discover_caches, purge_cache, CacheInfo};
use colored::Colorize;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};
use dialoguer::MultiSelect;

pub fn list() -> Result<(), String> {
    let caches = discover_caches();

    if caches.is_empty() {
        print_success("No purgeable caches found. System is clean!");
        return Ok(());
    }

    println!("{}", "System Caches".bold().cyan());
    println!("{}", "═".repeat(50).dimmed());
    println!();

    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec!["CACHE", "SIZE", "PATH"]);

    let mut total_size = 0u64;

    for cache in &caches {
        total_size += cache.size;
        table.add_row(vec![
            cache.name.clone(),
            format_bytes(cache.size),
            cache.path.display().to_string(),
        ]);
    }

    println!("{table}");
    println!();
    println!(
        "{} {}",
        "Total Purgeable:".bold(),
        format_bytes(total_size).green().bold()
    );
    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!(
        "Run {} to interactively select caches to purge",
        "dockerase system select".cyan().bold()
    );
    println!(
        "Run {} to purge all caches",
        "dockerase system purge".cyan().bold()
    );

    Ok(())
}

pub fn purge(force: bool, dry_run: bool, interactive: bool) -> Result<(), String> {
    let caches = discover_caches();

    if caches.is_empty() {
        print_success("No purgeable caches found. System is clean!");
        return Ok(());
    }

    if dry_run {
        println!("{}", "[DRY RUN] No changes will be made".yellow().bold());
        println!();
    }

    let selections: Vec<usize> = if interactive {
        println!("{}", "Select caches to purge:".bold());
        println!("{}", "(Use space to select, enter to confirm)".dimmed());
        println!();

        let labels: Vec<String> = caches
            .iter()
            .map(|c| format!("{} ({})", c.name, format_bytes(c.size)))
            .collect();

        let label_refs: Vec<&str> = labels.iter().map(|s| s.as_str()).collect();

        if force {
            (0..caches.len()).collect()
        } else {
            MultiSelect::new()
                .items(&label_refs)
                .interact()
                .map_err(|e| e.to_string())?
        }
    } else {
        // Non-interactive: select all
        if !force && !dry_run {
            use dialoguer::Confirm;
            let total: u64 = caches.iter().map(|c| c.size).sum();
            let confirm = Confirm::new()
                .with_prompt(format!(
                    "Purge all {} caches ({})? This cannot be undone",
                    caches.len(),
                    format_bytes(total)
                ))
                .default(false)
                .interact()
                .map_err(|e| e.to_string())?;

            if !confirm {
                print_warning("Aborted");
                return Ok(());
            }
        }
        (0..caches.len()).collect()
    };

    if selections.is_empty() {
        print_warning("Nothing selected. Aborting.");
        return Ok(());
    }

    println!();
    println!("{}", "Selected for removal:".bold());
    let selected_caches: Vec<&CacheInfo> = selections.iter().map(|&i| &caches[i]).collect();

    for cache in &selected_caches {
        print_info(&format!(
            "{} ({}) - {}",
            cache.name,
            format_bytes(cache.size),
            cache.description
        ));
    }
    println!();

    if dry_run {
        print_warning("Dry run - no changes made");
        return Ok(());
    }

    let mut total_freed = 0u64;

    for cache in selected_caches {
        print_info(&format!("Removing {}...", cache.name));
        match purge_cache(cache) {
            Ok(size) => {
                total_freed += size;
                print_success(&format!("{} cleared", cache.name));
            }
            Err(e) => {
                print_error(&format!("Failed to clear {}: {}", cache.name, e));
            }
        }
    }

    if total_freed > 0 {
        println!();
        println!(
            "{} {}",
            "Space freed:".bold(),
            format_bytes(total_freed).green().bold()
        );
    }

    Ok(())
}
