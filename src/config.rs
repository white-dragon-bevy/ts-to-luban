use serde::Deserialize;
use std::path::PathBuf;

/// Table configuration - supports both simple string format and full object format
/// Simple: "module.ClassName" = "../datas/path"
/// Full: "module.ClassName" = { input = "../datas/path", mode = "one", index = "id", name = "TbCustom" }
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum TableConfig {
    /// Simple format: just the input path
    Simple(String),
    /// Full format: all options
    Full {
        input: String,
        #[serde(default, alias = "table_name")]
        name: Option<String>,
        #[serde(default)]
        mode: Option<String>,
        #[serde(default)]
        index: Option<String>,
    },
}

impl TableConfig {
    /// Get the input path
    pub fn input(&self) -> &str {
        match self {
            TableConfig::Simple(s) => s,
            TableConfig::Full { input, .. } => input,
        }
    }

    /// Get the table name (or None for default)
    pub fn name(&self) -> Option<&str> {
        match self {
            TableConfig::Simple(_) => None,
            TableConfig::Full { name, .. } => name.as_deref(),
        }
    }

    /// Get the mode (default: "map")
    pub fn mode(&self) -> &str {
        match self {
            TableConfig::Simple(_) => "map",
            TableConfig::Full { mode, .. } => mode.as_deref().unwrap_or("map"),
        }
    }

    /// Get the index field (default: "id")
    pub fn index(&self) -> &str {
        match self {
            TableConfig::Simple(_) => "id",
            TableConfig::Full { index, .. } => index.as_deref().unwrap_or("id"),
        }
    }
}

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
    pub ref_configs: Vec<RefConfig>,
    #[serde(default)]
    pub table_mappings: Vec<TableMapping>,
    /// New [tables] configuration - maps "module.ClassName" to table config
    #[serde(default)]
    pub tables: std::collections::HashMap<String, TableConfig>,
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
    /// Path to output TypeScript table code
    #[serde(default)]
    pub table_output_path: Option<PathBuf>,
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
    Registration {
        path: PathBuf,
    },
}

#[derive(Debug, Deserialize, Default)]
pub struct DefaultsConfig {}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ScanOptions {
    #[serde(default)]
    pub include_dts: bool,
    #[serde(default)]
    pub include_node_modules: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RefConfig {
    pub path: PathBuf,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TableMapping {
    pub pattern: String,
    pub input: String,
    pub output: Option<String>,
    #[serde(default)]
    pub table_name: Option<String>,
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Load config and merge referenced configs
    pub fn load_with_refs(path: &std::path::Path) -> anyhow::Result<Self> {
        let mut config = Self::load(path)?;
        let config_dir = path.parent().unwrap_or(std::path::Path::new("."));

        // Collect sources from referenced configs
        for ref_config in &config.ref_configs {
            let ref_path = config_dir.join(&ref_config.path);
            let ref_path = ref_path.canonicalize().unwrap_or(ref_path.clone());

            // Recursively load referenced config
            let referenced = Self::load_with_refs(&ref_path).map_err(|e| {
                anyhow::anyhow!("Failed to load ref_config {:?}: {}", ref_config.path, e)
            })?;
            let ref_dir = ref_path.parent().unwrap_or(std::path::Path::new("."));

            // Merge sources with resolved paths
            for source in referenced.sources {
                let resolved_source = Self::resolve_source_path(source, ref_dir);
                config.sources.push(resolved_source);
            }
        }

        Ok(config)
    }

    /// Resolve source path relative to the config directory
    /// Note: output_path is NOT resolved - it uses the runtime root directory
    fn resolve_source_path(source: SourceConfig, base_dir: &std::path::Path) -> SourceConfig {
        match source {
            SourceConfig::Directory {
                path,
                scan_options,
                output_path,
                module_name,
            } => {
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
            SourceConfig::File {
                path,
                output_path,
                module_name,
            } => {
                let resolved = if path.is_absolute() {
                    path
                } else {
                    base_dir.join(&path)
                };
                SourceConfig::File {
                    path: resolved,
                    output_path,
                    module_name,
                }
            }
            SourceConfig::Files {
                paths,
                output_path,
                module_name,
            } => {
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
            SourceConfig::Glob {
                pattern,
                output_path,
                module_name,
            } => {
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
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.project.tsconfig, PathBuf::from("tsconfig.json"));
        assert_eq!(config.output.path, PathBuf::from("output.xml"));
        assert_eq!(config.sources.len(), 1);
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
        assert_eq!(
            config.type_mappings.get("Vector3"),
            Some(&"Vector3".to_string())
        );
        assert_eq!(
            config.type_mappings.get("Entity"),
            Some(&"long".to_string())
        );
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
        assert_eq!(
            config.ref_configs[0].path,
            PathBuf::from("../other-pkg/ts-luban.config.toml")
        );
        assert_eq!(
            config.ref_configs[1].path,
            PathBuf::from("../another-pkg/ts-luban.config.toml")
        );
    }

    #[test]
    fn test_load_with_refs_merges_sources() {
        let config_path =
            PathBuf::from("tests/fixtures/ref_config_test/pkg-a/ts-luban.config.toml");
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
        assert!(
            has_pkg_b_source,
            "Should have pkg-b source with resolved path"
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

        if let SourceConfig::Files {
            paths, output_path, ..
        } = &config.sources[0]
        {
            assert_eq!(paths.len(), 3);
            assert_eq!(paths[0], PathBuf::from("src/types/a.ts"));
            assert_eq!(paths[1], PathBuf::from("src/types/b.ts"));
            assert_eq!(paths[2], PathBuf::from("src/events/c.ts"));
            assert_eq!(
                output_path.as_ref().unwrap(),
                &PathBuf::from("output/types.xml")
            );
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
            assert_eq!(
                output_path.as_ref().unwrap(),
                &PathBuf::from("output/triggers.xml")
            );
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

        if let SourceConfig::Glob {
            pattern,
            output_path,
            module_name,
        } = &config.sources[0]
        {
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

        if let SourceConfig::Glob {
            pattern,
            output_path,
            module_name,
        } = &config.sources[0]
        {
            assert_eq!(pattern, "src/**/*.ts");
            assert_eq!(
                output_path.as_ref().unwrap(),
                &PathBuf::from("output/matched.xml")
            );
            assert_eq!(module_name.as_deref(), Some("matched"));
        } else {
            panic!("Expected Glob source");
        }
    }

    #[test]
    fn test_parse_table_mappings() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[table_mappings]]
pattern = "Tb.*"
input = "configs/{name}.xlsx"
output = "{name}"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.table_mappings.len(), 1);
        assert_eq!(config.table_mappings[0].pattern, "Tb.*");
    }

    #[test]
    fn test_parse_table_output_path() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"
table_output_path = "out/tables"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(
            config.output.table_output_path,
            Some(PathBuf::from("out/tables"))
        );
    }

    #[test]
    fn test_parse_tables_simple_format() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[tables]
"role.RoleConfig" = "../datas/role"
"weapon.WeaponConfig" = "../datas/weapon"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tables.len(), 2);

        let role_config = config.tables.get("role.RoleConfig").unwrap();
        assert_eq!(role_config.input(), "../datas/role");
        assert_eq!(role_config.mode(), "map");
        assert_eq!(role_config.index(), "id");
        assert!(role_config.name().is_none());

        let weapon_config = config.tables.get("weapon.WeaponConfig").unwrap();
        assert_eq!(weapon_config.input(), "../datas/weapon");
    }

    #[test]
    fn test_parse_tables_full_format() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[tables]
"rollSkill.RollSkillConfig" = { input = "../datas/roll-skill", mode = "one" }
"battle.BattleData" = { input = "../datas/battle", name = "TbBattle", index = "battleId" }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tables.len(), 2);

        let roll_skill = config.tables.get("rollSkill.RollSkillConfig").unwrap();
        assert_eq!(roll_skill.input(), "../datas/roll-skill");
        assert_eq!(roll_skill.mode(), "one");
        assert_eq!(roll_skill.index(), "id"); // default
        assert!(roll_skill.name().is_none());

        let battle = config.tables.get("battle.BattleData").unwrap();
        assert_eq!(battle.input(), "../datas/battle");
        assert_eq!(battle.mode(), "map"); // default
        assert_eq!(battle.index(), "battleId");
        assert_eq!(battle.name(), Some("TbBattle"));
    }

    #[test]
    fn test_parse_tables_mixed_format() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[tables]
"role.RoleConfig" = "../datas/role"
"skill.SkillConfig" = { input = "../datas/skill", mode = "list" }
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tables.len(), 2);

        let role = config.tables.get("role.RoleConfig").unwrap();
        assert_eq!(role.input(), "../datas/role");
        assert_eq!(role.mode(), "map");

        let skill = config.tables.get("skill.SkillConfig").unwrap();
        assert_eq!(skill.input(), "../datas/skill");
        assert_eq!(skill.mode(), "list");
    }

    #[test]
    fn test_parse_tables_empty() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.tables.len(), 0);
    }
}
