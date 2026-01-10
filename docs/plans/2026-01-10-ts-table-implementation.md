# TypeScript Table 代码生成实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 由 ts-to-luban (Rust) 生成 TypeScript table 代码，取代 Luban codebuild。

**Architecture:**
1. 扩展配置支持 `table_output_path`
2. 扩展 FieldInfo 识别 `ObjectFactory<T>` 泛型
3. 新增 `ts_generator` 模块生成 creators、tables、registry、index.ts

**Tech Stack:** Rust, SWC (TypeScript 解析), TOML (配置)

---

## Phase 1: 配置扩展

### Task 1.1: 添加 table_output_path 配置字段

**Files:**
- Modify: `src/config.rs:27-40`
- Test: `src/config.rs` (existing tests section)

**Step 1: 在 OutputConfig 添加 table_output_path 字段**

```rust
// src/config.rs - OutputConfig struct
#[derive(Debug, Deserialize)]
pub struct OutputConfig {
    pub path: PathBuf,
    #[serde(default = "default_cache_file")]
    pub cache_file: PathBuf,
    #[serde(default)]
    pub module_name: String,
    #[serde(default)]
    pub enum_path: Option<PathBuf>,
    #[serde(default)]
    pub bean_types_path: Option<PathBuf>,
    /// Path to output TypeScript table code
    #[serde(default)]
    pub table_output_path: Option<PathBuf>,
}
```

**Step 2: 添加配置解析测试**

```rust
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
    assert_eq!(config.output.table_output_path, Some(PathBuf::from("out/tables")));
}
```

**Step 3: 运行测试**

Run: `cargo test test_parse_table_output_path`
Expected: PASS

**Step 4: Commit**

```bash
git add src/config.rs
git commit -m "feat: add table_output_path config option"
```

---

## Phase 2: FieldInfo 扩展

### Task 2.1: 添加 ObjectFactory 字段信息

**Files:**
- Modify: `src/parser/field_info.rs`

**Step 1: 扩展 FieldInfo 结构**

```rust
// src/parser/field_info.rs
#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub is_optional: bool,
    pub validators: FieldValidators,
    /// Whether this field is ObjectFactory<T> type
    pub is_object_factory: bool,
    /// Inner type T for ObjectFactory<T>
    pub factory_inner_type: Option<String>,
    /// Original TypeScript type (before mapping)
    pub original_type: String,
}
```

**Step 2: 更新 Default 实现或构造函数**

在解析器创建 FieldInfo 时，需要设置新字段的默认值。

**Step 3: Commit**

```bash
git add src/parser/field_info.rs
git commit -m "feat: add ObjectFactory fields to FieldInfo"
```

---

### Task 2.2: 解析 ObjectFactory<T> 泛型

**Files:**
- Modify: `src/parser.rs` (parse_field 函数)

**Step 1: 在解析字段类型时检测 ObjectFactory**

```rust
// 在 parse_field 函数中，解析类型后：
let (is_object_factory, factory_inner_type, effective_type) =
    detect_object_factory(&field_type_str);

// detect_object_factory 函数
fn detect_object_factory(type_str: &str) -> (bool, Option<String>, String) {
    // Match ObjectFactory<T> pattern
    let re = regex::Regex::new(r"^ObjectFactory<(.+)>$").unwrap();
    if let Some(caps) = re.captures(type_str) {
        let inner = caps.get(1).unwrap().as_str().to_string();
        (true, Some(inner.clone()), inner)
    } else {
        (false, None, type_str.to_string())
    }
}
```

**Step 2: 添加测试**

```rust
#[test]
fn test_parse_object_factory_field() {
    let code = r#"
    export class TestClass {
        factory: ObjectFactory<SomeBean>;
        factories: ObjectFactory<BaseType>[];
    }
    "#;
    // Parse and verify is_object_factory = true
}
```

**Step 3: Commit**

```bash
git add src/parser.rs src/parser/field_info.rs
git commit -m "feat: detect ObjectFactory<T> generic in field parsing"
```

---

## Phase 3: TypeScript 代码生成器

### Task 3.1: 创建 ts_generator 模块骨架

**Files:**
- Create: `src/ts_generator.rs`
- Create: `src/ts_generator/mod.rs`
- Create: `src/ts_generator/creator_gen.rs`
- Create: `src/ts_generator/table_gen.rs`
- Create: `src/ts_generator/registry_gen.rs`
- Create: `src/ts_generator/index_gen.rs`
- Modify: `src/lib.rs`

**Step 1: 创建模块结构**

```rust
// src/ts_generator/mod.rs
mod creator_gen;
mod table_gen;
mod registry_gen;
mod index_gen;

pub use creator_gen::CreatorGenerator;
pub use table_gen::TableGenerator;
pub use registry_gen::RegistryGenerator;
pub use index_gen::IndexGenerator;

use std::path::PathBuf;
use crate::parser::ClassInfo;

/// Main TypeScript code generator
pub struct TsCodeGenerator {
    output_path: PathBuf,
    classes: Vec<ClassInfo>,
}

impl TsCodeGenerator {
    pub fn new(output_path: PathBuf, classes: Vec<ClassInfo>) -> Self {
        Self { output_path, classes }
    }

    pub fn generate(&self) -> anyhow::Result<()> {
        // TODO: implement
        Ok(())
    }
}
```

**Step 2: 添加到 lib.rs**

```rust
// src/lib.rs
pub mod ts_generator;
pub use ts_generator::TsCodeGenerator;
```

**Step 3: Commit**

```bash
git add src/ts_generator/ src/lib.rs
git commit -m "feat: create ts_generator module skeleton"
```

---

### Task 3.2: 实现 Import 路径计算

**Files:**
- Create: `src/ts_generator/import_resolver.rs`

**Step 1: 创建 ImportResolver**

```rust
// src/ts_generator/import_resolver.rs
use std::path::{Path, PathBuf};

pub struct ImportResolver {
    /// tsconfig paths mapping
    paths: std::collections::HashMap<String, Vec<String>>,
    /// Base URL from tsconfig
    base_url: Option<PathBuf>,
}

impl ImportResolver {
    pub fn new(tsconfig: &crate::tsconfig::TsConfig) -> Self {
        Self {
            paths: tsconfig.compiler_options.paths.clone().unwrap_or_default(),
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
        if let Some(idx) = path_str.find("node_modules") {
            let after = &path_str[idx + "node_modules/".len()..];
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
        // Use pathdiff crate or manual calculation
        let from_dir = from.parent().unwrap_or(Path::new("."));
        pathdiff::diff_paths(to, from_dir)
            .map(|p| {
                let s = p.to_string_lossy().replace('\\', "/");
                // Remove .ts extension
                let s = s.trim_end_matches(".ts");
                if s.starts_with('.') { s.to_string() } else { format!("./{}", s) }
            })
            .unwrap_or_else(|| to.to_string_lossy().to_string())
    }
}
```

**Step 2: 添加 pathdiff 依赖**

Run: `cargo add pathdiff`

**Step 3: 添加测试**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_relative_path() {
        let resolver = ImportResolver::new(&Default::default());
        let from = Path::new("out/tables/item-table.ts");
        let to = Path::new("src/types/item.ts");
        let result = resolver.calculate_relative_path(from, to);
        assert!(result.contains("../"));
    }

    #[test]
    fn test_node_modules_package() {
        let resolver = ImportResolver::new(&Default::default());
        let path = Path::new("node_modules/@white-dragon-bevy/ts-to-luban/src/index.ts");
        let result = resolver.extract_package_name(path);
        assert_eq!(result, "@white-dragon-bevy/ts-to-luban");
    }
}
```

**Step 4: Commit**

```bash
git add src/ts_generator/import_resolver.rs Cargo.toml Cargo.lock
git commit -m "feat: add ImportResolver for path calculation"
```

---

### Task 3.3: 实现 Creator 生成器

**Files:**
- Modify: `src/ts_generator/creator_gen.rs`

**Step 1: 实现基本 Creator 生成**

```rust
// src/ts_generator/creator_gen.rs
use crate::parser::{ClassInfo, FieldInfo};
use super::import_resolver::ImportResolver;

pub struct CreatorGenerator<'a> {
    import_resolver: &'a ImportResolver,
    all_classes: &'a [ClassInfo],
}

impl<'a> CreatorGenerator<'a> {
    pub fn new(import_resolver: &'a ImportResolver, all_classes: &'a [ClassInfo]) -> Self {
        Self { import_resolver, all_classes }
    }

    /// Generate creator file content for a class
    pub fn generate(&self, class: &ClassInfo, output_file: &std::path::Path) -> String {
        let mut lines = vec![
            "// Auto-generated by ts-to-luban".to_string(),
            format!("import {{ {} }} from \"{}\";",
                class.name,
                self.import_resolver.resolve(output_file, std::path::Path::new(&class.source_file))
            ),
            "import { createBean } from \"../registry\";".to_string(),
            String::new(),
        ];

        // Generate creator function
        lines.push(format!("export function create{}(json: any): {} {{", class.name, class.name));
        lines.push(format!("    const obj = new {}();", class.name));

        for field in &class.fields {
            let assignment = self.generate_field_assignment(field);
            lines.push(format!("    {}", assignment));
        }

        lines.push("    return obj;".to_string());
        lines.push("}".to_string());

        lines.join("\n")
    }

    fn generate_field_assignment(&self, field: &FieldInfo) -> String {
        let name = &field.name;

        if field.is_object_factory {
            // ObjectFactory<T> → factory function
            if field.field_type.contains("[]") || field.field_type.starts_with("list,") {
                format!("obj.{} = (json.{} as any[]).map(item => {{ const data = item; return () => createBean(data.$type, data); }});", name, name)
            } else {
                format!("obj.{} = (() => {{ const data = json.{}; return () => createBean(data.$type, data); }})();", name, name)
            }
        } else if self.is_bean_type(&field.original_type) {
            // Nested bean
            let bean_name = self.extract_bean_name(&field.original_type);
            if field.field_type.contains("[]") || field.field_type.starts_with("list,") {
                format!("obj.{} = (json.{} as any[]).map(item => createBean(\"{}\", item));", name, name, bean_name)
            } else {
                format!("obj.{} = createBean(\"{}\", json.{});", name, bean_name, name)
            }
        } else {
            // Primitive type
            format!("obj.{} = json.{};", name, name)
        }
    }

    fn is_bean_type(&self, type_str: &str) -> bool {
        self.all_classes.iter().any(|c| c.name == type_str || type_str.contains(&c.name))
    }

    fn extract_bean_name(&self, type_str: &str) -> String {
        // Extract class name from type like "DropItem" or "DropItem[]"
        type_str.trim_end_matches("[]").to_string()
    }
}
```

**Step 2: Commit**

```bash
git add src/ts_generator/creator_gen.rs
git commit -m "feat: implement CreatorGenerator"
```

---

### Task 3.4: 实现 Table 生成器

**Files:**
- Modify: `src/ts_generator/table_gen.rs`

**Step 1: 实现 Table 生成**

```rust
// src/ts_generator/table_gen.rs
use crate::parser::ClassInfo;

pub struct TableGenerator;

impl TableGenerator {
    /// Generate table file content for a @LubanTable class
    pub fn generate(class: &ClassInfo, output_file: &std::path::Path) -> String {
        let config = class.luban_table.as_ref().expect("Class must have @LubanTable");
        let class_name = &class.name;
        let table_name = format!("{}Table", class_name);
        let index_field = &config.index;

        match config.mode.as_str() {
            "map" => Self::generate_map_table(class_name, &table_name, index_field),
            "list" => Self::generate_list_table(class_name, &table_name),
            "one" | "singleton" => Self::generate_one_table(class_name, &table_name),
            _ => panic!("Unknown table mode: {}", config.mode),
        }
    }

    fn generate_map_table(class_name: &str, table_name: &str, index_field: &str) -> String {
        format!(r#"// Auto-generated by ts-to-luban
import {{ {} }} from "../creators/{}";
import {{ createBean }} from "../registry";

export interface {} {{
    readonly dataMap: Map<number, {}>;
    readonly dataList: readonly {}[];
    get(key: number): {} | undefined;
}}

export function create{}(json: any): {} {{
    const dataMap = new Map<number, {}>();
    const dataList: {}[] = [];

    for (const [_key, item] of pairs(json)) {{
        const obj = createBean<{}>("{}", item);
        dataList.push(obj);
        dataMap.set(obj.{}, obj);
    }}

    return {{
        dataMap,
        dataList,
        get(key: number) {{ return dataMap.get(key); }}
    }};
}}
"#,
            class_name, to_kebab_case(class_name),
            table_name, class_name, class_name, class_name,
            table_name, table_name, class_name, class_name,
            class_name, class_name, index_field
        )
    }

    fn generate_list_table(class_name: &str, table_name: &str) -> String {
        format!(r#"// Auto-generated by ts-to-luban
import {{ createBean }} from "../registry";

export interface {} {{
    readonly dataList: readonly {}[];
}}

export function create{}(json: any): {} {{
    const dataList = (json as any[]).map(item => createBean<{}>("{}", item));
    return {{ dataList }};
}}
"#,
            table_name, class_name,
            table_name, table_name, class_name, class_name
        )
    }

    fn generate_one_table(class_name: &str, table_name: &str) -> String {
        format!(r#"// Auto-generated by ts-to-luban
import {{ createBean }} from "../registry";

export interface {} {{
    readonly data: {};
}}

export function create{}(json: any): {} {{
    const data = createBean<{}>("{}", json);
    return {{ data }};
}}
"#,
            table_name, class_name,
            table_name, table_name, class_name, class_name
        )
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
```

**Step 2: Commit**

```bash
git add src/ts_generator/table_gen.rs
git commit -m "feat: implement TableGenerator for all modes"
```

---

### Task 3.5: 实现 Registry 生成器

**Files:**
- Modify: `src/ts_generator/registry_gen.rs`

**Step 1: 实现 Registry 生成**

```rust
// src/ts_generator/registry_gen.rs
use crate::parser::ClassInfo;

pub struct RegistryGenerator;

impl RegistryGenerator {
    pub fn generate() -> String {
        r#"// Auto-generated by ts-to-luban
// Bean registry for handling circular dependencies and $type polymorphism

type Creator<T> = (json: any) => T;
const beanRegistry: Record<string, Creator<any>> = {};

export function registerCreator(name: string, creator: Creator<any>): void {
    beanRegistry[name] = creator;
}

export function createBean<T>(name: string, json: any): T {
    const creator = beanRegistry[name];
    if (!creator) {
        error(`Unknown bean: ${name}`);
    }
    return creator(json) as T;
}

export function createByType<T>(typeName: string, json: any): T {
    return createBean<T>(typeName, json);
}
"#.to_string()
    }
}
```

**Step 2: Commit**

```bash
git add src/ts_generator/registry_gen.rs
git commit -m "feat: implement RegistryGenerator"
```

---

### Task 3.6: 实现 Index 生成器

**Files:**
- Modify: `src/ts_generator/index_gen.rs`

**Step 1: 实现 Index 生成**

```rust
// src/ts_generator/index_gen.rs
use crate::parser::ClassInfo;

pub struct IndexGenerator;

impl IndexGenerator {
    /// Generate index.ts with all registrations and AllTables
    pub fn generate(all_classes: &[ClassInfo], table_classes: &[&ClassInfo]) -> String {
        let mut lines = vec![
            "// Auto-generated by ts-to-luban".to_string(),
            "import { registerCreator, createBean, createByType } from \"./registry\";".to_string(),
            String::new(),
            "// Import all creators".to_string(),
        ];

        // Import creators
        for class in all_classes {
            lines.push(format!(
                "import {{ create{} }} from \"./creators/{}\";",
                class.name, to_kebab_case(&class.name)
            ));
        }

        lines.push(String::new());
        lines.push("// Register all creators".to_string());

        // Register creators
        for class in all_classes {
            lines.push(format!(
                "registerCreator(\"{}\", create{});",
                class.name, class.name
            ));
        }

        lines.push(String::new());
        lines.push("// Import tables".to_string());

        // Import tables
        for class in table_classes {
            let table_name = format!("{}Table", class.name);
            lines.push(format!(
                "import {{ create{}, {} }} from \"./tables/{}\";",
                table_name, table_name, to_kebab_case(&class.name)
            ));
        }

        lines.push(String::new());

        // Generate AllTables interface
        lines.push("export interface AllTables {".to_string());
        for class in table_classes {
            lines.push(format!("    readonly {}Table: {}Table;", class.name, class.name));
        }
        lines.push("}".to_string());
        lines.push(String::new());

        // Generate createAllTables function
        lines.push("export function createAllTables(loader: (file: string) => unknown): AllTables {".to_string());
        lines.push("    return {".to_string());
        for class in table_classes {
            lines.push(format!(
                "        {}Table: create{}Table(loader(\"{}\")),",
                class.name, class.name, to_kebab_case(&class.name)
            ));
        }
        lines.push("    };".to_string());
        lines.push("}".to_string());
        lines.push(String::new());

        // Re-export registry functions
        lines.push("export { createBean, createByType } from \"./registry\";".to_string());

        lines.join("\n")
    }
}

fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
```

**Step 2: Commit**

```bash
git add src/ts_generator/index_gen.rs
git commit -m "feat: implement IndexGenerator"
```

---

## Phase 4: 主程序集成

### Task 4.1: 集成 TsCodeGenerator 到 main.rs

**Files:**
- Modify: `src/main.rs`

**Step 1: 添加 TypeScript 代码生成调用**

在 XML 生成后，如果配置了 `table_output_path`，调用 TypeScript 生成器：

```rust
// In main.rs, after XML generation

// Generate TypeScript table code if configured
if let Some(table_output_path) = &config.output.table_output_path {
    println!("\n[5/5] Generating TypeScript table code...");

    let resolved_path = project_root.join(table_output_path);
    let ts_generator = ts_generator::TsCodeGenerator::new(
        resolved_path,
        final_classes.clone(),
        &tsconfig,
    );

    ts_generator.generate()?;
    println!("  Written TypeScript tables to {:?}", table_output_path);
}
```

**Step 2: 更新进度提示**

将 `[4/4]` 改为 `[4/5]`（或动态计算）。

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "feat: integrate TsCodeGenerator into main"
```

---

### Task 4.2: 实现完整的 TsCodeGenerator

**Files:**
- Modify: `src/ts_generator/mod.rs`

**Step 1: 实现 generate 方法**

```rust
impl TsCodeGenerator {
    pub fn generate(&self) -> anyhow::Result<()> {
        // Create output directories
        std::fs::create_dir_all(self.output_path.join("creators"))?;
        std::fs::create_dir_all(self.output_path.join("tables"))?;

        // Generate registry.ts
        let registry_content = RegistryGenerator::generate();
        std::fs::write(self.output_path.join("registry.ts"), registry_content)?;

        // Generate creators
        for class in &self.classes {
            let file_name = format!("{}.ts", to_kebab_case(&class.name));
            let file_path = self.output_path.join("creators").join(&file_name);
            let content = self.creator_gen.generate(class, &file_path);
            std::fs::write(&file_path, content)?;
        }

        // Generate tables for @LubanTable classes
        let table_classes: Vec<_> = self.classes.iter()
            .filter(|c| c.luban_table.is_some())
            .collect();

        for class in &table_classes {
            let file_name = format!("{}-table.ts", to_kebab_case(&class.name));
            let file_path = self.output_path.join("tables").join(&file_name);
            let content = TableGenerator::generate(class, &file_path);
            std::fs::write(&file_path, content)?;
        }

        // Generate index.ts
        let index_content = IndexGenerator::generate(&self.classes, &table_classes);
        std::fs::write(self.output_path.join("index.ts"), index_content)?;

        Ok(())
    }
}
```

**Step 2: Commit**

```bash
git add src/ts_generator/mod.rs
git commit -m "feat: implement TsCodeGenerator.generate()"
```

---

## Phase 5: 测试

### Task 5.1: 端到端测试

**Files:**
- Test with: `luban-ts/luban.config.toml`

**Step 1: 更新测试配置**

```toml
# luban-ts/luban.config.toml
[output]
path = "configs/defines/examples.xml"
module_name = "examples"
table_output_path = "out/ts-tables"
```

**Step 2: 运行生成**

Run: `cargo run -- -c luban-ts/luban.config.toml -f`

**Step 3: 验证生成的文件**

Check:
- `out/ts-tables/registry.ts` exists
- `out/ts-tables/creators/*.ts` exist
- `out/ts-tables/tables/*.ts` exist
- `out/ts-tables/index.ts` exists

**Step 4: Commit**

```bash
git add luban-ts/luban.config.toml
git commit -m "test: add table_output_path to test config"
```

---

### Task 5.2: TypeScript 编译验证

**Step 1: 尝试编译生成的代码**

```bash
cd luban-ts
npx tsc --noEmit out/ts-tables/*.ts out/ts-tables/**/*.ts
```

**Step 2: 修复任何类型错误**

如果有错误，返回修改生成器代码。

**Step 3: Commit fixes**

```bash
git add .
git commit -m "fix: address TypeScript compilation issues"
```

---

## Summary

Total tasks: 12
Estimated implementation phases: 5

Key modules:
1. `src/config.rs` - 配置扩展
2. `src/parser/field_info.rs` - ObjectFactory 识别
3. `src/ts_generator/` - TypeScript 代码生成器
   - `mod.rs` - 主入口
   - `import_resolver.rs` - Import 路径计算
   - `creator_gen.rs` - Creator 生成
   - `table_gen.rs` - Table 生成
   - `registry_gen.rs` - Registry 生成
   - `index_gen.rs` - Index 生成
