use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub project: ProjectConfig,
    pub output: OutputConfig,
    #[serde(default)]
    pub sources: Vec<SourceConfig>,
    #[serde(default)]
    pub base_class_mappings: Vec<BaseClassMapping>,
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub type_mappings: std::collections::HashMap<String, String>,
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
}

fn default_cache_file() -> PathBuf {
    PathBuf::from(".luban-cache.json")
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum SourceConfig {
    Directory { path: PathBuf },
    Registration { path: PathBuf },
}

#[derive(Debug, Deserialize)]
pub struct BaseClassMapping {
    pub interface: String,
    pub maps_to: String,
}

#[derive(Debug, Deserialize, Default)]
pub struct DefaultsConfig {
    #[serde(default = "default_base_class")]
    pub base_class: String,
}

fn default_base_class() -> String {
    "TsClass".to_string()
}

impl Config {
    pub fn load(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
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
    fn test_parse_base_class_mappings() {
        let toml_str = r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"

[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

[[base_class_mappings]]
interface = "Component"
maps_to = "TsComponentClass"
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.base_class_mappings.len(), 2);
        assert_eq!(config.base_class_mappings[0].interface, "EntityTrigger");
        assert_eq!(config.base_class_mappings[0].maps_to, "TsTriggerClass");
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
}
