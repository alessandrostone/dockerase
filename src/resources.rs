use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Image {
    #[serde(rename = "ID")]
    pub id: String,
    #[allow(dead_code)]
    #[serde(rename = "Repository")]
    pub repository: String,
    #[allow(dead_code)]
    #[serde(rename = "Tag")]
    pub tag: String,
    #[allow(dead_code)]
    #[serde(rename = "Size")]
    pub size: String,
    #[allow(dead_code)]
    #[serde(rename = "CreatedAt", default)]
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct Container {
    #[serde(rename = "ID")]
    pub id: String,
    #[allow(dead_code)]
    #[serde(rename = "Names")]
    pub names: String,
    #[allow(dead_code)]
    #[serde(rename = "Image")]
    pub image: String,
    #[serde(rename = "State")]
    pub state: String,
    #[allow(dead_code)]
    #[serde(rename = "Status")]
    pub status: String,
    #[allow(dead_code)]
    #[serde(rename = "Size", default)]
    pub size: String,
}

impl Container {
    pub fn is_running(&self) -> bool {
        self.state == "running"
    }
}

#[derive(Debug, Deserialize)]
pub struct Volume {
    #[serde(rename = "Name")]
    pub name: String,
    #[allow(dead_code)]
    #[serde(rename = "Driver")]
    pub driver: String,
    #[allow(dead_code)]
    #[serde(rename = "Mountpoint", default)]
    pub mountpoint: String,
}

#[derive(Debug, Deserialize)]
pub struct Network {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[allow(dead_code)]
    #[serde(rename = "Driver")]
    pub driver: String,
    #[allow(dead_code)]
    #[serde(rename = "Scope", default)]
    pub scope: String,
}

impl Network {
    pub fn is_default(&self) -> bool {
        matches!(self.name.as_str(), "bridge" | "host" | "none")
    }
}

#[derive(Debug, Default)]
pub struct DiskUsage {
    pub images_size: u64,
    pub images_reclaimable: u64,
    pub images_count: usize,
    pub images_active: usize,

    pub containers_size: u64,
    pub containers_reclaimable: u64,
    pub containers_count: usize,
    pub containers_active: usize,

    pub volumes_size: u64,
    pub volumes_reclaimable: u64,
    pub volumes_count: usize,
    pub volumes_active: usize,

    pub build_cache_size: u64,
    pub build_cache_reclaimable: u64,
    pub build_cache_count: usize,
    pub build_cache_active: usize,
}

impl DiskUsage {
    pub fn total_size(&self) -> u64 {
        self.images_size + self.containers_size + self.volumes_size + self.build_cache_size
    }

    pub fn total_reclaimable(&self) -> u64 {
        self.images_reclaimable
            + self.containers_reclaimable
            + self.volumes_reclaimable
            + self.build_cache_reclaimable
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_container(state: &str) -> Container {
        Container {
            id: "abc123".to_string(),
            names: "test".to_string(),
            image: "alpine".to_string(),
            state: state.to_string(),
            status: "Up 1 hour".to_string(),
            size: "0B".to_string(),
        }
    }

    fn make_network(name: &str) -> Network {
        Network {
            id: "net123".to_string(),
            name: name.to_string(),
            driver: "bridge".to_string(),
            scope: "local".to_string(),
        }
    }

    #[test]
    fn test_container_is_running() {
        let running = make_container("running");
        let exited = make_container("exited");
        let created = make_container("created");

        assert!(running.is_running());
        assert!(!exited.is_running());
        assert!(!created.is_running());
    }

    #[test]
    fn test_network_is_default() {
        assert!(make_network("bridge").is_default());
        assert!(make_network("host").is_default());
        assert!(make_network("none").is_default());
        assert!(!make_network("my-network").is_default());
        assert!(!make_network("custom_net").is_default());
    }

    #[test]
    fn test_disk_usage_total_size() {
        let usage = DiskUsage {
            images_size: 1_000_000_000,
            containers_size: 500_000_000,
            volumes_size: 250_000_000,
            build_cache_size: 100_000_000,
            ..Default::default()
        };

        assert_eq!(usage.total_size(), 1_850_000_000);
    }

    #[test]
    fn test_disk_usage_total_reclaimable() {
        let usage = DiskUsage {
            images_reclaimable: 800_000_000,
            containers_reclaimable: 400_000_000,
            volumes_reclaimable: 200_000_000,
            build_cache_reclaimable: 100_000_000,
            ..Default::default()
        };

        assert_eq!(usage.total_reclaimable(), 1_500_000_000);
    }

    #[test]
    fn test_disk_usage_default() {
        let usage = DiskUsage::default();

        assert_eq!(usage.total_size(), 0);
        assert_eq!(usage.total_reclaimable(), 0);
    }
}
