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
                let entry = entry.map_err(|e| format!("Failed to read entry in Trash: {}", e))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_get_home_dir() {
        let home = get_home_dir();
        assert!(home.is_some());
        assert!(home.unwrap().exists());
    }

    #[test]
    fn test_cache_info_non_existent_path() {
        let cache = CacheInfo::new(
            "TestCache",
            PathBuf::from("/nonexistent/path/that/does/not/exist"),
            "Test description",
        );

        assert_eq!(cache.name, "TestCache");
        assert!(!cache.exists);
        assert_eq!(cache.size, 0);
        assert_eq!(cache.description, "Test description");
    }

    #[test]
    fn test_cache_info_existing_path() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "Hello, world!").unwrap();

        let cache = CacheInfo::new("TestCache", dir.path().to_path_buf(), "Test");

        assert_eq!(cache.name, "TestCache");
        assert!(cache.exists);
        assert!(cache.size > 0);
    }

    #[test]
    fn test_dir_size_empty_dir() {
        let dir = tempdir().unwrap();
        let size = dir_size(&dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 0);
    }

    #[test]
    fn test_dir_size_with_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "12345").unwrap(); // 5 bytes

        let size = dir_size(&dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 5);
    }

    #[test]
    fn test_dir_size_nested() {
        let dir = tempdir().unwrap();

        // Create nested structure
        let subdir = dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file1 = dir.path().join("file1.txt");
        let file2 = subdir.join("file2.txt");

        let mut f1 = File::create(&file1).unwrap();
        write!(f1, "abc").unwrap(); // 3 bytes

        let mut f2 = File::create(&file2).unwrap();
        write!(f2, "defgh").unwrap(); // 5 bytes

        let size = dir_size(&dir.path().to_path_buf()).unwrap();
        assert_eq!(size, 8);
    }

    #[test]
    fn test_dir_size_single_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "1234567890").unwrap(); // 10 bytes

        let size = dir_size(&file_path).unwrap();
        assert_eq!(size, 10);
    }

    #[test]
    fn test_purge_cache_non_existent() {
        let cache = CacheInfo {
            name: "Test".to_string(),
            path: PathBuf::from("/nonexistent"),
            size: 0,
            exists: false,
            description: "Test".to_string(),
        };

        let result = purge_cache(&cache);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 0);
    }

    #[test]
    fn test_purge_cache_directory() {
        let dir = tempdir().unwrap();
        let cache_dir = dir.path().join("cache");
        fs::create_dir(&cache_dir).unwrap();

        let file_path = cache_dir.join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        write!(file, "test data").unwrap();

        let cache = CacheInfo {
            name: "TestCache".to_string(),
            path: cache_dir.clone(),
            size: 9,
            exists: true,
            description: "Test".to_string(),
        };

        let result = purge_cache(&cache);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 9);

        // Directory should be recreated but empty
        assert!(cache_dir.exists());
        assert!(fs::read_dir(&cache_dir).unwrap().next().is_none());
    }

    #[test]
    fn test_purge_cache_trash_behavior() {
        let dir = tempdir().unwrap();
        let trash_dir = dir.path().join(".Trash");
        fs::create_dir(&trash_dir).unwrap();

        // Add files to "trash"
        let file1 = trash_dir.join("file1.txt");
        let subdir = trash_dir.join("subdir");
        fs::create_dir(&subdir).unwrap();
        let file2 = subdir.join("file2.txt");

        File::create(&file1).unwrap();
        File::create(&file2).unwrap();

        let cache = CacheInfo {
            name: "Trash".to_string(),
            path: trash_dir.clone(),
            size: 100,
            exists: true,
            description: "Test Trash".to_string(),
        };

        let result = purge_cache(&cache);
        assert!(result.is_ok());

        // Trash directory should still exist but be empty
        assert!(trash_dir.exists());
        assert!(fs::read_dir(&trash_dir).unwrap().next().is_none());
    }

    #[test]
    fn test_discover_caches_returns_sorted() {
        let caches = discover_caches();

        // Verify sorted by size descending
        for window in caches.windows(2) {
            assert!(window[0].size >= window[1].size);
        }

        // All returned caches should exist and have size > 0
        for cache in &caches {
            assert!(cache.exists);
            assert!(cache.size > 0);
        }
    }
}
