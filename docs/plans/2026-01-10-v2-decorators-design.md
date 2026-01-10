# V2 设计：装饰器支持

## 概述

在现有 Rust CLI 基础上扩展，通过 SWC 静态解析 TypeScript 装饰器，生成 Luban XML Schema（包含验证器和 table 定义）。

## v2 范围

### 核心功能

| 功能 | 描述 |
|------|------|
| 装饰器解析 | SWC 静态解析装饰器语法 |
| `@LubanTable` | 生成 `<table>` + `<bean>` |
| `@Ref(Class)` | 引用验证，类型安全，无法 import 报错 |
| `@Range(min, max)` | 数值范围验证 |
| `@Required` | 不能为默认值 |
| `@Size(n)` / `@Size(min, max)` | 容器大小验证 |
| `@Set(...)` | 值在集合内 |
| `@Index("field")` | 列表内字段唯一 |
| `@Nominal` | 添加 `nominal="true"` |
| `ObjectFactory<T>` | 泛型识别，添加 tags |
| `[[table_mappings]]` | 正则匹配 input/output |

### 延后功能

- `@Path` 验证器
- JSON Schema 生成
- Watch 模式
- 文档生成（面向策划）

## 架构设计

### 目录结构

```
src/
├── parser/
│   ├── class_info.rs      # 扩展：存储装饰器信息
│   ├── field_info.rs      # 扩展：存储字段装饰器
│   ├── decorator.rs       # 新增：装饰器 AST 解析
│   └── table_registry.rs  # 新增：className → namespace 映射
├── config.rs              # 扩展：新增 [[table_mappings]]
├── generator.rs           # 扩展：生成 <table> 元素
└── validator.rs           # 新增：装饰器验证规则生成
```

### 数据流

```
TypeScript 源码
    ↓ SWC 解析
AST + 装饰器信息
    ↓ 第一遍扫描
收集 @LubanTable 类 → TableRegistry
    ↓ 第二遍扫描
解析 @Ref 等装饰器，查表生成完整引用
    ↓ validator
装饰器 → Luban 验证语法
    ↓ generator
XML 输出 (<bean> + <table>)
```

## 数据结构

### 装饰器参数

```typescript
type DecoratorArg =
    | { type: "number", value: number }
    | { type: "string", value: string }
    | { type: "identifier", name: string }
    | { type: "array", items: DecoratorArg[] }

interface Decorator {
    name: string
    args: DecoratorArg[]
    namedArgs: Record<string, DecoratorArg>
}
```

### LubanTable 配置

```typescript
interface LubanTableConfig {
    mode: "map" | "list" | "one" | "singleton"
    index: string
    group?: string
    tags?: string
}
```

### 字段验证器

```typescript
interface FieldValidators {
    refTarget?: string
    range?: [number, number]
    required?: boolean
    size?: number | [number, number]
    setValues?: (number | string)[]
    indexField?: string
    nominal?: boolean
}
```

### Table 注册表

```typescript
interface TableRegistry {
    [className: string]: {
        namespace: string
        fullName: string
    }
}
```

## npm 包：luban-ts/

roblox-ts 项目，包含空实现装饰器和类型定义。

发布为现有包 `@white-dragon-bevy/ts-to-luban` 的一部分。

### 装饰器定义

```typescript
// 类装饰器
export function LubanTable(config: {
    mode: "map" | "list" | "one" | "singleton"
    index: string
    group?: string
    tags?: string
}): ClassDecorator {
    return () => {}
}

// 字段装饰器
export function Ref<T>(target: new (...args: any[]) => T): PropertyDecorator {
    return () => {}
}

export function Range(min: number, max: number): PropertyDecorator {
    return () => {}
}

export function Required(): PropertyDecorator {
    return () => {}
}

export function Size(size: number): PropertyDecorator
export function Size(min: number, max: number): PropertyDecorator
export function Size(minOrSize: number, max?: number): PropertyDecorator {
    return () => {}
}

export function Set(...values: (number | string)[]): PropertyDecorator {
    return () => {}
}

export function Index(field: string): PropertyDecorator {
    return () => {}
}

export function Nominal(): PropertyDecorator {
    return () => {}
}

// 泛型类型
export type ObjectFactory<T> = () => T
```

## 配置文件扩展

新增 `[[table_mappings]]`：

```toml
[[table_mappings]]
pattern = "Tb.*"
input = "configs/{name}.xlsx"
output = "{name}"

[[table_mappings]]
pattern = "TbItem"
input = "items/item_data.xlsx"
output = "item"
```

匹配规则：
- 精确匹配优先于正则
- 多个正则按配置顺序，首次匹配生效
- 无匹配则报错

## 验证器语法生成

| 装饰器 | 输入 | 输出 |
|--------|------|------|
| `@Ref(TbItem)` | `id: number` | `type="double#ref=item.TbItem"` |
| `@Range(1, 100)` | `count: number` | `type="double#range=[1,100]"` |
| `@Required` | `name: string` | `type="string!"` |
| `@Size(4)` | `items: number[]` | `type="(list#size=4),double"` |
| `@Size(2, 5)` | `items: number[]` | `type="(list#size=[2,5]),double"` |
| `@Set(1, 2, 3)` | `level: number` | `type="double#set=1;2;3"` |
| `@Index("id")` | `list: Foo[]` | `type="(list#index=id),Foo"` |
| `@Nominal` | `flag: boolean` | `nominal="true"` |
| `ObjectFactory<T>` | `trigger: ObjectFactory<BaseTrigger>` | `type="BaseTrigger" tags="objectFactory"` |

组合示例：
```typescript
@Ref(TbItem)
@Required
itemId: number
// → type="double!#ref=item.TbItem"
```

## Table 生成示例

输入：
```typescript
@LubanTable({ mode: "map", index: "id", group: "client" })
export class TbItem {
    id: number
    name: string
    price: number
}
```

配置：
```toml
[[table_mappings]]
pattern = "Tb.*"
input = "configs/{name}.xlsx"
output = "{name}"
```

输出（同一文件）：
```xml
<bean name="TbItem">
    <var name="id" type="double"/>
    <var name="name" type="string"/>
    <var name="price" type="double"/>
</bean>

<table name="TbItem"
       value="TbItem"
       mode="map"
       index="id"
       input="configs/TbItem.xlsx"
       output="TbItem"
       group="client"/>
```
