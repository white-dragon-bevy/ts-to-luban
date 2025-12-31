use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

pub fn scan_directory(dir: &Path) -> Result<Vec<PathBuf>> {
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

        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        // Include .ts and .tsx files
        if !file_name.ends_with(".ts") && !file_name.ends_with(".tsx") {
            continue;
        }

        // Exclude declaration files and test files
        if file_name.ends_with(".d.ts")
            || file_name.ends_with(".spec.ts")
            || file_name.ends_with(".test.ts")
            || file_name.ends_with(".spec.tsx")
            || file_name.ends_with(".test.tsx")
        {
            continue;
        }

        // Exclude node_modules
        if path.components().any(|c| c.as_os_str() == "node_modules") {
            continue;
        }

        files.push(path.to_path_buf());
    }

    Ok(files)
}

pub fn scan_directories(dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut all_files = Vec::new();

    for dir in dirs {
        let files = scan_directory(dir)?;
        all_files.extend(files);
    }

    Ok(all_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

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
}
