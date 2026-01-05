---
name: ts-to-luban
description: 用于将 TypeScript 类/接口/枚举转换为 Luban XML Schema 定义、配置源文件模式、设置父类映射或排查 schema 生成问题 - 提供完整的 CLI 用法、配置参考和类型映射规则
---

# ts-to-luban

## 概述

高性能 Rust 工具，将 TypeScript 类、接口和枚举转换为 Luban XML Schema 定义。使用 SWC 解析 TypeScript，使用 Rayon 进行并行处理。

**核心原则：** 配置决定一切 - TypeScript 的 `extends` 被忽略，父类完全由配置模式决定。

## 使用场景

- 将 TypeScript 类型定义转换为 Luban XML schema
- 为项目配置 `luban.config.toml`
- 使用正则表达式配置父类映射
- 添加 JSDoc 注解（alias、flags、注释）
- 排查 schema 生成问题
- 了解类型映射规则（TS → Luban）

## 安装

```bash
npm install @white-dragon-bevy/ts-to-luban
```

## 快速命令

```bash
# 使用配置运行
npx luban-gen -c luban.config.toml

# 强制重新生成（忽略缓存）
npx luban-gen -c luban.config.toml -f
```

## 配置参考

### 基本结构

```toml
[output]
path = "output/generated.xml"      # 主 bean 输出
enum_path = "output/enums.xml"     # 可选：枚举输出（默认：{output}_enums.xml）
bean_types_path = "output/types.xml"  # 可选：bean 类型枚举（使用全局 module_name）
module_name = ""                   # 模块名（空字符串 = 不包装 <module>）

[defaults]
base_class = "BaseClass"           # 所有 bean 的默认父类

[[parent_mappings]]
pattern = ".*Trigger$"             # 匹配类名的正则表达式
parent = "TsTriggerClass"          # 匹配时分配的父类

[[sources]]
type = "file"                      # 源类型（见下文）
path = "src/types.ts"
module_name = "types"              # 可选：覆盖默认 module_name
output_path = "output/types.xml"   # 可选：覆盖默认输出路径
```

### 源类型

| 类型 | 用途 | 关键字段 |
|------|------|----------|
| `file` | 单个文件 | `path` |
| `files` | 多个文件 | `paths`（数组） |
| `directory` | 目录扫描 | `path`, `scan_options` |
| `glob` | 模式匹配 | `pattern`（如 `src/**/*Trigger.ts`） |
| `registration` | 注册文件 | （未完全实现） |

**示例：**

```toml
# 单个文件
[[sources]]
type = "file"
path = "src/triggers/damage.ts"

# 多个文件共享配置
[[sources]]
type = "files"
paths = ["src/a.ts", "src/b.ts"]
output_path = "output/ab.xml"

# 带选项的目录
[[sources]]
type = "directory"
path = "src/triggers"
scan_options = { include_dts = true }

# Glob 模式
[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"
output_path = "output/triggers.xml"
```

## 类型映射

| TypeScript | Luban |
|------------|-------|
| `number` | `double` |
| `string` | `string` |
| `boolean` | `bool` |
| `int` / `float` / `long` | 保持不变 |
| `T[]` / `Array<T>` | `list,T` |
| `Map<K,V>` / `Record<K,V>` | `map,K,V` |

## JSDoc 注解

### 类/接口级别

```typescript
/**
 * 伤害触发器描述
 * @alias:伤害触发器
 */
export class DamageTrigger { ... }
```

生成：`<bean name="DamageTrigger" alias="伤害触发器" comment="伤害触发器描述">`

### 字段级别

```typescript
export class Example {
    /** @param 字段描述 */
    damage: number;
}
```

生成：`<var name="damage" type="double" comment="字段描述"/>`

### @ignore 标签

使用 `@ignore` 标签排除不需要导出的类、接口或枚举：

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

带有 `@ignore` 标签的类型不会出现在生成的 XML 中。

### 枚举注解

```typescript
/**
 * 单位标志
 * @flags="true"
 * @alias:权限
 */
export enum UnitFlag {
    /** @alias="移动" */
    CAN_MOVE = 1 << 0,
    CAN_ATTACK = 1 << 1,
    BASICS = CAN_MOVE | CAN_ATTACK,  // 支持位运算表达式
}
```

生成：
```xml
<enum name="UnitFlag" alias="权限" flags="true" comment="单位标志">
    <var name="CAN_MOVE" alias="移动" value="1"/>
    <var name="CAN_ATTACK" alias="can_attack" value="2"/>
    <var name="BASICS" alias="basics" value="3"/>
</enum>
```

## 枚举规则

| 枚举类型 | `tags` 属性 | `value` |
|----------|-------------|---------|
| 字符串枚举 | `tags="string"` | 从 1 自动递增 |
| 数值枚举 | （无） | 原始数值 |
| 标志枚举 | + `flags="true"` | 计算后的位运算结果 |

**位运算表达式：** 支持 `1 << N`、`A | B`、`A & B` 和成员引用。例如 `BASICS = CAN_MOVE | CAN_ATTACK` 计算为 `3`（1 | 2）。

**成员 alias：** 使用 `@alias="..."` 的值，否则使用小写的 name。

## 父类解析优先级

1. `[[parent_mappings]]` 正则匹配（第一个匹配生效）
2. `[defaults].base_class`（兜底）

**重要：** TypeScript 的 `extends` 关键字被完全忽略。父类 100% 由配置决定。

## Bean 类型枚举

配置 `bean_types_path` 后，会按父类分组生成枚举：

```xml
<enum name="TriggerBaseEnum" comment="TriggerBase 的子类型">
    <var name="DamageTrigger" alias="伤害" value="DamageTrigger" comment="..."/>
    <var name="HealTrigger" value="HealTrigger" comment="..."/>
</enum>
```

## 常见问题

| 问题 | 解决方法 |
|------|----------|
| 期望 `extends` 设置父类 | 使用 `[[parent_mappings]]` 或 `defaults.base_class` |
| 枚举输出缺失 | 检查 `enum_path` 配置或查找 `{output}_enums.xml` |
| 枚举 alias 未生效 | 在 JSDoc 中使用 `@alias:xxx` 或 `@alias="xxx"` |
| Glob 模式不匹配 | 确认使用正斜杠，检查 `**` 匹配子目录 |
| 缓存导致未更新 | 使用 `-f` 标志强制重新生成 |
| 需要排除某些类型 | 在 JSDoc 中添加 `@ignore` 标签 |
