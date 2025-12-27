use crate::resources::{Container, DiskUsage, Image, Network, Volume};
use std::process::Command;

pub struct Docker;

impl Docker {
    fn run_command(args: &[&str]) -> Result<String, String> {
        let output = Command::new("docker")
            .args(args)
            .output()
            .map_err(|e| format!("Failed to execute docker: {}", e))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.is_empty() {
                Ok(String::new())
            } else {
                Err(stderr.to_string())
            }
        }
    }

    pub fn is_available() -> bool {
        Command::new("docker")
            .arg("version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    pub fn get_disk_usage() -> Result<DiskUsage, String> {
        let output = Self::run_command(&["system", "df", "--format", "{{json .}}"])?;
        let mut usage = DiskUsage::default();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) {
                let type_name = entry["Type"].as_str().unwrap_or("");
                let size = parse_size(entry["Size"].as_str().unwrap_or("0"));
                let reclaimable_str = entry["Reclaimable"].as_str().unwrap_or("0");
                let reclaimable = parse_reclaimable(reclaimable_str);
                let count = entry["TotalCount"].as_i64().unwrap_or(0) as usize;
                let active = entry["Active"].as_i64().unwrap_or(0) as usize;

                match type_name {
                    "Images" => {
                        usage.images_size = size;
                        usage.images_reclaimable = reclaimable;
                        usage.images_count = count;
                        usage.images_active = active;
                    }
                    "Containers" => {
                        usage.containers_size = size;
                        usage.containers_reclaimable = reclaimable;
                        usage.containers_count = count;
                        usage.containers_active = active;
                    }
                    "Local Volumes" => {
                        usage.volumes_size = size;
                        usage.volumes_reclaimable = reclaimable;
                        usage.volumes_count = count;
                        usage.volumes_active = active;
                    }
                    "Build Cache" => {
                        usage.build_cache_size = size;
                        usage.build_cache_reclaimable = reclaimable;
                        usage.build_cache_count = count;
                        usage.build_cache_active = active;
                    }
                    _ => {}
                }
            }
        }
        Ok(usage)
    }

    pub fn list_images() -> Result<Vec<Image>, String> {
        let output = Self::run_command(&["images", "--format", "{{json .}}"])?;

        let mut images = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(img) = serde_json::from_str::<Image>(line) {
                images.push(img);
            }
        }
        Ok(images)
    }

    pub fn list_containers(all: bool) -> Result<Vec<Container>, String> {
        let mut args = vec!["ps", "--format", "{{json .}}"];
        if all {
            args.insert(1, "-a");
        }

        let output = Self::run_command(&args)?;
        let mut containers = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(c) = serde_json::from_str::<Container>(line) {
                containers.push(c);
            }
        }
        Ok(containers)
    }

    pub fn list_volumes() -> Result<Vec<Volume>, String> {
        let output = Self::run_command(&["volume", "ls", "--format", "{{json .}}"])?;
        let mut volumes = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(v) = serde_json::from_str::<Volume>(line) {
                volumes.push(v);
            }
        }
        Ok(volumes)
    }

    pub fn list_networks() -> Result<Vec<Network>, String> {
        let output = Self::run_command(&["network", "ls", "--format", "{{json .}}"])?;
        let mut networks = Vec::new();
        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(n) = serde_json::from_str::<Network>(line) {
                networks.push(n);
            }
        }
        Ok(networks)
    }

    pub fn prune_containers() -> Result<String, String> {
        Self::run_command(&["container", "prune", "-f"])
    }

    pub fn prune_images(all: bool) -> Result<String, String> {
        if all {
            Self::run_command(&["image", "prune", "-af"])
        } else {
            Self::run_command(&["image", "prune", "-f"])
        }
    }

    pub fn prune_volumes() -> Result<String, String> {
        Self::run_command(&["volume", "prune", "-f"])
    }

    pub fn prune_networks() -> Result<String, String> {
        Self::run_command(&["network", "prune", "-f"])
    }

    pub fn prune_build_cache(all: bool) -> Result<String, String> {
        if all {
            Self::run_command(&["builder", "prune", "-af"])
        } else {
            Self::run_command(&["builder", "prune", "-f"])
        }
    }

    pub fn stop_all_containers() -> Result<String, String> {
        let containers = Self::list_containers(false)?;
        if containers.is_empty() {
            return Ok(String::new());
        }
        let ids: Vec<&str> = containers.iter().map(|c| c.id.as_str()).collect();
        let mut args = vec!["stop"];
        args.extend(ids);
        Self::run_command(&args)
    }

    pub fn remove_all_containers() -> Result<String, String> {
        let containers = Self::list_containers(true)?;
        if containers.is_empty() {
            return Ok(String::new());
        }
        let ids: Vec<&str> = containers.iter().map(|c| c.id.as_str()).collect();
        let mut args = vec!["rm", "-f"];
        args.extend(ids);
        Self::run_command(&args)
    }

    pub fn remove_all_images() -> Result<String, String> {
        let images = Self::list_images()?;
        if images.is_empty() {
            return Ok(String::new());
        }
        let ids: Vec<&str> = images.iter().map(|i| i.id.as_str()).collect();
        let mut args = vec!["rmi", "-f"];
        args.extend(ids);
        Self::run_command(&args)
    }

    pub fn remove_all_volumes() -> Result<String, String> {
        let volumes = Self::list_volumes()?;
        if volumes.is_empty() {
            return Ok(String::new());
        }
        let names: Vec<&str> = volumes.iter().map(|v| v.name.as_str()).collect();
        let mut args = vec!["volume", "rm", "-f"];
        args.extend(names);
        Self::run_command(&args)
    }

    pub fn remove_custom_networks() -> Result<String, String> {
        let networks = Self::list_networks()?;
        let custom: Vec<&str> = networks
            .iter()
            .filter(|n| !["bridge", "host", "none"].contains(&n.name.as_str()))
            .map(|n| n.id.as_str())
            .collect();
        if custom.is_empty() {
            return Ok(String::new());
        }
        let mut args = vec!["network", "rm"];
        args.extend(custom);
        Self::run_command(&args)
    }
}

fn parse_size(s: &str) -> u64 {
    let s = s.trim();
    if s == "0" || s == "0B" || s.is_empty() {
        return 0;
    }

    let (num_str, multiplier) = if let Some(n) = s.strip_suffix("GB") {
        (n, 1_000_000_000.0)
    } else if let Some(n) = s.strip_suffix("MB") {
        (n, 1_000_000.0)
    } else if let Some(n) = s.strip_suffix("kB") {
        (n, 1_000.0)
    } else if let Some(n) = s.strip_suffix("KB") {
        (n, 1_000.0)
    } else if let Some(n) = s.strip_suffix("B") {
        (n, 1.0)
    } else {
        (s, 1.0)
    };

    let num: f64 = num_str.trim().parse().unwrap_or(0.0);
    (num * multiplier) as u64
}

fn parse_reclaimable(s: &str) -> u64 {
    // Format: "1.2GB (50%)" or just "1.2GB"
    let size_part = s.split('(').next().unwrap_or(s).trim();
    parse_size(size_part)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_zero() {
        assert_eq!(parse_size("0"), 0);
        assert_eq!(parse_size("0B"), 0);
        assert_eq!(parse_size(""), 0);
    }

    #[test]
    fn test_parse_size_bytes() {
        assert_eq!(parse_size("100B"), 100);
        assert_eq!(parse_size("1B"), 1);
    }

    #[test]
    fn test_parse_size_kilobytes() {
        assert_eq!(parse_size("1kB"), 1_000);
        assert_eq!(parse_size("1KB"), 1_000);
        assert_eq!(parse_size("1.5kB"), 1_500);
    }

    #[test]
    fn test_parse_size_megabytes() {
        assert_eq!(parse_size("1MB"), 1_000_000);
        assert_eq!(parse_size("1.5MB"), 1_500_000);
        assert_eq!(parse_size("100MB"), 100_000_000);
    }

    #[test]
    fn test_parse_size_gigabytes() {
        assert_eq!(parse_size("1GB"), 1_000_000_000);
        assert_eq!(parse_size("1.5GB"), 1_500_000_000);
        assert_eq!(parse_size("12.5GB"), 12_500_000_000);
    }

    #[test]
    fn test_parse_size_with_whitespace() {
        assert_eq!(parse_size("  1GB  "), 1_000_000_000);
        assert_eq!(parse_size(" 100MB "), 100_000_000);
    }

    #[test]
    fn test_parse_reclaimable_simple() {
        assert_eq!(parse_reclaimable("1GB"), 1_000_000_000);
        assert_eq!(parse_reclaimable("500MB"), 500_000_000);
    }

    #[test]
    fn test_parse_reclaimable_with_percentage() {
        assert_eq!(parse_reclaimable("1.2GB (50%)"), 1_200_000_000);
        assert_eq!(parse_reclaimable("500MB (100%)"), 500_000_000);
    }
}
