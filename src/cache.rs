use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub entries: HashMap<String, CacheEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub source: String,
    pub hash: String,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: Utc::now(),
            entries: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let cache = Self::from_json(&content)?;

        // Invalidate cache if version mismatch
        if cache.version != env!("CARGO_PKG_VERSION") {
            return Ok(Self::new());
        }

        Ok(cache)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        let json = self.to_json()?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let cache: Cache = serde_json::from_str(json)?;
        Ok(cache)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    #[allow(dead_code)]
    pub fn get_entry(&self, class_name: &str) -> Option<&CacheEntry> {
        self.entries.get(class_name)
    }

    pub fn set_entry(&mut self, class_name: &str, source: &str, hash: &str) {
        self.entries.insert(
            class_name.to_string(),
            CacheEntry {
                source: source.to_string(),
                hash: hash.to_string(),
            },
        );
    }

    pub fn is_valid(&self, class_name: &str, current_hash: &str) -> bool {
        self.entries
            .get(class_name)
            .map(|e| e.hash == current_hash)
            .unwrap_or(false)
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.entries.clear();
        self.generated_at = Utc::now();
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_roundtrip() {
        let mut cache = Cache::new();
        cache.set_entry("MyClass", "test.ts", "abc123");

        let json = cache.to_json().unwrap();
        let loaded = Cache::from_json(&json).unwrap();

        let entry = loaded.get_entry("MyClass").unwrap();
        assert_eq!(entry.source, "test.ts");
        assert_eq!(entry.hash, "abc123");
    }

    #[test]
    fn test_is_valid() {
        let mut cache = Cache::new();
        cache.set_entry("MyClass", "test.ts", "abc123");

        assert!(cache.is_valid("MyClass", "abc123"));
        assert!(!cache.is_valid("MyClass", "different"));
        assert!(!cache.is_valid("OtherClass", "abc123"));
    }

    #[test]
    fn test_load_missing_file() {
        let cache = Cache::load(Path::new("/nonexistent/path.json")).unwrap();
        assert!(cache.entries.is_empty());
    }
}
