use crate::config::ScanOptions;
use anyhow::Result;
use glob::glob;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Default)]
pub struct ScanConfig {
    pub include_dts: bool,
    pub include_node_modules: bool,
}

impl From<&ScanOptions> for ScanConfig {
    fn from(opts: &ScanOptions) -> Self {
        Self {
            include_dts: opts.include_dts,
            include_node_modules: opts.include_node_modules,
        }
    }
}

#[allow(dead_code)]
pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    scan_directory_with_options(dir, &ScanConfig::default())
}

pub fn scan_directory_with_options(dir: &Path, config: &ScanConfig) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Include .ts and .tsx files
        if !file_name.ends_with(".ts") && !file_name.ends_with(".tsx") {
            continue;
        }

        // Exclude test files
        if file_name.ends_with(".spec.ts")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".spec.tsx")
            || file_name.ends_with(".test.tsx")
        {
            continue;
        }

        // Exclude declaration files unless configured
        if !config.include_dts && file_name.ends_with(".d.ts") {
            continue;
        }

        // Exclude node_modules unless configured
        if !config.include_node_modules
            && path.components().any(|c| c.as_os_str() == "node_modules")
        {
            continue;
        }

        files.push(path.to_path_buf());
    }

    Ok(files)
}

#[allow(dead_code)]
pub fn scan_directories(dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut all_files = Vec::new();

    for dir in dirs {
        let files = scan_directory(dir)?;
        all_files.extend(files);
    }

    Ok(all_files)
}

/// Expand a glob pattern and return matching files
/// Only returns files (not directories) that match the pattern
pub fn expand_glob(pattern: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in glob(pattern).map_err(|e| anyhow::anyhow!("Invalid glob pattern: {}", e))? {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    files.push(path);
                }
            }
            Err(e) => {
                eprintln!("  Warning: Glob error for {:?}: {}", pattern, e);
            }
        }
    }

    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_scan_directory() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("a.ts"), "export class A {}").unwrap();
        fs::write(dir.path().join("b.tsx"), "export class B {}").unwrap();
        fs::write(dir.path().join("c.d.ts"), "declare class C {}").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_exclude_test_files() {
        let dir = TempDir::new().unwrap();
        fs::write(dir.path().join("main.ts"), "export class A {}").unwrap();
        fs::write(dir.path().join("main.spec.ts"), "test").unwrap();
        fs::write(dir.path().join("main.test.ts"), "test").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_recursive_scan() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("sub");
        fs::create_dir(&sub).unwrap();

        fs::write(dir.path().join("a.ts"), "export class A {}").unwrap();
        fs::write(sub.join("b.ts"), "export class B {}").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_exclude_node_modules() {
        let dir = TempDir::new().unwrap();
        let nm = dir.path().join("node_modules");
        fs::create_dir(&nm).unwrap();

        fs::write(dir.path().join("a.ts"), "export class A {}").unwrap();
        fs::write(nm.join("b.ts"), "export class B {}").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 1);
    }

    #[test]
    fn test_expand_glob_pattern() {
        let dir = TempDir::new().unwrap();
        let sub = dir.path().join("src");
        fs::create_dir(&sub).unwrap();

        fs::write(
            sub.join("DamageTrigger.ts"),
            "export class DamageTrigger {}",
        )
        .unwrap();
        fs::write(sub.join("HealTrigger.ts"), "export class HealTrigger {}").unwrap();
        fs::write(sub.join("Component.ts"), "export class Component {}").unwrap();

        // Pattern matching *Trigger.ts
        let pattern = format!("{}/*Trigger.ts", sub.display());
        let files = expand_glob(&pattern).unwrap();
        assert_eq!(files.len(), 2);

        // All files should end with Trigger.ts
        for file in &files {
            assert!(file
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .ends_with("Trigger.ts"));
        }
    }

    #[test]
    fn test_expand_glob_recursive() {
        let dir = TempDir::new().unwrap();
        let src = dir.path().join("src");
        let nested = src.join("nested");
        fs::create_dir_all(&nested).unwrap();

        fs::write(src.join("a.ts"), "export class A {}").unwrap();
        fs::write(nested.join("b.ts"), "export class B {}").unwrap();

        // Recursive pattern **/*.ts
        let pattern = format!("{}/**/*.ts", src.display());
        let files = expand_glob(&pattern).unwrap();
        assert_eq!(files.len(), 2);
    }
}
