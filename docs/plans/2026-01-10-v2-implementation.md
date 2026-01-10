# V2 装饰器支持实现计划

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 扩展现有 Rust CLI，支持解析 TypeScript 装饰器并生成 Luban 验证器语法和 table 定义。

**Architecture:** 两遍扫描：第一遍收集 `@LubanTable` 类建立 TableRegistry，第二遍解析装饰器生成完整引用。新增 `decorator.rs` 解析装饰器 AST，`validator.rs` 生成验证语法。

**Tech Stack:** Rust, SWC, roblox-ts (npm 包)

---

## Phase 1: npm 包 - luban-ts

### Task 1.1: 初始化 roblox-ts 项目

**Files:**
- Create: `luban-ts/package.json`
- Create: `luban-ts/tsconfig.json`
- Create: `luban-ts/src/index.ts`

**Step 1: 创建 package.json**

```json
{
  "name": "@white-dragon-bevy/ts-to-luban",
  "version": "0.1.0",
  "main": "out/init.lua",
  "types": "out/index.d.ts",
  "files": ["out"],
  "scripts": {
    "build": "rbxtsc",
    "watch": "rbxtsc -w"
  },
  "devDependencies": {
    "@rbxts/compiler-types": "^3.0.0",
    "@rbxts/types": "^1.0.0",
    "roblox-ts": "^3.0.0",
    "typescript": "^5.0.0"
  },
  "peerDependencies": {
    "@rbxts/compiler-types": "^3.0.0"
  }
}
```

**Step 2: 创建 tsconfig.json**

```json
{
  "compilerOptions": {
    "allowSyntheticDefaultImports": true,
    "downlevelIteration": true,
    "jsx": "react",
    "jsxFactory": "React.createElement",
    "jsxFragmentFactory": "React.Fragment",
    "module": "commonjs",
    "moduleResolution": "Node",
    "noLib": true,
    "resolveJsonModule": true,
    "experimentalDecorators": true,
    "strict": true,
    "target": "ESNext",
    "typeRoots": ["node_modules/@rbxts"],
    "rootDir": "src",
    "outDir": "out",
    "declaration": true
  }
}
```

**Step 3: 创建空的 index.ts**

```typescript
// Luban decorators and types for roblox-ts
```

**Step 4: 验证**

```bash
cd luban-ts && npm install && npm run build
```

**Step 5: Commit**

```bash
git add luban-ts/
git commit -m "feat(luban-ts): initialize roblox-ts project"
```

---

### Task 1.2: 实现装饰器定义

**Files:**
- Modify: `luban-ts/src/index.ts`

**Step 1: 添加装饰器类型和实现**

```typescript
// === 类装饰器 ===

export interface LubanTableConfig {
  mode: "map" | "list" | "one" | "singleton";
  index: string;
  group?: string;
  tags?: string;
}

export function LubanTable(config: LubanTableConfig): ClassDecorator {
  return () => {};
}

// === 字段装饰器 ===

export function Ref<T>(target: new (...args: never[]) => T): PropertyDecorator {
  return () => {};
}

export function Range(min: number, max: number): PropertyDecorator {
  return () => {};
}

export function Required(): PropertyDecorator {
  return () => {};
}

export function Size(size: number): PropertyDecorator;
export function Size(min: number, max: number): PropertyDecorator;
export function Size(_minOrSize: number, _max?: number): PropertyDecorator {
  return () => {};
}

export function Set(..._values: (number | string)[]): PropertyDecorator {
  return () => {};
}

export function Index(field: string): PropertyDecorator {
  return () => {};
}

export function Nominal(): PropertyDecorator {
  return () => {};
}

// === 泛型类型 ===

export type ObjectFactory<T> = () => T;
```

**Step 2: 验证编译**

```bash
cd luban-ts && npm run build
```

**Step 3: Commit**

```bash
git add luban-ts/src/index.ts
git commit -m "feat(luban-ts): add decorator definitions"
```

---

## Phase 2: Rust - 装饰器数据结构

### Task 2.1: 扩展 FieldInfo 结构

**Files:**
- Modify: `src/parser/field_info.rs`

**Step 1: 添加验证器字段**

```rust
#[derive(Debug, Clone, Default)]
pub struct FieldValidators {
    pub ref_target: Option<String>,
    pub range: Option<(f64, f64)>,
    pub required: bool,
    pub size: Option<SizeConstraint>,
    pub set_values: Vec<String>,
    pub index_field: Option<String>,
    pub nominal: bool,
}

#[derive(Debug, Clone)]
pub enum SizeConstraint {
    Exact(usize),
    Range(usize, usize),
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub is_optional: bool,
    pub validators: FieldValidators,
}
```

**Step 2: 运行测试确认编译**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/parser/field_info.rs
git commit -m "feat(parser): add FieldValidators to FieldInfo"
```

---

### Task 2.2: 扩展 ClassInfo 结构

**Files:**
- Modify: `src/parser/class_info.rs`

**Step 1: 添加 LubanTable 配置**

```rust
#[derive(Debug, Clone, Default)]
pub struct LubanTableConfig {
    pub mode: String,
    pub index: String,
    pub group: Option<String>,
    pub tags: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    // ... existing fields ...
    pub luban_table: Option<LubanTableConfig>,
}
```

**Step 2: 运行测试确认编译**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/parser/class_info.rs
git commit -m "feat(parser): add LubanTableConfig to ClassInfo"
```

---

### Task 2.3: 添加装饰器解析模块

**Files:**
- Create: `src/parser/decorator.rs`
- Modify: `src/parser.rs`

**Step 1: 创建 decorator.rs**

```rust
use swc_ecma_ast::*;

#[derive(Debug, Clone)]
pub enum DecoratorArg {
    Number(f64),
    String(String),
    Identifier(String),
    Array(Vec<DecoratorArg>),
}

#[derive(Debug, Clone)]
pub struct ParsedDecorator {
    pub name: String,
    pub args: Vec<DecoratorArg>,
    pub named_args: std::collections::HashMap<String, DecoratorArg>,
}

pub fn parse_decorator(decorator: &Decorator) -> Option<ParsedDecorator> {
    match &*decorator.expr {
        Expr::Call(call) => {
            let name = match &*call.callee {
                Callee::Expr(expr) => match &**expr {
                    Expr::Ident(ident) => ident.sym.to_string(),
                    _ => return None,
                },
                _ => return None,
            };

            let mut args = Vec::new();
            let mut named_args = std::collections::HashMap::new();

            for arg in &call.args {
                match &*arg.expr {
                    Expr::Lit(Lit::Num(n)) => {
                        args.push(DecoratorArg::Number(n.value));
                    }
                    Expr::Lit(Lit::Str(s)) => {
                        args.push(DecoratorArg::String(s.value.to_string()));
                    }
                    Expr::Ident(ident) => {
                        args.push(DecoratorArg::Identifier(ident.sym.to_string()));
                    }
                    Expr::Object(obj) => {
                        for prop in &obj.props {
                            if let PropOrSpread::Prop(prop) = prop {
                                if let Prop::KeyValue(kv) = &**prop {
                                    let key = match &kv.key {
                                        PropName::Ident(i) => i.sym.to_string(),
                                        PropName::Str(s) => s.value.to_string(),
                                        _ => continue,
                                    };
                                    let value = parse_expr_to_arg(&kv.value);
                                    if let Some(v) = value {
                                        named_args.insert(key, v);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Some(ParsedDecorator { name, args, named_args })
        }
        Expr::Ident(ident) => {
            Some(ParsedDecorator {
                name: ident.sym.to_string(),
                args: Vec::new(),
                named_args: std::collections::HashMap::new(),
            })
        }
        _ => None,
    }
}

fn parse_expr_to_arg(expr: &Expr) -> Option<DecoratorArg> {
    match expr {
        Expr::Lit(Lit::Num(n)) => Some(DecoratorArg::Number(n.value)),
        Expr::Lit(Lit::Str(s)) => Some(DecoratorArg::String(s.value.to_string())),
        Expr::Ident(ident) => Some(DecoratorArg::Identifier(ident.sym.to_string())),
        _ => None,
    }
}
```

**Step 2: 在 parser.rs 中导出模块**

在 `src/parser.rs` 顶部添加：
```rust
pub mod decorator;
pub use decorator::{ParsedDecorator, DecoratorArg, parse_decorator};
```

**Step 3: 运行测试**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add src/parser/decorator.rs src/parser.rs
git commit -m "feat(parser): add decorator parsing module"
```

---

## Phase 3: Rust - 解析装饰器

### Task 3.1: 解析类装饰器 @LubanTable

**Files:**
- Modify: `src/parser.rs`

**Step 1: 在 extract_class 中解析 @LubanTable**

在 `extract_class` 函数中，解析类的装饰器：

```rust
// Parse class decorators
let mut luban_table = None;
for dec in &class_decl.class.decorators {
    if let Some(parsed) = decorator::parse_decorator(dec) {
        if parsed.name == "LubanTable" {
            luban_table = Some(LubanTableConfig {
                mode: parsed.named_args.get("mode")
                    .and_then(|v| match v {
                        DecoratorArg::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_else(|| "map".to_string()),
                index: parsed.named_args.get("index")
                    .and_then(|v| match v {
                        DecoratorArg::String(s) => Some(s.clone()),
                        _ => None,
                    })
                    .unwrap_or_default(),
                group: parsed.named_args.get("group")
                    .and_then(|v| match v {
                        DecoratorArg::String(s) => Some(s.clone()),
                        _ => None,
                    }),
                tags: parsed.named_args.get("tags")
                    .and_then(|v| match v {
                        DecoratorArg::String(s) => Some(s.clone()),
                        _ => None,
                    }),
            });
        }
    }
}
```

**Step 2: 添加到 ClassInfo 返回值**

**Step 3: 运行测试**

```bash
cargo test
```

**Step 4: Commit**

```bash
git add src/parser.rs
git commit -m "feat(parser): parse @LubanTable decorator"
```

---

### Task 3.2: 解析字段装饰器

**Files:**
- Modify: `src/parser.rs`

**Step 1: 创建 parse_field_decorators 函数**

```rust
fn parse_field_decorators(decorators: &[Decorator]) -> FieldValidators {
    let mut validators = FieldValidators::default();

    for dec in decorators {
        if let Some(parsed) = decorator::parse_decorator(dec) {
            match parsed.name.as_str() {
                "Ref" => {
                    if let Some(DecoratorArg::Identifier(class_name)) = parsed.args.first() {
                        validators.ref_target = Some(class_name.clone());
                    }
                }
                "Range" => {
                    if parsed.args.len() >= 2 {
                        if let (Some(DecoratorArg::Number(min)), Some(DecoratorArg::Number(max))) =
                            (parsed.args.get(0), parsed.args.get(1)) {
                            validators.range = Some((*min, *max));
                        }
                    }
                }
                "Required" => {
                    validators.required = true;
                }
                "Size" => {
                    match parsed.args.len() {
                        1 => {
                            if let Some(DecoratorArg::Number(n)) = parsed.args.first() {
                                validators.size = Some(SizeConstraint::Exact(*n as usize));
                            }
                        }
                        2 => {
                            if let (Some(DecoratorArg::Number(min)), Some(DecoratorArg::Number(max))) =
                                (parsed.args.get(0), parsed.args.get(1)) {
                                validators.size = Some(SizeConstraint::Range(*min as usize, *max as usize));
                            }
                        }
                        _ => {}
                    }
                }
                "Set" => {
                    for arg in &parsed.args {
                        match arg {
                            DecoratorArg::Number(n) => validators.set_values.push(n.to_string()),
                            DecoratorArg::String(s) => validators.set_values.push(s.clone()),
                            _ => {}
                        }
                    }
                }
                "Index" => {
                    if let Some(DecoratorArg::String(field)) = parsed.args.first() {
                        validators.index_field = Some(field.clone());
                    }
                }
                "Nominal" => {
                    validators.nominal = true;
                }
                _ => {}
            }
        }
    }

    validators
}
```

**Step 2: 运行测试**

```bash
cargo test
```

**Step 3: Commit**

```bash
git add src/parser.rs
git commit -m "feat(parser): parse field decorator validators"
```

---

## Phase 4: Rust - TableRegistry

### Task 4.1: 创建 TableRegistry 模块

**Files:**
- Create: `src/table_registry.rs`
- Modify: `src/lib.rs`

**Step 1: 创建 table_registry.rs**

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TableEntry {
    pub namespace: String,
    pub full_name: String,
}

#[derive(Debug, Default)]
pub struct TableRegistry {
    entries: HashMap<String, TableEntry>,
}

impl TableRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, class_name: &str, namespace: &str) {
        let full_name = if namespace.is_empty() {
            class_name.to_string()
        } else {
            format!("{}.{}", namespace, class_name)
        };

        self.entries.insert(class_name.to_string(), TableEntry {
            namespace: namespace.to_string(),
            full_name,
        });
    }

    pub fn get(&self, class_name: &str) -> Option<&TableEntry> {
        self.entries.get(class_name)
    }

    pub fn resolve_ref(&self, class_name: &str) -> Option<String> {
        self.get(class_name).map(|e| e.full_name.clone())
    }
}
```

**Step 2: 在 lib.rs 中导出**

```rust
pub mod table_registry;
pub use table_registry::TableRegistry;
```

**Step 3: 运行测试**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add src/table_registry.rs src/lib.rs
git commit -m "feat: add TableRegistry for class reference resolution"
```

---

## Phase 5: Rust - 配置扩展

### Task 5.1: 添加 table_mappings 配置

**Files:**
- Modify: `src/config.rs`

**Step 1: 添加 TableMapping 结构**

```rust
#[derive(Debug, Deserialize, Clone)]
pub struct TableMapping {
    pub pattern: String,
    pub input: String,
    pub output: Option<String>,
}
```

**Step 2: 在 Config 中添加字段**

```rust
pub struct Config {
    // ... existing fields ...
    #[serde(default)]
    pub table_mappings: Vec<TableMapping>,
}
```

**Step 3: 添加测试**

```rust
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
```

**Step 4: 运行测试**

```bash
cargo test test_parse_table_mappings
```

**Step 5: Commit**

```bash
git add src/config.rs
git commit -m "feat(config): add table_mappings configuration"
```

---

## Phase 6: Rust - 验证器生成

### Task 6.1: 创建验证器语法生成模块

**Files:**
- Create: `src/validator.rs`
- Modify: `src/lib.rs`

**Step 1: 创建 validator.rs**

```rust
use crate::parser::field_info::{FieldValidators, SizeConstraint};
use crate::table_registry::TableRegistry;

pub struct ValidatorGenerator<'a> {
    registry: &'a TableRegistry,
}

impl<'a> ValidatorGenerator<'a> {
    pub fn new(registry: &'a TableRegistry) -> Self {
        Self { registry }
    }

    /// Generate Luban type string with validators
    /// e.g., "double#range=[1,100]" or "int!#ref=item.TbItem"
    pub fn generate_type(&self, base_type: &str, validators: &FieldValidators) -> String {
        let mut result = base_type.to_string();

        // Handle required (!)
        if validators.required {
            result.push('!');
        }

        // Handle ref
        if let Some(ref_target) = &validators.ref_target {
            if let Some(full_name) = self.registry.resolve_ref(ref_target) {
                result.push_str(&format!("#ref={}", full_name));
            } else {
                eprintln!("Warning: Could not resolve ref target: {}", ref_target);
            }
        }

        // Handle range
        if let Some((min, max)) = &validators.range {
            result.push_str(&format!("#range=[{},{}]", min, max));
        }

        // Handle set
        if !validators.set_values.is_empty() {
            let set_str = validators.set_values.join(";");
            result.push_str(&format!("#set={}", set_str));
        }

        result
    }

    /// Generate container type with size/index validators
    /// e.g., "(list#size=4),Foo" or "(list#index=id),Foo"
    pub fn generate_container_type(&self, container: &str, element_type: &str, validators: &FieldValidators) -> String {
        let mut container_mods = Vec::new();

        if let Some(size) = &validators.size {
            match size {
                SizeConstraint::Exact(n) => container_mods.push(format!("size={}", n)),
                SizeConstraint::Range(min, max) => container_mods.push(format!("size=[{},{}]", min, max)),
            }
        }

        if let Some(index) = &validators.index_field {
            container_mods.push(format!("index={}", index));
        }

        if container_mods.is_empty() {
            format!("{},{}", container, element_type)
        } else {
            format!("({}#{}),{}", container, container_mods.join(","), element_type)
        }
    }
}
```

**Step 2: 在 lib.rs 中导出**

```rust
pub mod validator;
```

**Step 3: 运行测试**

```bash
cargo build
```

**Step 4: Commit**

```bash
git add src/validator.rs src/lib.rs
git commit -m "feat: add ValidatorGenerator for Luban syntax"
```

---

## Phase 7: Rust - 生成 table 元素

### Task 7.1: 扩展 generator.rs 生成 table

**Files:**
- Modify: `src/generator.rs`

**Step 1: 添加 generate_table 函数**

```rust
pub fn generate_table(
    class: &ClassInfo,
    input: &str,
    output: &str,
) -> String {
    let config = class.luban_table.as_ref().expect("Class must have @LubanTable");

    let mut attrs = vec![
        format!(r#"name="{}""#, class.name),
        format!(r#"value="{}""#, class.name),
        format!(r#"mode="{}""#, config.mode),
        format!(r#"index="{}""#, config.index),
        format!(r#"input="{}""#, input),
        format!(r#"output="{}""#, output),
    ];

    if let Some(group) = &config.group {
        attrs.push(format!(r#"group="{}""#, group));
    }

    if let Some(tags) = &config.tags {
        attrs.push(format!(r#"tags="{}""#, tags));
    }

    format!(r#"    <table {}/>"#, attrs.join(" "))
}
```

**Step 2: 运行测试**

```bash
cargo build
```

**Step 3: Commit**

```bash
git add src/generator.rs
git commit -m "feat(generator): add table element generation"
```

---

## Phase 8: 集成两遍扫描

### Task 8.1: 修改 main.rs 实现两遍扫描

**Files:**
- Modify: `src/main.rs`

**Step 1: 第一遍扫描收集 @LubanTable**

```rust
// First pass: collect @LubanTable classes into registry
let mut table_registry = TableRegistry::new();
for class in &all_classes {
    if class.luban_table.is_some() {
        let namespace = class.module_name.as_deref().unwrap_or("");
        table_registry.register(&class.name, namespace);
    }
}
```

**Step 2: 第二遍扫描生成 XML**

使用 `ValidatorGenerator` 解析 `@Ref` 等装饰器，查找 TableRegistry 生成完整引用。

**Step 3: 运行测试**

```bash
cargo test
```

**Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat: implement two-pass scanning for decorator resolution"
```

---

## 执行选择

计划已保存到 `docs/plans/2026-01-10-v2-implementation.md`。

**两种执行方式：**

1. **Subagent-Driven（当前会话）** - 每个任务派遣 subagent，任务间 code review
2. **Parallel Session（新会话）** - 打开新会话使用 executing-plans，批量执行

**选择哪种？**
