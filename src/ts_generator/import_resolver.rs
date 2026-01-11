use crate::tsconfig::TsConfig;
use std::collections::HashMap;
use std::path::Path;

pub struct ImportResolver {
    /// tsconfig paths mapping
    #[allow(dead_code)]
    paths: HashMap<String, Vec<String>>,
    /// Base URL from tsconfig
    #[allow(dead_code)]
    base_url: Option<String>,
}

impl ImportResolver {
    pub fn new(tsconfig: &TsConfig) -> Self {
        Self {
            paths: tsconfig.compiler_options.paths.clone(),
            base_url: tsconfig.compiler_options.base_url.clone(),
        }
    }

    /// Resolve import path from generated file to source file
    /// Returns package name if in node_modules, else relative path
    pub fn resolve(&self, from: &Path, to: &Path) -> String {
        // Check if 'to' is in node_modules
        let to_str = to.to_string_lossy();
        if to_str.contains("node_modules") {
            // Extract package name
            return self.extract_package_name(to);
        }

        // Calculate relative path
        self.calculate_relative_path(from, to)
    }

    fn extract_package_name(&self, path: &Path) -> String {
        // Find node_modules in path and extract @scope/package or package
        let path_str = path.to_string_lossy();
        // Handle both forward and backslashes
        let normalized = path_str.replace('\\', "/");

        if let Some(idx) = normalized.find("node_modules/") {
            let after = &normalized[idx + "node_modules/".len()..];
            // Handle scoped packages (@scope/pkg)
            if after.starts_with('@') {
                let parts: Vec<&str> = after.splitn(3, '/').collect();
                if parts.len() >= 2 {
                    return format!("{}/{}", parts[0], parts[1]);
                }
            }
            // Regular package
            if let Some(pkg) = after.split('/').next() {
                return pkg.to_string();
            }
        }
        path.to_string_lossy().to_string()
    }

    fn calculate_relative_path(&self, from: &Path, to: &Path) -> String {
        // Get the directory containing the 'from' file
        let from_dir = from.parent().unwrap_or(Path::new("."));

        pathdiff::diff_paths(to, from_dir)
            .map(|p| {
                // Normalize to forward slashes for TypeScript imports
                let s = p.to_string_lossy().replace('\\', "/");
                // Remove .ts extension
                let s = s.trim_end_matches(".ts");
                // Ensure it starts with ./ or ../
                if s.starts_with('.') {
                    s.to_string()
                } else {
                    format!("./{}", s)
                }
            })
            .unwrap_or_else(|| to.to_string_lossy().to_string())
    }
}

impl Default for ImportResolver {
    fn default() -> Self {
        Self {
            paths: HashMap::new(),
            base_url: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_path() {
        let resolver = ImportResolver::default();
        let from = Path::new("out/tables/item-table.ts");
        let to = Path::new("src/types/item.ts");
        let result = resolver.calculate_relative_path(from, to);
        assert!(result.contains(".."));
    }

    #[test]
    fn test_node_modules_package() {
        let resolver = ImportResolver::default();
        let path = Path::new("node_modules/@white-dragon-bevy/ts-to-luban/src/index.ts");
        let result = resolver.extract_package_name(path);
        assert_eq!(result, "@white-dragon-bevy/ts-to-luban");
    }

    #[test]
    fn test_node_modules_regular_package() {
        let resolver = ImportResolver::default();
        let path = Path::new("node_modules/lodash/index.ts");
        let result = resolver.extract_package_name(path);
        assert_eq!(result, "lodash");
    }
}
