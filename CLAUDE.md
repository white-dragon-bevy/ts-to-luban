# Luban Schema Generator

高性能 Rust 工具，将 TypeScript 类/接口转换为 Luban XML Schema 定义。

## 项目概述

- **语言**: Rust
- **解析引擎**: SWC (高性能 TypeScript 解析器)
- **并行处理**: Rayon
- **配置格式**: TOML
- **npm 包**: `@white-dragon-bevy/ts-to-luban` (GitHub Packages)
- **仓库**: https://github.com/white-dragon-bevy/ts-to-luban

## 快速命令

```bash
# 构建
cargo build --release

# 运行示例
cargo run -- -c example/luban.config.toml

# 强制重新生成（忽略缓存）
cargo run -- -c example/luban.config.toml -f

# 运行测试
cargo test
```

## 项目结构

```
src/
├── main.rs          # CLI 入口
├── lib.rs           # 库导出
├── config.rs        # TOML 配置解析
├── parser.rs        # TypeScript AST 解析 (SWC)
├── parser/
│   ├── class_info.rs   # ClassInfo 结构
│   ├── enum_info.rs    # EnumInfo 结构
│   └── field_info.rs   # FieldInfo 结构
├── type_mapper.rs   # TS → Luban 类型映射
├── base_class.rs    # 父类解析 (仅配置决定，忽略 extends)
├── generator.rs     # XML 生成 (bean, enum, bean types)
├── cache.rs         # 增量缓存系统
├── scanner.rs       # 文件扫描
└── tsconfig.rs      # tsconfig.json 路径解析
```

## 核心功能

### 1. 类型转换
- `number` → `double`
- `string` → `string`
- `boolean` → `bool`
- `int` / `float` / `long` → 保持原样
- `T[]` / `Array<T>` → `list,T`
- `Map<K,V>` / `Record<K,V>` → `map,K,V`

### 2. 父类解析优先级
1. `[[parent_mappings]]` 正则匹配
2. `defaults.base_class` (默认)

> **注意**: TypeScript 的 `extends` 关键字被忽略，parent 完全由配置决定。

### 3. JSDoc 注释
- 类注释 → `<bean comment="...">`
- `@param` 标签 → `<var comment="...">`
- `@alias` 标签 → `<bean alias="...">` 或 `<enum alias="...">`
  - 支持两种格式：`@alias:别名` 或 `@alias="别名"`
- `@ignore` 标签 → 不导出该类/接口/枚举

**@ignore 示例**：
```typescript
/**
 * 内部使用的辅助类，不导出到 Luban
 * @ignore
 */
export class InternalHelper {
    public helperData: string;
}

/**
 * 内部接口，不导出
 * @ignore
 */
export interface InternalInterface {
    internalField: number;
}

/**
 * 调试用枚举，不导出
 * @ignore
 */
export enum DebugLevel {
    Off = 0,
    Error = 1,
}
```

### 4. 配置选项
```toml
[output]
path = "output.xml"
module_name = ""  # 默认为空字符串

[[sources]]
type = "file"             # 单个文件
path = "src/types.ts"
module_name = "types"     # 可选：覆盖默认 module_name

[[sources]]
type = "files"            # 多个文件
paths = ["src/a.ts", "src/b.ts"]
output_path = "output/ab.xml"
module_name = ""          # 允许空字符串

[[sources]]
type = "directory"        # 目录扫描
path = "src/triggers"
scan_options = { include_dts = true }

[[parent_mappings]]
pattern = ".*Trigger$"    # 正则匹配类名
parent = "TsTriggerClass"
```

### 5. Source 类型
- `file`: 单个文件
- `files`: 多个文件（共享 output_path 和 module_name）
- `directory`: 目录扫描
- `glob`: Glob 模式匹配（支持 `*`, `**`, `?`, `[abc]`）
- `registration`: 注册文件（未完全实现）

### 6. Glob 模式配置
```toml
[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"    # 匹配所有 Trigger 文件
output_path = "output/triggers.xml"
module_name = "triggers"
```

### 7. Per-Source 配置
每个 source 可独立配置：
- `output_path`: 覆盖默认输出路径
- `module_name`: 覆盖默认 module name（允许空字符串）

### 8. Enum 导出
TypeScript 枚举会被转换为 Luban XML `<enum>` 元素：

**字符串枚举**（使用 `tags="string"`）：
```typescript
export enum ItemType {
    Role = "role",        // → value="1"
    Consumable = "consumable"  // → value="2"
}
```
生成：
```xml
<enum name="ItemType" tags="string">
    <var name="Role" alias="role" value="1"/>
    <var name="Consumable" alias="consumable" value="2"/>
</enum>
```

**数值枚举**（无 tags 属性）：
```typescript
export enum SkillStyle {
    Attack = 1,   // → value="1"
    Defense = 2   // → value="2"
}
```
生成：
```xml
<enum name="SkillStyle">
    <var name="Attack" alias="attack" value="1"/>
    <var name="Defense" alias="defense" value="2"/>
</enum>
```

**位标志枚举**（使用 `@flags="true"` 和 `@alias`）：
```typescript
/**
 * 单位权限标志
 * @flags="true"
 * @alias:权限
 */
export enum UnitFlag {
    /**
     * 可以移动
     * @alias="移动"
     */
    CAN_MOVE = 1 << 0,
    /**
     * 可以攻击
     * @alias="攻击"
     */
    CAN_ATTACK = 1 << 1,
    /** 组合标志 */
    BASICS = CAN_MOVE | CAN_ATTACK,
}
```
生成：
```xml
<enum name="UnitFlag" alias="权限" flags="true" comment="单位权限标志">
    <var name="CAN_MOVE" alias="移动" value="1" comment="可以移动"/>
    <var name="CAN_ATTACK" alias="攻击" value="2" comment="可以攻击"/>
    <var name="BASICS" alias="basics" value="3" comment="组合标志"/>
</enum>
```

**规则**：
- 枚举 `alias` = `@alias:xxx` 或 `@alias="xxx"` 标签值（可选）
- 成员 `alias` = `@alias="xxx"` 标签值，或小写的 name
- `@flags="true"` 标签 → 生成 `flags="true"` 属性
- 支持位运算表达式：`1 << N`、`A | B`、`A & B` 等
- 支持枚举成员引用：`BASICS = CAN_MOVE | CAN_ATTACK`
- 字符串枚举的 value 从 1 自动递增（原始字符串值被忽略）
- 数值枚举使用原始数值
- 枚举输出到独立文件，默认为 `{output}_enums.xml`

**配置**：
```toml
[output]
path = "output/generated.xml"
enum_path = "output/enums.xml"  # 可选：自定义枚举输出路径
```

### 9. Bean 类型枚举导出（按 parent 分组）
将所有 bean 按 parent 分组导出为枚举，用于类型安全的 bean 引用：

```toml
[output]
path = "output/generated.xml"
module_name = "triggers"
bean_types_path = "output/bean_types.xml"  # 使用全局 module_name
```

**规则**：
- 每个 parent 生成一个独立的枚举，名称为 `{parent}Enum`
- `value` = bean 名称（字符串）
- `alias` 仅当 bean 有 `@alias` 标签时生成
- `comment` = bean 的注释
- 没有 parent 的 bean 不会生成枚举

**示例**：
```typescript
/**
 * 伤害触发器
 * @alias:伤害
 */
export class DamageTrigger { ... }

/** 治疗触发器 */
export class HealTrigger { ... }
```

生成（假设 parent 都是 `TriggerBase`）：
```xml
<module name="triggers" comment="自动生成的 bean 类型枚举">
    <enum name="TriggerBaseEnum" comment="TriggerBase 的子类型">
        <var name="DamageTrigger" alias="伤害" value="DamageTrigger" comment="伤害触发器"/>
        <var name="HealTrigger" value="HealTrigger" comment="治疗触发器"/>
    </enum>
</module>
```

## 开发注意事项

- SWC 的 `TsUnionType` 在新版本中变为 `TsUnionOrIntersectionType::TsUnionType`
- `TsParser` 包含 `Lrc<SourceMap>` 不是 `Sync`，需要在并行闭包中创建新实例
- 注释附加在 `export` 关键字位置，需要使用 `export.span.lo` 获取
- `ClassInfo.extends` 字段保留但不再用于 parent 解析

## 发布流程

```bash
# 1. 更新版本号 (Cargo.toml 和 package.json)
# 2. 提交并打 tag
git add . && git commit -m "release: vX.Y.Z"
git tag vX.Y.Z && git push && git push --tags

# 自动触发:
# - Release workflow: 构建 linux/windows/macos 二进制 → GitHub Releases
# - Publish workflow: 发布到 GitHub Packages (@white-dragon-bevy/ts-to-luban)
```

## CI/CD 文件

- `.github/workflows/release.yml` - 多平台构建和 GitHub Release
- `.github/workflows/publish-npm.yml` - 发布到 GitHub Packages
- `.github/workflows/ci.yml` - PR/push 测试
