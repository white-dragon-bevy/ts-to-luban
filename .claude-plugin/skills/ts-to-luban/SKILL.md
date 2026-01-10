---
name: configuring-ts-luban
description: 用于配置 ts-luban 将 TypeScript 类转换为 Luban XML Schema - 涵盖 TOML 配置包括 sources、table_mappings、ref_configs、scan_options 和 output paths，基于 src/config.rs 和 src/scanner.rs 的实际 Rust 实现
---

# 配置 ts-luban

## 概述

ts-luban 将使用 `@LubanTable` 装饰器的 TypeScript 类转换为 Luban XML 定义，并生成类型安全的 table 加载器。配置使用 TOML 格式，由 `src/config.rs` 的 Rust 实现解析。

## 使用场景

```
需要配置 ts-luban？ → 是：使用此技能
```

- 创建新的 `ts-luban.config.toml`
- 调试缺失的 beans/tables/enums
- 设置跨包引用
- 理解为什么某些文件被排除
- 配置 TypeScript table 代码生成

## 快速参考

| 问题 | 解决方案 |
|------|----------|
| 枚举混在 beans 中 | 添加 `enum_path = "output/enums.xml"` |
| @Ref 跨包失败 | 添加 `[[ref_configs]]` 指向外部配置 |
| 没有生成 `<table>` 元素 | 为 @LubanTable 类添加 `[[table_mappings]]` |
| 测试文件 (.spec.ts) 被排除 | **自动行为** - 无法禁用 |
| .d.ts 文件被排除 | 添加 `scan_options = { include_dts = true }` |
| node_modules 被排除 | 添加 `scan_options = { include_node_modules = true }` |
| {name} 占位符错误 | `{name}` = **小写**类名（如 `TbItem` → `tbitem`）

## 配置结构

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "output/beans.xml"           # 源的默认输出路径
module_name = ""                     # 默认模块名（空字符串）
cache_file = ".luban-cache.json"     # 缓存文件（默认：.luban-cache.json）
enum_path = "output/enums.xml"      # 单独的枚举输出（可选）
bean_types_path = "output/types.xml" # Bean 类型枚举（可选）
table_output_path = "out/tables"     # TS table 代码生成（可选）

[[sources]]
type = "directory"                   # | "file" | "files" | "glob"
path = "src/types"
module_name = "types"                # 覆盖默认 module_name
output_path = "custom.xml"           # 覆盖默认输出路径
scan_options = { include_dts = true } # 仅用于 directory 类型

[[table_mappings]]
pattern = "Tb.*"                     # 正则表达式模式
input = "data/{name}.xlsx"           # {name} = 小写类名
output = "{name}"                    # 可选的输出模板

[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"

[type_mappings]
Vector3 = "Vector3"
```

## 源类型

| 类型 | 字段 | 说明 |
|------|------|------|
| `directory` | `path`, `output_path?`, `module_name?`, `scan_options?` | 递归扫描 |
| `file` | `path`, `output_path?`, `module_name?` | 单个文件 |
| `files` | `paths`, `output_path?`, `module_name?` | 多个文件 |
| `glob` | `pattern`, `output_path?`, `module_name?` | 模式匹配 |
| `registration` | `path` | **尚未实现** |

**ScanOptions**（仅 directory 类型）：
- `include_dts: bool` - 包含 `.d.ts` 文件（默认：false）
- `include_node_modules: bool` - 包含 `node_modules`（默认：false）

## 自动文件排除

**总是排除**（无法禁用）：
- 测试文件：`.spec.ts`, `.test.ts`, `.spec.tsx`, `.test.tsx`

**有条件排除**：
- `.d.ts` 文件 → 除非 `include_dts = true` 否则排除
- `node_modules` → 除非 `include_node_modules = true` 否则排除

**默认包含**：
- 仅 `.ts` 和 `.tsx` 文件

## Table 映射

`@LubanTable` 类需要 `[[table_mappings]]` 来生成 `<table>` 元素。

```toml
[[table_mappings]]
pattern = "Tb.*"                     # 正则：以 "Tb" 开头的类
input = "data/{name}.xlsx"           # {name} = 小写类名
output = "{name}"                    # 可选
```

**关键：`{name}` 占位符行为：**
- `{name}` 被替换为**小写**类名
- `TbItem` → `{name}` → `tbitem` → `data/tbitem.xlsx`
- `MonsterConfig` → `{name}` → `monsterconfig` → `data/monsterconfig.xlsx`

**模式匹配：**
- 第一个匹配生效（按顺序处理）
- 使用正则语法（如 `Tb.*`, `.*Config$`, `^TbItem$`）

## 跨包引用

使用 `[[ref_configs]]` 引用其他包中定义的 beans（`@Ref` 需要）：

```toml
[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"
```

**路径解析：**
- 源路径相对于引用配置的目录解析
- `output_path` 不解析 - 使用运行时根目录
- 支持递归加载

## 输出路径行为

- `[output]path` = 没有 `output_path` 的源的默认值
- `[[sources]]output_path` = 每个源的覆盖
- 多个源可以有不同的 `output_path` 值

```toml
[output]
path = "default.xml"  # 当源没有 output_path 时使用

[[sources]]
type = "directory"
path = "src/configs"
output_path = "configs.xml"  # 使用这个代替

[[sources]]
type = "directory"
path = "src/entities"
# 使用 default.xml（未指定 output_path）
```

## 类型映射

将 TypeScript 类型映射到 Luban 类型：

```toml
[type_mappings]
Vector3 = "Vector3"
Color3 = "math.Color3"
Entity = "long"
```

## 默认值

| 字段 | 默认值 |
|------|--------|
| `module_name` | `""` (空字符串) |
| `cache_file` | `.luban-cache.json` |
| `sources` | `[]` (空) |
| `table_mappings` | `[]` (空) |
| `ref_configs` | `[]` (空) |
| `type_mappings` | `{}` (空) |
| `defaults` | 空结构体（无字段） |

## 完整示例

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "configs/defines/beans.xml"
enum_path = "configs/defines/enums.xml"
bean_types_path = "configs/defines/bean_types.xml"
table_output_path = "out/tables"

[[sources]]
type = "directory"
path = "src/configs"
module_name = "configs"

[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"
module_name = "triggers"

[[ref_configs]]
path = "../shared/ts-luban.config.toml"

[[table_mappings]]
pattern = "Tb.*"
input = "data/{name}.xlsx"
output = "{name}"

[type_mappings]
Vector3 = "math.Vector3"
```

## 运行

```bash
# 生成
cargo run -- -c ts-luban.config.toml

# 强制重新生成（忽略缓存）
cargo run -- -c ts-luban.config.toml -f
```

## 实现细节

**来自实际 Rust 代码的关键行为（`src/config.rs`, `src/scanner.rs`, `src/table_mapping.rs`）：**

1. **`{name}` 占位符**：替换为小写类名（`table_mapping.rs:23`）
2. **测试文件**：总是排除（`scanner.rs:50-57`）
3. **`output_path`**：不相对于配置目录解析（`config.rs:140`）
4. **Glob 模式**：使用标准 glob 语法（`*`, `**`, `?`, `[abc]`）
5. **DefaultsConfig**：无字段的空结构体（`config.rs:84-85`）
6. **Registration 类型**：存在于枚举中但未实现（`config.rs:80`）
