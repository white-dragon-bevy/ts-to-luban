use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    #[serde(default)]
    #[allow(dead_code)]
    pub compiler_options: CompilerOptions,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    #[serde(default)]
    #[allow(dead_code)]
    pub base_url: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    pub paths: HashMap<String, Vec<String>>,
}

impl TsConfig {
    pub fn load(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        // Remove comments (tsconfig allows them)
        let cleaned = remove_json_comments(&content);
        let config: TsConfig = serde_json::from_str(&cleaned)?;
        Ok(config)
    }
}

fn remove_json_comments(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    let mut in_string = false;

    while let Some(c) = chars.next() {
        if in_string {
            result.push(c);
            if c == '"' {
                in_string = false;
            } else if c == '\\' {
                if let Some(next) = chars.next() {
                    result.push(next);
                }
            }
        } else if c == '"' {
            in_string = true;
            result.push(c);
        } else if c == '/' {
            match chars.peek() {
                Some('/') => {
                    // Line comment - skip until newline
                    while let Some(nc) = chars.next() {
                        if nc == '\n' {
                            result.push('\n');
                            break;
                        }
                    }
                }
                Some('*') => {
                    // Block comment - skip until */
                    chars.next(); // consume *
                    while let Some(nc) = chars.next() {
                        if nc == '*' && chars.peek() == Some(&'/') {
                            chars.next();
                            break;
                        }
                    }
                }
                _ => result.push(c),
            }
        } else {
            result.push(c);
        }
    }
    result
}

#[allow(dead_code)]
pub struct PathResolver {
    base_url: PathBuf,
    paths: Vec<(String, String)>,
}

#[allow(dead_code)]
impl PathResolver {
    pub fn new(tsconfig: &TsConfig, project_root: &Path) -> Self {
        let base_url = tsconfig
            .compiler_options
            .base_url
            .as_ref()
            .map(|b| project_root.join(b))
            .unwrap_or_else(|| project_root.to_path_buf());

        let mut paths = Vec::new();
        for (pattern, replacements) in &tsconfig.compiler_options.paths {
            if let Some(replacement) = replacements.first() {
                // Skip pure wildcard "*" pattern
                if pattern != "*" {
                    paths.push((pattern.clone(), replacement.clone()));
                }
            }
        }

        Self { base_url, paths }
    }

    pub fn resolve(&self, import_path: &str) -> PathBuf {
        // Check path aliases
        for (pattern, replacement) in &self.paths {
            if let Some(matched) = self.match_pattern(pattern, import_path) {
                let resolved = replacement.replace("*", &matched);
                return self.base_url.join(resolved);
            }
        }

        // Default: treat as relative to base_url
        self.base_url.join(import_path)
    }

    fn match_pattern(&self, pattern: &str, input: &str) -> Option<String> {
        if let Some(prefix) = pattern.strip_suffix("/*") {
            if let Some(suffix) = input.strip_prefix(&format!("{}/", prefix)) {
                return Some(suffix.to_string());
            }
        } else if pattern == input {
            return Some(String::new());
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_paths() {
        let json = r#"{
            "compilerOptions": {
                "baseUrl": "./src",
                "paths": {
                    "shared/*": ["shared/*"],
                    "@types/*": ["types/*"]
                }
            }
        }"#;
        let tsconfig: TsConfig = serde_json::from_str(json).unwrap();
        let resolver = PathResolver::new(&tsconfig, Path::new("/project"));

        let resolved = resolver.resolve("shared/utils");
        assert_eq!(resolved, PathBuf::from("/project/src/shared/utils"));
    }

    #[test]
    fn test_resolve_at_alias() {
        let json = r#"{
            "compilerOptions": {
                "baseUrl": "./src",
                "paths": {
                    "@types/*": ["types/*"]
                }
            }
        }"#;
        let tsconfig: TsConfig = serde_json::from_str(json).unwrap();
        let resolver = PathResolver::new(&tsconfig, Path::new("/project"));

        let resolved = resolver.resolve("@types/models");
        assert_eq!(resolved, PathBuf::from("/project/src/types/models"));
    }

    #[test]
    fn test_remove_json_comments() {
        let input = r#"{
            // This is a comment
            "key": "value", /* block comment */
            "key2": "value2"
        }"#;
        let cleaned = remove_json_comments(input);
        assert!(!cleaned.contains("//"));
        assert!(!cleaned.contains("/*"));
        assert!(cleaned.contains("\"key\""));
    }
}
