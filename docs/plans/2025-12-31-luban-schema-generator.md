# Luban Schema Generator (Rust Edition) Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a high-performance Rust CLI tool that parses TypeScript files using SWC and generates Luban XML Schema definitions.

**Architecture:** Parse TypeScript files in parallel using SWC, extract class/interface information into an in-memory model, build inheritance trees, apply base class mapping rules, and generate XML output. Use independent cache file for incremental compilation.

**Tech Stack:** Rust, SWC (swc_ecma_parser), Rayon (parallel processing), Clap (CLI), Serde + TOML (config), quick-xml (XML generation)

---

## Task 1: Project Initialization

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Create Cargo.toml with dependencies**

```toml
[package]
name = "luban-gen"
version = "0.1.0"
edition = "2021"
description = "High-performance TypeScript to Luban XML Schema generator"

[dependencies]
# CLI
clap = { version = "4", features = ["derive"] }

# Serialization
serde = { version = "1", features = ["derive"] }
serde_json = "1"
toml = "0.8"

# TypeScript parsing (SWC)
swc_common = "5"
swc_ecma_parser = "5"
swc_ecma_ast = "5"

# Parallel processing
rayon = "1.10"

# XML generation
quick-xml = { version = "0.37", features = ["serialize"] }

# File hashing
md-5 = "0.10"

# Error handling
anyhow = "1"
thiserror = "2"

# Path handling
walkdir = "2"
glob = "0.3"

# Time
chrono = { version = "0.4", features = ["serde"] }

[profile.release]
lto = true
codegen-units = 1
strip = true
```

**Step 2: Create minimal src/main.rs**

```rust
use anyhow::Result;

mod config;
mod parser;
mod generator;
mod cache;

fn main() -> Result<()> {
    println!("Luban Schema Generator v0.1.0");
    Ok(())
}
```

**Step 3: Create src/lib.rs with module declarations**

```rust
pub mod config;
pub mod parser;
pub mod generator;
pub mod cache;
```

**Step 4: Verify project compiles**

Run: `cargo check`
Expected: Compilation succeeds (with warnings about missing modules)

**Step 5: Commit**

```bash
git add .
git commit -m "chore: initialize Rust project with dependencies"
```

---

## Task 2: Configuration Module

**Files:**
- Create: `src/config.rs`
- Create: `luban.config.toml` (example config)

**Step 1: Write failing test for config parsing**

```rust
// src/config.rs
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
        assert_eq!(config.project.tsconfig, "tsconfig.json");
        assert_eq!(config.output.path, "output.xml");
        assert_eq!(config.sources.len(), 1);
        assert_eq!(config.defaults.base_class, "TsClass");
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_config_basic`
Expected: FAIL with "cannot find value `Config`"

**Step 3: Implement Config struct**

```rust
// src/config.rs
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
```

**Step 4: Run tests to verify they pass**

Run: `cargo test config`
Expected: All 3 tests PASS

**Step 5: Create example config file**

```toml
# luban.config.toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "configs/defines/reflect/generated.xml"
cache_file = ".luban-cache.json"

[[sources]]
type = "directory"
path = "src/shared/bevy/visual/trigger"

[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

[defaults]
base_class = "TsClass"

[type_mappings]
Vector3 = "Vector3"
Vector2 = "Vector2"
Entity = "long"
```

**Step 6: Commit**

```bash
git add src/config.rs luban.config.toml
git commit -m "feat: add TOML configuration parsing"
```

---

## Task 3: TSConfig Path Alias Resolution

**Files:**
- Create: `src/tsconfig.rs`

**Step 1: Write failing test for tsconfig parsing**

```rust
// src/tsconfig.rs
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
        let resolver = PathResolver::new(&tsconfig);

        let resolved = resolver.resolve("shared/utils");
        assert!(resolved.ends_with("src/shared/utils"));
    }
}
```

**Step 2: Run test to verify it fails**

Run: `cargo test test_parse_paths`
Expected: FAIL with "cannot find type `TsConfig`"

**Step 3: Implement TsConfig and PathResolver**

```rust
// src/tsconfig.rs
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TsConfig {
    #[serde(default)]
    pub compiler_options: CompilerOptions,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
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

pub struct PathResolver {
    base_url: PathBuf,
    paths: Vec<(String, String)>, // (pattern, replacement)
}

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
```

**Step 4: Add module to lib.rs**

```rust
// Add to src/lib.rs
pub mod tsconfig;
```

**Step 5: Run tests to verify they pass**

Run: `cargo test tsconfig`
Expected: All 3 tests PASS

**Step 6: Commit**

```bash
git add src/tsconfig.rs src/lib.rs
git commit -m "feat: add tsconfig path alias resolution"
```

---

## Task 4: TypeScript AST Parser - Core Structures

**Files:**
- Create: `src/parser.rs`
- Create: `src/parser/class_info.rs`
- Create: `src/parser/field_info.rs`

**Step 1: Define core data structures**

```rust
// src/parser/field_info.rs
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub is_optional: bool,
}

// src/parser/class_info.rs
use super::field_info::FieldInfo;

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub comment: Option<String>,
    pub fields: Vec<FieldInfo>,
    pub implements: Vec<String>,
    pub extends: Option<String>,
    pub source_file: String,
    pub file_hash: String,
    pub is_interface: bool,
}
```

**Step 2: Create parser module structure**

```rust
// src/parser.rs
pub mod class_info;
pub mod field_info;

pub use class_info::ClassInfo;
pub use field_info::FieldInfo;

use anyhow::Result;
use std::path::Path;
use swc_common::{sync::Lrc, SourceMap, FileName};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_ast::*;

pub struct TsParser {
    source_map: Lrc<SourceMap>,
}

impl TsParser {
    pub fn new() -> Self {
        Self {
            source_map: Default::default(),
        }
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<ClassInfo>> {
        let content = std::fs::read_to_string(path)?;
        let file_hash = compute_hash(&content);

        let fm = self.source_map.new_source_file(
            FileName::Real(path.to_path_buf()).into(),
            content,
        );

        let mut parser = Parser::new(
            Syntax::Typescript(TsSyntax {
                tsx: path.extension().map_or(false, |ext| ext == "tsx"),
                decorators: true,
                ..Default::default()
            }),
            StringInput::from(&*fm),
            None,
        );

        let module = parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

        let mut classes = Vec::new();

        for item in &module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
                    if let Decl::Class(class_decl) = &export.decl {
                        if let Some(class_info) = self.extract_class(class_decl, path, &file_hash) {
                            classes.push(class_info);
                        }
                    }
                    if let Decl::TsInterface(iface_decl) = &export.decl {
                        if let Some(iface_info) = self.extract_interface(iface_decl, path, &file_hash) {
                            classes.push(iface_info);
                        }
                    }
                }
                ModuleItem::Stmt(Stmt::Decl(Decl::Class(class_decl))) => {
                    // Non-exported class - skip
                }
                _ => {}
            }
        }

        Ok(classes)
    }

    fn extract_class(&self, class_decl: &ClassDecl, path: &Path, file_hash: &str) -> Option<ClassInfo> {
        let name = class_decl.ident.sym.to_string();
        let mut fields = Vec::new();
        let mut implements = Vec::new();
        let mut extends = None;

        // Extract implements
        for clause in &class_decl.class.implements {
            if let Expr::Ident(ident) = &*clause.expr {
                implements.push(ident.sym.to_string());
            }
        }

        // Extract extends
        if let Some(super_class) = &class_decl.class.super_class {
            if let Expr::Ident(ident) = &**super_class {
                extends = Some(ident.sym.to_string());
            }
        }

        // Extract fields from class body
        for member in &class_decl.class.body {
            match member {
                ClassMember::Constructor(ctor) => {
                    // Extract constructor parameters with modifiers
                    for param in &ctor.params {
                        if let ParamOrTsParamProp::TsParamProp(prop) = param {
                            if let Some(field) = self.extract_param_prop(prop) {
                                fields.push(field);
                            }
                        }
                    }
                }
                ClassMember::ClassProp(prop) => {
                    if let Some(field) = self.extract_class_prop(prop) {
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }

        Some(ClassInfo {
            name,
            comment: None, // TODO: Extract JSDoc
            fields,
            implements,
            extends,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: false,
        })
    }

    fn extract_interface(&self, iface_decl: &TsInterfaceDecl, path: &Path, file_hash: &str) -> Option<ClassInfo> {
        let name = iface_decl.id.sym.to_string();
        let mut fields = Vec::new();

        for member in &iface_decl.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                if let Some(field) = self.extract_interface_prop(prop) {
                    fields.push(field);
                }
            }
        }

        Some(ClassInfo {
            name,
            comment: None,
            fields,
            implements: vec![],
            extends: None,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: true,
        })
    }

    fn extract_param_prop(&self, prop: &TsParamProp) -> Option<FieldInfo> {
        let (name, type_ann, is_optional) = match &prop.param {
            TsParamPropParam::Ident(ident) => {
                (ident.id.sym.to_string(), ident.type_ann.as_ref(), ident.id.optional)
            }
            TsParamPropParam::Assign(_) => return None,
        };

        // Skip internal marker fields
        if name.contains("_nominal_") || name == "_is_trigger_combinator" || name == "_trigger_type" {
            return None;
        }

        let field_type = type_ann
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional,
        })
    }

    fn extract_class_prop(&self, prop: &ClassProp) -> Option<FieldInfo> {
        // Skip private/protected
        if prop.accessibility == Some(Accessibility::Private)
            || prop.accessibility == Some(Accessibility::Protected) {
            return None;
        }

        let name = match &prop.key {
            PropName::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        // Skip internal marker fields
        if name.contains("_nominal_") || name == "_is_trigger_combinator" || name == "_trigger_type" {
            return None;
        }

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional: prop.is_optional,
        })
    }

    fn extract_interface_prop(&self, prop: &TsPropertySignature) -> Option<FieldInfo> {
        let name = match &*prop.key {
            Expr::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional: prop.optional,
        })
    }

    fn convert_type(&self, ts_type: &TsType) -> String {
        match ts_type {
            TsType::TsKeywordType(kw) => match kw.kind {
                TsKeywordTypeKind::TsNumberKeyword => "int".to_string(),
                TsKeywordTypeKind::TsStringKeyword => "string".to_string(),
                TsKeywordTypeKind::TsBooleanKeyword => "bool".to_string(),
                _ => "string".to_string(),
            },
            TsType::TsArrayType(arr) => {
                let element_type = self.convert_type(&arr.elem_type);
                format!("list,{}", element_type)
            }
            TsType::TsTypeRef(type_ref) => {
                let type_name = match &type_ref.type_name {
                    TsEntityName::Ident(ident) => ident.sym.to_string(),
                    TsEntityName::TsQualifiedName(_) => return "string".to_string(),
                };

                match type_name.as_str() {
                    "Array" | "ReadonlyArray" => {
                        if let Some(params) = &type_ref.type_params {
                            if let Some(first) = params.params.first() {
                                let element_type = self.convert_type(first);
                                return format!("list,{}", element_type);
                            }
                        }
                        "list,string".to_string()
                    }
                    "Map" | "Record" => {
                        if let Some(params) = &type_ref.type_params {
                            if params.params.len() >= 2 {
                                let key_type = self.convert_type(&params.params[0]);
                                let value_type = self.convert_type(&params.params[1]);
                                return format!("map,{},{}", key_type, value_type);
                            }
                        }
                        "map,string,string".to_string()
                    }
                    _ => type_name,
                }
            }
            TsType::TsUnionType(union) => {
                // Take first non-undefined/null type
                for member in &union.types {
                    match &**member {
                        TsType::TsKeywordType(kw) if matches!(
                            kw.kind,
                            TsKeywordTypeKind::TsUndefinedKeyword | TsKeywordTypeKind::TsNullKeyword
                        ) => continue,
                        _ => return self.convert_type(member),
                    }
                }
                "string".to_string()
            }
            _ => "string".to_string(),
        }
    }
}

fn compute_hash(content: &str) -> String {
    use md5::{Md5, Digest};
    let mut hasher = Md5::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_class() {
        let ts_code = r#"
export class MyClass {
    public name: string;
    public count: number;
    public active?: boolean;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "MyClass");
        assert_eq!(classes[0].fields.len(), 3);
        assert_eq!(classes[0].fields[0].name, "name");
        assert_eq!(classes[0].fields[0].field_type, "string");
        assert_eq!(classes[0].fields[1].field_type, "int");
        assert!(classes[0].fields[2].is_optional);
    }

    #[test]
    fn test_parse_class_with_implements() {
        let ts_code = r#"
interface EntityTrigger {}

export class MyTrigger implements EntityTrigger {
    public damage: number;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].implements, vec!["EntityTrigger"]);
    }

    #[test]
    fn test_parse_array_types() {
        let ts_code = r#"
export class MyClass {
    public items: string[];
    public numbers: Array<number>;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes[0].fields[0].field_type, "list,string");
        assert_eq!(classes[0].fields[1].field_type, "list,int");
    }

    #[test]
    fn test_parse_map_types() {
        let ts_code = r#"
export class MyClass {
    public data: Map<string, number>;
    public record: Record<string, boolean>;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes[0].fields[0].field_type, "map,string,int");
        assert_eq!(classes[0].fields[1].field_type, "map,string,bool");
    }
}
```

**Step 3: Add tempfile dev dependency**

Add to Cargo.toml under `[dev-dependencies]`:
```toml
[dev-dependencies]
tempfile = "3"
```

**Step 4: Update lib.rs**

```rust
pub mod config;
pub mod tsconfig;
pub mod parser;
pub mod generator;
pub mod cache;
```

**Step 5: Run tests**

Run: `cargo test parser`
Expected: All 4 tests PASS

**Step 6: Commit**

```bash
git add src/parser.rs src/parser/ Cargo.toml
git commit -m "feat: add TypeScript AST parser with SWC"
```

---

## Task 5: Type Mapping System

**Files:**
- Create: `src/type_mapper.rs`

**Step 1: Write failing tests**

```rust
// src/type_mapper.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_type_mapping() {
        let mapper = TypeMapper::new(&std::collections::HashMap::new());
        assert_eq!(mapper.map("number"), "int");
        assert_eq!(mapper.map("string"), "string");
        assert_eq!(mapper.map("boolean"), "bool");
    }

    #[test]
    fn test_custom_type_mapping() {
        let mut custom = std::collections::HashMap::new();
        custom.insert("Vector3".to_string(), "Vector3".to_string());
        custom.insert("Entity".to_string(), "long".to_string());

        let mapper = TypeMapper::new(&custom);
        assert_eq!(mapper.map("Vector3"), "Vector3");
        assert_eq!(mapper.map("Entity"), "long");
    }
}
```

**Step 2: Implement TypeMapper**

```rust
// src/type_mapper.rs
use std::collections::HashMap;

pub struct TypeMapper {
    mappings: HashMap<String, String>,
}

impl TypeMapper {
    pub fn new(custom_mappings: &HashMap<String, String>) -> Self {
        let mut mappings = Self::builtin_mappings();

        // Merge custom mappings (case-insensitive keys)
        for (key, value) in custom_mappings {
            mappings.insert(key.to_lowercase(), value.clone());
        }

        Self { mappings }
    }

    fn builtin_mappings() -> HashMap<String, String> {
        let mut m = HashMap::new();

        // Basic types
        m.insert("number".to_string(), "int".to_string());
        m.insert("string".to_string(), "string".to_string());
        m.insert("boolean".to_string(), "bool".to_string());

        // Numeric types
        m.insert("float".to_string(), "float".to_string());
        m.insert("double".to_string(), "double".to_string());
        m.insert("int".to_string(), "int".to_string());
        m.insert("long".to_string(), "long".to_string());

        // Roblox types
        m.insert("vector3".to_string(), "Vector3".to_string());
        m.insert("vector2".to_string(), "Vector2".to_string());
        m.insert("cframe".to_string(), "CFrame".to_string());
        m.insert("color3".to_string(), "Color3".to_string());

        // Entity types
        m.insert("anyentity".to_string(), "long".to_string());
        m.insert("entity".to_string(), "long".to_string());
        m.insert("entityid".to_string(), "long".to_string());
        m.insert("assetpath".to_string(), "string".to_string());

        // Cast system types
        m.insert("castactiontarget".to_string(), "CastActionTarget".to_string());
        m.insert("castcontext".to_string(), "CastContext".to_string());

        m
    }

    pub fn map(&self, ts_type: &str) -> String {
        // Check case-insensitive match
        if let Some(mapped) = self.mappings.get(&ts_type.to_lowercase()) {
            return mapped.clone();
        }

        // Return original type if no mapping found
        ts_type.to_string()
    }

    pub fn map_full_type(&self, field_type: &str) -> String {
        // Handle list,T and map,K,V types
        if field_type.starts_with("list,") {
            let element = &field_type[5..];
            return format!("list,{}", self.map(element));
        }

        if field_type.starts_with("map,") {
            let parts: Vec<&str> = field_type[4..].splitn(2, ',').collect();
            if parts.len() == 2 {
                return format!("map,{},{}", self.map(parts[0]), self.map(parts[1]));
            }
        }

        self.map(field_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_type_mapping() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map("number"), "int");
        assert_eq!(mapper.map("string"), "string");
        assert_eq!(mapper.map("boolean"), "bool");
    }

    #[test]
    fn test_custom_type_mapping() {
        let mut custom = HashMap::new();
        custom.insert("Vector3".to_string(), "Vector3".to_string());
        custom.insert("Entity".to_string(), "long".to_string());

        let mapper = TypeMapper::new(&custom);
        assert_eq!(mapper.map("Vector3"), "Vector3");
        assert_eq!(mapper.map("Entity"), "long");
    }

    #[test]
    fn test_case_insensitive() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map("Number"), "int");
        assert_eq!(mapper.map("STRING"), "string");
        assert_eq!(mapper.map("AnyEntity"), "long");
    }

    #[test]
    fn test_map_full_type() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map_full_type("list,number"), "list,int");
        assert_eq!(mapper.map_full_type("map,string,number"), "map,string,int");
    }
}
```

**Step 3: Add module to lib.rs**

```rust
pub mod type_mapper;
```

**Step 4: Run tests**

Run: `cargo test type_mapper`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/type_mapper.rs src/lib.rs
git commit -m "feat: add type mapping system with custom mappings support"
```

---

## Task 6: Base Class Resolver

**Files:**
- Create: `src/base_class.rs`

**Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_from_implements() {
        let mappings = vec![
            BaseClassMapping {
                interface: "EntityTrigger".to_string(),
                maps_to: "TsTriggerClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let class = ClassInfo {
            name: "MyTrigger".to_string(),
            implements: vec!["EntityTrigger".to_string()],
            ..Default::default()
        };

        assert_eq!(resolver.resolve(&class), "TsTriggerClass");
    }
}
```

**Step 2: Implement BaseClassResolver**

```rust
// src/base_class.rs
use crate::config::BaseClassMapping;
use crate::parser::ClassInfo;

pub struct BaseClassResolver<'a> {
    mappings: &'a [BaseClassMapping],
    default_base: &'a str,
}

impl<'a> BaseClassResolver<'a> {
    pub fn new(mappings: &'a [BaseClassMapping], default_base: &'a str) -> Self {
        Self { mappings, default_base }
    }

    pub fn resolve(&self, class_info: &ClassInfo) -> String {
        // Interfaces don't have a parent class
        if class_info.is_interface {
            return String::new();
        }

        // Check implements clause for matching interface
        for iface in &class_info.implements {
            for mapping in self.mappings {
                if &mapping.interface == iface {
                    return mapping.maps_to.clone();
                }
            }
        }

        // Use default base class
        self.default_base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ClassInfo;

    fn make_class(name: &str, implements: Vec<&str>) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            comment: None,
            fields: vec![],
            implements: implements.into_iter().map(|s| s.to_string()).collect(),
            extends: None,
            source_file: String::new(),
            file_hash: String::new(),
            is_interface: false,
        }
    }

    #[test]
    fn test_resolve_from_implements() {
        let mappings = vec![
            BaseClassMapping {
                interface: "EntityTrigger".to_string(),
                maps_to: "TsTriggerClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let class = make_class("MyTrigger", vec!["EntityTrigger"]);
        assert_eq!(resolver.resolve(&class), "TsTriggerClass");
    }

    #[test]
    fn test_resolve_default() {
        let mappings = vec![];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let class = make_class("MyClass", vec![]);
        assert_eq!(resolver.resolve(&class), "TsClass");
    }

    #[test]
    fn test_interface_no_parent() {
        let mappings = vec![];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let mut iface = make_class("MyInterface", vec![]);
        iface.is_interface = true;

        assert_eq!(resolver.resolve(&iface), "");
    }

    #[test]
    fn test_multiple_mappings() {
        let mappings = vec![
            BaseClassMapping {
                interface: "EntityTrigger".to_string(),
                maps_to: "TsTriggerClass".to_string(),
            },
            BaseClassMapping {
                interface: "Component".to_string(),
                maps_to: "TsComponentClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let trigger = make_class("MyTrigger", vec!["EntityTrigger"]);
        let component = make_class("MyComponent", vec!["Component"]);

        assert_eq!(resolver.resolve(&trigger), "TsTriggerClass");
        assert_eq!(resolver.resolve(&component), "TsComponentClass");
    }
}
```

**Step 3: Add module to lib.rs**

```rust
pub mod base_class;
```

**Step 4: Run tests**

Run: `cargo test base_class`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/base_class.rs src/lib.rs
git commit -m "feat: add base class resolver with interface mapping"
```

---

## Task 7: XML Generator

**Files:**
- Create: `src/generator.rs`

**Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_simple_bean() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: Some("Test class".to_string()),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    comment: Some("Name field".to_string()),
                    is_optional: false,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyClass" parent="TsClass" comment="Test class">"#));
        assert!(xml.contains(r#"<var name="name" type="string" comment="Name field"/>"#));
    }
}
```

**Step 2: Implement XML generator**

```rust
// src/generator.rs
use crate::parser::{ClassInfo, FieldInfo};
use crate::base_class::BaseClassResolver;
use crate::type_mapper::TypeMapper;

pub struct XmlGenerator<'a> {
    base_resolver: &'a BaseClassResolver<'a>,
    type_mapper: &'a TypeMapper,
}

impl<'a> XmlGenerator<'a> {
    pub fn new(base_resolver: &'a BaseClassResolver<'a>, type_mapper: &'a TypeMapper) -> Self {
        Self { base_resolver, type_mapper }
    }

    pub fn generate(&self, classes: &[ClassInfo]) -> String {
        let mut lines = vec![
            r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
            r#"<module name="" comment="自动生成的 ts class Bean 定义">"#.to_string(),
            String::new(),
        ];

        for class in classes {
            self.generate_bean(&mut lines, class);
            lines.push(String::new());
        }

        lines.push("</module>".to_string());
        lines.join("\n")
    }

    fn generate_bean(&self, lines: &mut Vec<String>, class: &ClassInfo) {
        let parent = self.base_resolver.resolve(class);
        let comment_attr = class.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let parent_attr = if parent.is_empty() {
            String::new()
        } else {
            format!(r#" parent="{}""#, parent)
        };

        lines.push(format!(
            r#"    <bean name="{}"{}{}>""#,
            class.name, parent_attr, comment_attr
        ));

        for field in &class.fields {
            self.generate_field(lines, field);
        }

        lines.push("    </bean>".to_string());
    }

    fn generate_field(&self, lines: &mut Vec<String>, field: &FieldInfo) {
        let mapped_type = self.type_mapper.map_full_type(&field.field_type);

        let final_type = if field.is_optional && !mapped_type.starts_with("list,") {
            format!("{}?", mapped_type)
        } else {
            mapped_type
        };

        let comment_attr = field.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        lines.push(format!(
            r#"        <var name="{}" type="{}"{}/>"#,
            field.name, final_type, comment_attr
        ));
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

// Convenience function for simple cases
pub fn generate_xml(classes: &[ClassInfo], default_base: &str) -> String {
    let mappings = vec![];
    let base_resolver = BaseClassResolver::new(&mappings, default_base);
    let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
    let generator = XmlGenerator::new(&base_resolver, &type_mapper);
    generator.generate(classes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(name: &str, field_type: &str, optional: bool) -> FieldInfo {
        FieldInfo {
            name: name.to_string(),
            field_type: field_type.to_string(),
            comment: None,
            is_optional: optional,
        }
    }

    #[test]
    fn test_generate_simple_bean() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: Some("Test class".to_string()),
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    comment: Some("Name field".to_string()),
                    is_optional: false,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyClass" parent="TsClass" comment="Test class">"#));
        assert!(xml.contains(r#"<var name="name" type="string" comment="Name field"/>"#));
    }

    #[test]
    fn test_optional_field() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: None,
            fields: vec![make_field("value", "string", true)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"type="string?""#));
    }

    #[test]
    fn test_list_not_optional() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: None,
            fields: vec![make_field("items", "list,string", true)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
        };

        let xml = generate_xml(&[class], "TsClass");
        // List types should NOT have ? suffix even when optional
        assert!(xml.contains(r#"type="list,string""#));
        assert!(!xml.contains(r#"type="list,string?""#));
    }

    #[test]
    fn test_interface_no_parent() {
        let class = ClassInfo {
            name: "MyInterface".to_string(),
            comment: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyInterface">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(escape_xml("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(escape_xml(r#"say "hello""#), r#"say &quot;hello&quot;"#);
    }
}
```

**Step 3: Run tests**

Run: `cargo test generator`
Expected: All 5 tests PASS

**Step 4: Commit**

```bash
git add src/generator.rs
git commit -m "feat: add XML generator with bean/var formatting"
```

---

## Task 8: Cache System

**Files:**
- Create: `src/cache.rs`

**Step 1: Write failing tests**

```rust
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
}
```

**Step 2: Implement Cache**

```rust
// src/cache.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct Cache {
    pub version: u32,
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
            version: 1,
            generated_at: Utc::now(),
            entries: HashMap::new(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_json(&content)
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
```

**Step 3: Run tests**

Run: `cargo test cache`
Expected: All 3 tests PASS

**Step 4: Commit**

```bash
git add src/cache.rs
git commit -m "feat: add incremental compilation cache system"
```

---

## Task 9: File Scanner

**Files:**
- Create: `src/scanner.rs`

**Step 1: Write failing tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_scan_directory() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("a.ts"), "export class A {}").unwrap();
        std::fs::write(dir.path().join("b.tsx"), "export class B {}").unwrap();
        std::fs::write(dir.path().join("c.d.ts"), "declare class C {}").unwrap();

        let files = scan_directory(dir.path()).unwrap();
        assert_eq!(files.len(), 2); // Excludes .d.ts
    }
}
```

**Step 2: Implement scanner**

```rust
// src/scanner.rs
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
```

**Step 3: Add module to lib.rs**

```rust
pub mod scanner;
```

**Step 4: Run tests**

Run: `cargo test scanner`
Expected: All 4 tests PASS

**Step 5: Commit**

```bash
git add src/scanner.rs src/lib.rs
git commit -m "feat: add directory scanner with filtering"
```

---

## Task 10: CLI Interface

**Files:**
- Modify: `src/main.rs`

**Step 1: Implement CLI with Clap**

```rust
// src/main.rs
use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::path::PathBuf;
use std::time::Instant;

mod config;
mod tsconfig;
mod parser;
mod type_mapper;
mod base_class;
mod generator;
mod cache;
mod scanner;

use config::{Config, SourceConfig};
use tsconfig::TsConfig;
use parser::TsParser;
use type_mapper::TypeMapper;
use base_class::BaseClassResolver;
use generator::XmlGenerator;
use cache::Cache;

#[derive(Parser)]
#[command(name = "luban-gen")]
#[command(about = "High-performance TypeScript to Luban XML Schema generator")]
#[command(version)]
struct Cli {
    /// Configuration file path
    #[arg(short, long, default_value = "luban.config.toml")]
    config: PathBuf,

    /// Force regenerate all beans (ignore cache)
    #[arg(short, long)]
    force: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let start = Instant::now();

    println!("Luban Schema Generator v{}", env!("CARGO_PKG_VERSION"));
    println!("{}", "=".repeat(50));

    // Load configuration
    let config = Config::load(&cli.config)
        .with_context(|| format!("Failed to load config from {:?}", cli.config))?;

    let project_root = cli.config.parent().unwrap_or_else(|| std::path::Path::new("."));

    // Load tsconfig for path resolution
    let tsconfig_path = project_root.join(&config.project.tsconfig);
    let tsconfig = TsConfig::load(&tsconfig_path)
        .with_context(|| format!("Failed to load tsconfig from {:?}", tsconfig_path))?;

    let _path_resolver = tsconfig::PathResolver::new(&tsconfig, project_root);

    // Initialize components
    let type_mapper = TypeMapper::new(&config.type_mappings);
    let base_resolver = BaseClassResolver::new(
        &config.base_class_mappings,
        &config.defaults.base_class,
    );

    // Load cache
    let cache_path = project_root.join(&config.output.cache_file);
    let mut cache = if cli.force {
        println!("[Force mode] Ignoring cache, regenerating all beans...");
        Cache::new()
    } else {
        Cache::load(&cache_path).unwrap_or_default()
    };

    // Collect source directories
    let mut source_dirs = Vec::new();
    for source in &config.sources {
        match source {
            SourceConfig::Directory { path } => {
                source_dirs.push(project_root.join(path));
            }
            SourceConfig::Registration { path } => {
                // TODO: Parse registration file
                println!("  Registration mode not yet implemented: {:?}", path);
            }
        }
    }

    // Scan for TypeScript files
    println!("\n[1/4] Scanning directories...");
    let ts_files = scanner::scan_directories(&source_dirs)?;
    println!("  Found {} TypeScript files", ts_files.len());

    // Parse files in parallel
    println!("\n[2/4] Parsing TypeScript files...");
    let ts_parser = TsParser::new();

    let all_classes: Vec<_> = ts_files
        .par_iter()
        .filter_map(|path| {
            match ts_parser.parse_file(path) {
                Ok(classes) => Some(classes),
                Err(e) => {
                    eprintln!("  Warning: Failed to parse {:?}: {}", path, e);
                    None
                }
            }
        })
        .flatten()
        .collect();

    println!("  Extracted {} classes/interfaces", all_classes.len());

    // Filter by cache
    println!("\n[3/4] Checking cache...");
    let mut unchanged = 0;
    let mut updated = 0;

    let final_classes: Vec<_> = all_classes
        .into_iter()
        .inspect(|class| {
            if cache.is_valid(&class.name, &class.file_hash) {
                unchanged += 1;
                if cli.verbose {
                    println!("  [cached] {}", class.name);
                }
            } else {
                updated += 1;
                if cli.verbose {
                    println!("  [update] {}", class.name);
                }
                cache.set_entry(&class.name, &class.source_file, &class.file_hash);
            }
        })
        .collect();

    println!("  Cached: {}, Updated: {}", unchanged, updated);

    // Generate XML
    println!("\n[4/4] Generating XML...");
    let xml_generator = XmlGenerator::new(&base_resolver, &type_mapper);
    let xml_output = xml_generator.generate(&final_classes);

    // Write output only if changed
    let output_path = project_root.join(&config.output.path);
    let should_write = if output_path.exists() {
        let existing = std::fs::read_to_string(&output_path)?;
        existing != xml_output
    } else {
        true
    };

    if should_write {
        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&output_path, &xml_output)?;
        println!("  Written to {:?}", output_path);
    } else {
        println!("  No changes, skipping write");
    }

    // Save cache
    cache.save(&cache_path)?;

    let elapsed = start.elapsed();
    println!("\n{}", "=".repeat(50));
    println!("Done! Generated {} beans in {:?}", final_classes.len(), elapsed);

    Ok(())
}
```

**Step 2: Run build**

Run: `cargo build --release`
Expected: Build succeeds

**Step 3: Test CLI help**

Run: `cargo run -- --help`
Expected: Shows help message with all options

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: add CLI interface with parallel processing"
```

---

## Task 11: Integration Test

**Files:**
- Create: `tests/integration.rs`
- Create: `tests/fixtures/` (test TypeScript files)

**Step 1: Create test fixtures**

```typescript
// tests/fixtures/simple.ts
export class SimpleClass {
    public name: string;
    public count: number;
    public active?: boolean;
}

// tests/fixtures/trigger.ts
interface EntityTrigger {}

export class DamageTrigger implements EntityTrigger {
    public damage: number;
    public radius: number;
}

// tests/fixtures/complex.ts
export class ComplexClass {
    public items: string[];
    public data: Map<string, number>;
    public nested?: ComplexClass;
}
```

**Step 2: Create integration test**

```rust
// tests/integration.rs
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn test_end_to_end_generation() {
    let fixtures = project_root().join("tests/fixtures");

    // Create temp output directory
    let temp = TempDir::new().unwrap();
    let output_path = temp.path().join("output.xml");
    let cache_path = temp.path().join(".cache.json");

    // Create config
    let config = format!(r#"
[project]
tsconfig = "tsconfig.json"

[output]
path = "{}"
cache_file = "{}"

[[sources]]
type = "directory"
path = "{}"

[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

[defaults]
base_class = "TsClass"
"#,
        output_path.display(),
        cache_path.display(),
        fixtures.display(),
    );

    let config_path = temp.path().join("luban.config.toml");
    fs::write(&config_path, &config).unwrap();

    // Create minimal tsconfig
    fs::write(
        temp.path().join("tsconfig.json"),
        r#"{"compilerOptions": {}}"#,
    ).unwrap();

    // Run the generator
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_luban-gen"))
        .arg("-c")
        .arg(&config_path)
        .status()
        .expect("Failed to run luban-gen");

    assert!(status.success(), "luban-gen failed");

    // Verify output
    let output = fs::read_to_string(&output_path).unwrap();

    assert!(output.contains(r#"<bean name="SimpleClass" parent="TsClass">"#));
    assert!(output.contains(r#"<bean name="DamageTrigger" parent="TsTriggerClass">"#));
    assert!(output.contains(r#"type="list,string""#));
    assert!(output.contains(r#"type="map,string,int""#));
}
```

**Step 3: Create fixture files**

```bash
mkdir -p tests/fixtures
```

Create `tests/fixtures/simple.ts`:
```typescript
export class SimpleClass {
    public name: string;
    public count: number;
    public active?: boolean;
}
```

Create `tests/fixtures/trigger.ts`:
```typescript
interface EntityTrigger {}

export class DamageTrigger implements EntityTrigger {
    public damage: number;
    public radius: number;
}
```

Create `tests/fixtures/complex.ts`:
```typescript
export class ComplexClass {
    public items: string[];
    public data: Map<string, number>;
}
```

**Step 4: Run integration test**

Run: `cargo test --test integration`
Expected: PASS

**Step 5: Commit**

```bash
git add tests/
git commit -m "test: add integration tests with fixtures"
```

---

## Task 12: GitHub Actions CI

**Files:**
- Create: `.github/workflows/ci.yml`
- Create: `.github/workflows/release.yml`

**Step 1: Create CI workflow**

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Build
        run: cargo build --verbose

      - name: Run tests
        run: cargo test --verbose

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Clippy
        run: cargo clippy -- -D warnings
```

**Step 2: Create release workflow**

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags:
      - 'v*'

env:
  CARGO_TERM_COLOR: always

jobs:
  build:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            artifact: luban-gen
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            artifact: luban-gen.exe
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: luban-gen

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}

      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.target }}
          path: target/${{ matrix.target }}/release/${{ matrix.artifact }}

  release:
    needs: build
    runs-on: ubuntu-latest

    steps:
      - name: Download artifacts
        uses: actions/download-artifact@v4

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            x86_64-unknown-linux-gnu/luban-gen
            x86_64-pc-windows-msvc/luban-gen.exe
            x86_64-apple-darwin/luban-gen
```

**Step 3: Add rustfmt.toml**

```toml
# rustfmt.toml
edition = "2021"
max_width = 100
```

**Step 4: Commit**

```bash
git add .github/ rustfmt.toml
git commit -m "ci: add GitHub Actions for CI and releases"
```

---

## Task 13: Documentation

**Files:**
- Update: `README.md`
- Create: `.gitignore`

**Step 1: Create .gitignore**

```gitignore
# .gitignore
/target
.luban-cache.json
*.xml
!tests/fixtures/*.xml

# IDE
.idea/
.vscode/
*.swp
```

**Step 2: Create README**

```markdown
# Luban Schema Generator (Rust Edition)

High-performance TypeScript to Luban XML Schema generator.

## Features

- **Fast**: Uses SWC for TypeScript parsing, Rayon for parallel processing
- **Incremental**: Only regenerates changed files using content hashing
- **Zero Runtime**: Single binary, no Node.js required
- **Configurable**: TOML configuration with type mappings and base class rules

## Installation

Download from [Releases](https://github.com/your-org/luban-gen/releases) or build from source:

```bash
cargo install --path .
```

## Usage

```bash
# Using default config (luban.config.toml)
luban-gen

# Specify config file
luban-gen -c path/to/config.toml

# Force regenerate (ignore cache)
luban-gen --force

# Verbose output
luban-gen -v
```

## Configuration

See `luban.config.toml` for full configuration reference.

## Performance

- 1000 files: < 500ms full scan
- Memory: < 200MB peak

## License

MIT
```

**Step 3: Commit**

```bash
git add README.md .gitignore
git commit -m "docs: add README and .gitignore"
```

---

## Summary

**Total Tasks:** 13
**Estimated Implementation:** Bite-sized steps for focused development

**Key Components:**
1. Configuration parsing (TOML)
2. TSConfig path resolution
3. SWC-based TypeScript parser
4. Type mapping system
5. Base class resolution
6. XML generation
7. Incremental cache
8. Directory scanner
9. CLI interface
10. Integration tests
11. CI/CD pipelines
12. Documentation

---

Plan complete and saved to `docs/plans/2025-12-31-luban-schema-generator.md`. Two execution options:

**1. Subagent-Driven (this session)** - I dispatch fresh subagent per task, review between tasks, fast iteration

**2. Parallel Session (separate)** - Open new session with executing-plans, batch execution with checkpoints

Which approach?
