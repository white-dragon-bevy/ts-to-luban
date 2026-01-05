use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub sources: Vec<SourceConfig>,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub type_mappings: std::collections::HashMap<String, String>,
    #[serde(default)]
    pub parent_mappings: Vec<ParentMapping>,
    #[serde(default)]
    pub ref_configs: Vec<RefConfig>,
}

#[derive(Debug, Deserialize)]
pub struct ProjectConfig {
    pub tsconfig: PathBuf,
}

#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
    #[serde(default = "default_cache_file")]
    pub cache_file: PathBuf,
    #[serde(default)]
    pub module_name: String,
    /// Path to output enum XML file (optional, defaults to {path}_enums.xml)
    #[serde(default)]
    pub enum_path: Option<PathBuf>,
    /// Path to output bean type enums XML file (grouped by parent)
    #[serde(default)]
    pub bean_types_path: Option<PathBuf>,
}

fn default_cache_file() -> PathBuf {
    PathBuf::from(".luban-cache.json")
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceConfig {
    Directory {
        path: PathBuf,
        #[serde(default)]
        scan_options: ScanOptions,
        #[serde(default)]
        output_path: Option<PathBuf>,
        #[serde(default)]
        module_name: Option<String>,
    },
    File {
        path: PathBuf,
        #[serde(default)]
        output_path: Option<PathBuf>,
        #[serde(default)]
        module_name: Option<String>,
    },
    Files {
        paths: Vec<PathBuf>,
        #[serde(default)]
        output_path: Option<PathBuf>,
        #[serde(default)]
        module_name: Option<String>,
    },
    Glob {
        pattern: String,
        #[serde(default)]
        output_path: Option<PathBuf>,
        #[serde(default)]
        module_name: Option<String>,
    },
    Registration { path: PathBuf },
}

#[derive(Debug, Deserialize, Default)]
pub struct DefaultsConfig {
    #[serde(default = "default_base_class")]
    pub base_class: String,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ScanOptions {
    #[serde(default)]
    pub include_dts: bool,
    #[serde(default)]
    pub include_node_modules: bool,
}

fn default_base_class() -> String {
    "TsClass".to_string()
}

#[derive(Debug, Deserialize, Clone)]
pub struct ParentMapping {
    pub pattern: String,
    pub parent: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RefConfig {
    pub path: PathBuf,
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load config and merge referenced configs
    /// Root config takes priority: if a pattern exists in root, ref's pattern is ignored
    pub fn load_with_refs(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut config = Self::load(path)?;
        let config_dir = path.parent().unwrap_or(std::path::Path::new("."));

        // Track existing patterns from root config for priority
        let existing_patterns: std::collections::HashSet<_> = config
            .parent_mappings
            .iter()
            .map(|m| m.pattern.clone())
            .collect();

        // Collect sources and parent_mappings from referenced configs
        for ref_config in &config.ref_configs {
            let ref_path = config_dir.join(&ref_config.path);
            let ref_path = ref_path.canonicalize().unwrap_or(ref_path.clone());

            // Recursively load referenced config
            let referenced = Self::load_with_refs(&ref_path)
                .map_err(|e| anyhow::anyhow!("Failed to load ref_config {:?}: {}", ref_config.path, e))?;
            let ref_dir = ref_path.parent().unwrap_or(std::path::Path::new("."));

            // Merge sources with resolved paths
            for source in referenced.sources {
                let resolved_source = Self::resolve_source_path(source, ref_dir);
                config.sources.push(resolved_source);
            }

            // Merge parent_mappings (only add if pattern doesn't exist in root)
            for mapping in referenced.parent_mappings {
                if !existing_patterns.contains(&mapping.pattern) {
                    config.parent_mappings.push(mapping);
                }
            }
        }

        Ok(config)
    }

    /// Resolve source path relative to the config directory
    /// Note: output_path is NOT resolved - it uses the runtime root directory
    fn resolve_source_path(source: SourceConfig, base_dir: &std::path::Path) -> SourceConfig {
        match source {
            SourceConfig::Directory { path, scan_options, output_path, module_name } => {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    base_dir.join(&path)
                };
                SourceConfig::Directory {
                    path: resolved,
                    scan_options,
                    output_path,
                    module_name,
                }
            }
            SourceConfig::File { path, output_path, module_name } => {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    base_dir.join(&path)
                };
                SourceConfig::File { path: resolved, output_path, module_name }
            }
            SourceConfig::Files { paths, output_path, module_name } => {
                let resolved_paths = paths
                    .into_iter()
                    .map(|p| {
                        if p.is_absolute() {
                            p
                        } else {
                            base_dir.join(&p)
                        }
                    })
                    .collect();
                SourceConfig::Files {
                    paths: resolved_paths,
                    output_path,
                    module_name,
                }
            }
            SourceConfig::Registration { path } => {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    base_dir.join(&path)
                };
                SourceConfig::Registration { path: resolved }
            }
            SourceConfig::Glob { pattern, output_path, module_name } => {
                // Prepend base_dir to the pattern for relative patterns
                let resolved_pattern = if std::path::Path::new(&pattern).is_absolute() {
                    pattern
                } else {
                    base_dir.join(&pattern).to_string_lossy().to_string()
                };
                SourceConfig::Glob {
                    pattern: resolved_pattern,
                    output_path,
                    module_name,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_config_basic() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"
cache_file = ".luban-cache.json"

[[sources]]
type = "directory"
path = "src/triggers"

[defaults]
base_class = "TsClass"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.tsconfig, PathBuf::from("tsconfig.json"));
        assert_eq!(config.output.path, PathBuf::from("output.xml"));
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.defaults.base_class, "TsClass");
    }

    #[test]
    fn test_parse_type_mappings() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[type_mappings]
Vector3 = "Vector3"
Entity = "long"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.type_mappings.get("Vector3"), Some(&"Vector3".to_string()));
        assert_eq!(config.type_mappings.get("Entity"), Some(&"long".to_string()));
    }

    #[test]
    fn test_parse_ref_configs() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[ref_configs]]
path = "../other-pkg/ts-luban.config.toml"

[[ref_configs]]
path = "../another-pkg/ts-luban.config.toml"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.ref_configs.len(), 2);
        assert_eq!(config.ref_configs[0].path, PathBuf::from("../other-pkg/ts-luban.config.toml"));
        assert_eq!(config.ref_configs[1].path, PathBuf::from("../another-pkg/ts-luban.config.toml"));
    }

    #[test]
    fn test_load_with_refs_merges_sources() {
        let config_path = PathBuf::from("tests/fixtures/ref_config_test/pkg-a/ts-luban.config.toml");
        let config = Config::load_with_refs(&config_path).unwrap();

        // Should have 2 sources: one from pkg-a (local-src) and one from pkg-b (src)
        assert_eq!(config.sources.len(), 2);

        // Check that pkg-b's source path is resolved relative to pkg-b
        let has_pkg_b_source = config.sources.iter().any(|s| {
            if let SourceConfig::Directory { path, .. } = s {
                path.ends_with("pkg-b/src") || path.to_string_lossy().contains("pkg-b")
            } else {
                false
            }
        });
        assert!(has_pkg_b_source, "Should have pkg-b source with resolved path");
    }

    #[test]
    fn test_load_with_refs_merges_parent_mappings() {
        let config_path = PathBuf::from("tests/fixtures/ref_config_test/pkg-a/ts-luban.config.toml");
        let config = Config::load_with_refs(&config_path).unwrap();

        // Should have 2 parent mappings: Trigger from pkg-a and Handler from pkg-b
        assert_eq!(config.parent_mappings.len(), 2);

        let has_trigger = config.parent_mappings.iter().any(|m| m.pattern.contains("Trigger"));
        let has_handler = config.parent_mappings.iter().any(|m| m.pattern.contains("Handler"));

        assert!(has_trigger, "Should have Trigger mapping from pkg-a");
        assert!(has_handler, "Should have Handler mapping from pkg-b");
    }

    #[test]
    fn test_load_with_refs_root_config_priority() {
        let config_path = PathBuf::from("tests/fixtures/priority_test/root/ts-luban.config.toml");
        let config = Config::load_with_refs(&config_path).unwrap();

        // Both configs have ".*Trigger$" pattern, but only root's should be kept
        let trigger_mappings: Vec<_> = config
            .parent_mappings
            .iter()
            .filter(|m| m.pattern == ".*Trigger$")
            .collect();

        assert_eq!(trigger_mappings.len(), 1, "Should have only one Trigger mapping");
        assert_eq!(
            trigger_mappings[0].parent, "RootTriggerBase",
            "Root config should take priority over ref config"
        );
    }

    #[test]
    fn test_parse_files_source() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[sources]]
type = "files"
paths = ["src/types/a.ts", "src/types/b.ts", "src/events/c.ts"]
output_path = "output/types.xml"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sources.len(), 1);

        if let SourceConfig::Files { paths, output_path, .. } = &config.sources[0] {
            assert_eq!(paths.len(), 3);
            assert_eq!(paths[0], PathBuf::from("src/types/a.ts"));
            assert_eq!(paths[1], PathBuf::from("src/types/b.ts"));
            assert_eq!(paths[2], PathBuf::from("src/events/c.ts"));
            assert_eq!(output_path.as_ref().unwrap(), &PathBuf::from("output/types.xml"));
        } else {
            panic!("Expected Files source");
        }
    }

    #[test]
    fn test_parse_source_with_module_name() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[sources]]
type = "directory"
path = "src/triggers"
module_name = "triggers"

[[sources]]
type = "file"
path = "src/types.ts"
module_name = "types"

[[sources]]
type = "files"
paths = ["src/a.ts", "src/b.ts"]
module_name = ""
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sources.len(), 3);

        // Directory with module_name
        if let SourceConfig::Directory { module_name, .. } = &config.sources[0] {
            assert_eq!(module_name.as_deref(), Some("triggers"));
        } else {
            panic!("Expected Directory source");
        }

        // File with module_name
        if let SourceConfig::File { module_name, .. } = &config.sources[1] {
            assert_eq!(module_name.as_deref(), Some("types"));
        } else {
            panic!("Expected File source");
        }

        // Files with empty module_name
        if let SourceConfig::Files { module_name, .. } = &config.sources[2] {
            assert_eq!(module_name.as_deref(), Some(""));
        } else {
            panic!("Expected Files source");
        }
    }

    #[test]
    fn test_default_module_name_is_empty() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        // Default module_name should be empty string
        assert_eq!(config.output.module_name, "");
    }

    #[test]
    fn test_parse_source_with_output_path() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output/default.xml"

[[sources]]
type = "directory"
path = "src/triggers"
output_path = "output/triggers.xml"

[[sources]]
type = "directory"
path = "src/events"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sources.len(), 2);

        // First source has custom output_path
        if let SourceConfig::Directory { output_path, .. } = &config.sources[0] {
            assert_eq!(output_path.as_ref().unwrap(), &PathBuf::from("output/triggers.xml"));
        } else {
            panic!("Expected Directory source");
        }

        // Second source has no output_path (None)
        if let SourceConfig::Directory { output_path, .. } = &config.sources[1] {
            assert!(output_path.is_none());
        } else {
            panic!("Expected Directory source");
        }
    }

    #[test]
    fn test_parse_glob_source() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sources.len(), 1);

        if let SourceConfig::Glob { pattern, output_path, module_name } = &config.sources[0] {
            assert_eq!(pattern, "src/**/*Trigger.ts");
            assert!(output_path.is_none());
            assert!(module_name.is_none());
        } else {
            panic!("Expected Glob source");
        }
    }

    #[test]
    fn test_parse_glob_with_options() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[sources]]
type = "glob"
pattern = "src/**/*.ts"
output_path = "output/matched.xml"
module_name = "matched"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.sources.len(), 1);

        if let SourceConfig::Glob { pattern, output_path, module_name } = &config.sources[0] {
            assert_eq!(pattern, "src/**/*.ts");
            assert_eq!(output_path.as_ref().unwrap(), &PathBuf::from("output/matched.xml"));
            assert_eq!(module_name.as_deref(), Some("matched"));
        } else {
            panic!("Expected Glob source");
        }
    }

}
