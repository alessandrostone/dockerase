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
