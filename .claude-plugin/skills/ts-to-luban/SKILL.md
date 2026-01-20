---
name: configuring-ts-luban
description: 配置 ts-luban 将 TypeScript 类转换为 Luban XML Schema - TOML 配置、sources、table_mappings、ref_configs、TypeScript 代码生成
---

# 配置 ts-luban

## 使用场景

- 创建/修改 `ts-luban.config.toml`
- 调试缺失的 beans/tables/enums
- 设置跨包引用
- 配置 TypeScript table 代码生成

## 快速问题排查

| 问题 | 解决方案 |
|------|----------|
| 枚举混在 beans 中 | 添加 `enum_path = "output/enums.xml"` |
| @Ref 跨包失败 | 添加 `[[ref_configs]]` 指向外部配置 |
| 没有生成 `<table>` | 为 @LubanTable 类添加 `[[table_mappings]]` |
| .d.ts 被排除 | 添加 `scan_options = { include_dts = true }` |
| `{name}` 占位符错误 | `{name}` = **小写**类名（`TbItem` → `tbitem`）|

## 配置结构

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "output/beans.xml"           # 默认输出路径
module_name = ""                     # 默认模块名
enum_path = "output/enums.xml"       # 枚举单独输出
bean_types_path = "output/types.xml" # Bean 类型枚举
table_output_path = "out/tables"     # TS 代码生成目录

[[sources]]
type = "directory"                   # | "file" | "files" | "glob"
path = "src/types"
module_name = "types"                # 覆盖默认
output_path = "custom.xml"           # 覆盖默认
scan_options = { include_dts = true }

[[table_mappings]]
pattern = "Tb.*"                     # 正则匹配
input = "data/{name}.xlsx"           # {name} = 小写类名
output = "{name}"

[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"

[type_mappings]
Vector3 = "Vector3"
Entity = "long"
```

## 源类型

| 类型 | 字段 | 说明 |
|------|------|------|
| `directory` | `path`, `output_path?`, `module_name?`, `scan_options?` | 递归扫描 |
| `file` | `path`, `output_path?`, `module_name?` | 单个文件 |
| `files` | `paths`, `output_path?`, `module_name?` | 多个文件 |
| `glob` | `pattern`, `output_path?`, `module_name?` | 模式匹配 |

**ScanOptions**（仅 directory）：
- `include_dts: bool` - 包含 `.d.ts`（默认 false）
- `include_node_modules: bool` - 包含 `node_modules`（默认 false）

## 文件排除规则

**总是排除**：`.spec.ts`, `.test.ts`, `.spec.tsx`, `.test.tsx`

**有条件排除**：
- `.d.ts` → 除非 `include_dts = true`
- `node_modules` → 除非 `include_node_modules = true`

## Table 映射

`@LubanTable` 类需要 `[[table_mappings]]` 生成 `<table>` 元素：

```toml
[[table_mappings]]
pattern = "Tb.*"                     # 正则：以 "Tb" 开头
input = "data/{name}.xlsx"           # {name} = 小写类名
output = "{name}"
```

**`{name}` 占位符**：
- `TbItem` → `tbitem` → `data/tbitem.xlsx`
- 第一个匹配生效（按顺序处理）

## 跨包引用

```toml
[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"
```

- 源路径相对于引用配置目录解析
- 支持递归加载

## TypeScript 代码生成

配置 `table_output_path` 启用：

```toml
[output]
table_output_path = "out/tables"
```

生成文件：
- `tables.ts` - Table 类型定义
- `beans.ts` - Class 字典

**tables.ts 类型映射**：
- `mode="map"` → `Map<number, Type>`
- `mode="list"` → `Type[]`
- `mode="one"` / `mode="singleton"` → `Type`

**beans.ts 键格式**：`module.ClassName`

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

## 运行命令

```bash
# 生成
cargo run -- -c ts-luban.config.toml

# 强制重新生成（忽略缓存）
cargo run -- -c ts-luban.config.toml -f

# Watch 模式
cargo run -- -c ts-luban.config.toml -w
```
