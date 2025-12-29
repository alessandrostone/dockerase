use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct CacheInfo {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    pub exists: bool,
    pub description: String,
}

impl CacheInfo {
    fn new(name: &str, path: PathBuf, description: &str) -> Self {
        let exists = path.exists();
        let size = if exists {
            dir_size(&path).unwrap_or(0)
        } else {
            0
        };

        Self {
            name: name.to_string(),
            path,
            size,
            exists,
            description: description.to_string(),
        }
    }
}

pub fn get_home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

pub fn discover_caches() -> Vec<CacheInfo> {
    let home = match get_home_dir() {
        Some(h) => h,
        None => return vec![],
    };

    let mut caches = vec![
        // Homebrew
        CacheInfo::new(
            "Homebrew",
            home.join("Library/Caches/Homebrew"),
            "Homebrew package downloads and cache",
        ),
        // npm
        CacheInfo::new(
            "npm",
            home.join(".npm/_cacache"),
            "Node.js npm package cache",
        ),
        // Yarn
        CacheInfo::new(
            "Yarn",
            home.join("Library/Caches/Yarn"),
            "Yarn package cache",
        ),
        // pnpm
        CacheInfo::new(
            "pnpm",
            home.join("Library/pnpm/store"),
            "pnpm package store",
        ),
        // Cargo registry
        CacheInfo::new(
            "Cargo Registry",
            home.join(".cargo/registry"),
            "Rust crates registry cache",
        ),
        // Cargo git
        CacheInfo::new(
            "Cargo Git",
            home.join(".cargo/git"),
            "Rust git dependencies cache",
        ),
        // pip
        CacheInfo::new(
            "pip",
            home.join("Library/Caches/pip"),
            "Python pip package cache",
        ),
        // Xcode DerivedData
        CacheInfo::new(
            "Xcode DerivedData",
            home.join("Library/Developer/Xcode/DerivedData"),
            "Xcode build artifacts and indexes",
        ),
        // Xcode Archives
        CacheInfo::new(
            "Xcode Archives",
            home.join("Library/Developer/Xcode/Archives"),
            "Xcode archived builds",
        ),
        // CocoaPods
        CacheInfo::new(
            "CocoaPods",
            home.join("Library/Caches/CocoaPods"),
            "CocoaPods spec and pod cache",
        ),
        // Gradle
        CacheInfo::new("Gradle", home.join(".gradle/caches"), "Gradle build cache"),
        // Maven
        CacheInfo::new(
            "Maven",
            home.join(".m2/repository"),
            "Maven local repository",
        ),
        // Go modules
        CacheInfo::new(
            "Go Modules",
            home.join("go/pkg/mod/cache"),
            "Go module cache",
        ),
        // Composer (PHP)
        CacheInfo::new(
            "Composer",
            home.join(".composer/cache"),
            "PHP Composer cache",
        ),
        // Trash
        CacheInfo::new("Trash", home.join(".Trash"), "Files in Trash"),
    ];

    // Filter to only existing caches with size > 0
    caches.retain(|c| c.exists && c.size > 0);

    // Sort by size descending
    caches.sort_by(|a, b| b.size.cmp(&a.size));

    caches
}

pub fn purge_cache(cache: &CacheInfo) -> Result<u64, String> {
    if !cache.exists {
        return Ok(0);
    }

    let size = cache.size;

    if cache.path.is_dir() {
        // Special handling for Trash - remove contents but not the directory itself
        // macOS protects the .Trash directory from being removed
        if cache.name == "Trash" {
            for entry in fs::read_dir(&cache.path)
                .map_err(|e| format!("Failed to read {}: {}", cache.path.display(), e))?
            {
                let entry =
                    entry.map_err(|e| format!("Failed to read entry in Trash: {}", e))?;
                let path = entry.path();
                if path.is_dir() {
                    fs::remove_dir_all(&path)
                        .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
                } else {
                    fs::remove_file(&path)
                        .map_err(|e| format!("Failed to remove {}: {}", path.display(), e))?;
                }
            }
        } else {
            fs::remove_dir_all(&cache.path)
                .map_err(|e| format!("Failed to remove {}: {}", cache.path.display(), e))?;

            // Recreate empty directory (some tools expect it to exist)
            fs::create_dir_all(&cache.path).ok();
        }
    } else if cache.path.is_file() {
        fs::remove_file(&cache.path)
            .map_err(|e| format!("Failed to remove {}: {}", cache.path.display(), e))?;
    }

    Ok(size)
}

fn dir_size(path: &PathBuf) -> Result<u64, std::io::Error> {
    let mut size = 0;

    if path.is_file() {
        return Ok(fs::metadata(path)?.len());
    }

    if path.is_dir() {
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                size += fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            } else if path.is_dir() {
                size += dir_size(&path).unwrap_or(0);
            }
        }
    }

    Ok(size)
}
