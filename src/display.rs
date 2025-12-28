use crate::resources::DiskUsage;
use bytesize::ByteSize;
use colored::Colorize;
use comfy_table::{presets::UTF8_BORDERS_ONLY, Table};

pub fn format_bytes(bytes: u64) -> String {
    ByteSize::b(bytes).to_string()
}

pub fn print_header() {
    println!("{}", "Docker Space Usage".bold().cyan());
    println!("{}", "═".repeat(50).dimmed());
}

pub fn print_disk_usage(usage: &DiskUsage) {
    let mut table = Table::new();
    table.load_preset(UTF8_BORDERS_ONLY);
    table.set_header(vec!["TYPE", "TOTAL", "RECLAIMABLE"]);

    table.add_row(vec![
        "Images".to_string(),
        format_bytes(usage.images_size),
        format!(
            "{} ({} unused)",
            format_bytes(usage.images_reclaimable),
            usage.images_count.saturating_sub(usage.images_active)
        ),
    ]);

    table.add_row(vec![
        "Containers".to_string(),
        format_bytes(usage.containers_size),
        format!(
            "{} ({} stopped)",
            format_bytes(usage.containers_reclaimable),
            usage
                .containers_count
                .saturating_sub(usage.containers_active)
        ),
    ]);

    table.add_row(vec![
        "Volumes".to_string(),
        format_bytes(usage.volumes_size),
        format!(
            "{} ({} unused)",
            format_bytes(usage.volumes_reclaimable),
            usage.volumes_count.saturating_sub(usage.volumes_active)
        ),
    ]);

    table.add_row(vec![
        "Build Cache".to_string(),
        format_bytes(usage.build_cache_size),
        format_bytes(usage.build_cache_reclaimable),
    ]);

    println!("{table}");
    println!();
    println!(
        "{} {}",
        "Total Reclaimable:".bold(),
        format_bytes(usage.total_reclaimable()).green().bold()
    );
}

pub fn print_footer() {
    println!();
    println!("{}", "─".repeat(50).dimmed());
    println!("Run {} to clean up safely", "dockerase purge".cyan().bold());
    println!(
        "Run {} to remove everything",
        "dockerase --nuclear".red().bold()
    );
}

pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

pub fn print_info(message: &str) {
    println!("{} {}", "→".blue().bold(), message);
}

pub fn print_space_saved(before: u64, after: u64) {
    let saved = before.saturating_sub(after);
    if saved > 0 {
        println!();
        println!(
            "{} {} {}",
            "Space freed:".bold(),
            format_bytes(saved).green().bold(),
            format!("({} → {})", format_bytes(before), format_bytes(after)).dimmed()
        );
    }
}

pub fn print_nuclear_warning() {
    println!();
    println!(
        "{}",
        "╔══════════════════════════════════════════════════════╗"
            .red()
            .bold()
    );
    println!(
        "{}",
        "║            ⚠️  NUCLEAR MODE WARNING ⚠️                 ║"
            .red()
            .bold()
    );
    println!(
        "{}",
        "╠══════════════════════════════════════════════════════╣"
            .red()
            .bold()
    );
    println!(
        "{}",
        "║  This will PERMANENTLY DELETE:                       ║".red()
    );
    println!(
        "{}",
        "║  • ALL containers (running and stopped)              ║".red()
    );
    println!(
        "{}",
        "║  • ALL images                                        ║".red()
    );
    println!(
        "{}",
        "║  • ALL volumes (including data!)                     ║".red()
    );
    println!(
        "{}",
        "║  • ALL custom networks                               ║".red()
    );
    println!(
        "{}",
        "║  • ALL build cache                                   ║".red()
    );
    println!(
        "{}",
        "╚══════════════════════════════════════════════════════╝"
            .red()
            .bold()
    );
    println!();
}

pub fn print_dry_run_header() {
    println!("{}", "[DRY RUN] No changes will be made".yellow().bold());
    println!();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes_zero() {
        assert_eq!(format_bytes(0), "0 B");
    }

    #[test]
    fn test_format_bytes_small() {
        let result = format_bytes(500);
        assert!(result.contains("500"));
        assert!(result.contains("B"));
    }

    #[test]
    fn test_format_bytes_kilobytes() {
        let result = format_bytes(5_000);
        assert!(result.contains("KB"));
    }

    #[test]
    fn test_format_bytes_megabytes() {
        let result = format_bytes(5_000_000);
        assert!(result.contains("MB"));
    }

    #[test]
    fn test_format_bytes_gigabytes() {
        let result = format_bytes(5_000_000_000);
        assert!(result.contains("GB"));
    }

    #[test]
    fn test_format_bytes_is_human_readable() {
        // Large numbers should not be displayed as raw bytes
        let result = format_bytes(1_000_000_000);
        assert!(!result.contains("1000000000"));
        assert!(result.contains("MB") || result.contains("GB"));
    }
}
