# 设计：使用 TypeScript extends 决定 bean parent

## 概述

简化 parent 解析逻辑：删除配置驱动的 `parent_mappings` 和 `defaults.base_class`，改为直接使用 TypeScript 的 `extends` 关键字决定 bean 的 parent 属性。

## 当前行为

```
TypeScript extends → 被忽略
配置 parent_mappings 正则 → 优先匹配
配置 defaults.base_class → 兜底
```

## 新行为

```
TypeScript extends → 直接作为 parent
没有 extends → 没有 parent 属性
```

class 和 interface 统一行为。

## 示例

```typescript
class Parent {
    parentField: number;
}

class Child extends Parent {
    childField: string;
}

class Standalone {
    data: string;
}
```

生成：

```xml
<bean name="Parent">
    <var name="parentField" type="int"/>
</bean>

<bean name="Child" parent="Parent">
    <var name="childField" type="string"/>
</bean>

<bean name="Standalone">
    <var name="data" type="string"/>
</bean>
```

## 删除的功能

1. `[[parent_mappings]]` 配置节
2. `[defaults].base_class` 配置项
3. `src/base_class.rs` 模块
4. `example/` 目录

## 文件改动清单

### 删除文件

| 文件 | 说明 |
|------|------|
| `src/base_class.rs` | 整个模块删除 |
| `example/` | 整个目录删除 |

### 修改文件

| 文件 | 改动 |
|------|------|
| `src/config.rs` | 删除 `parent_mappings: Vec<ParentMapping>` 字段、`ParentMapping` 结构体、`defaults.base_class` 字段、相关合并逻辑 |
| `src/lib.rs` | 删除 `pub mod base_class;` |
| `src/main.rs` | 删除 `mod base_class`、`use base_class::BaseClassResolver`、`BaseClassResolver` 实例化 |
| `src/generator.rs` | 删除 `base_resolver` 字段，`generate_bean` 中直接用 `class.extends.clone().unwrap_or_default()` |
| `tests/integration.rs` | 更新测试配置，移除 `base_class` 和 `parent_mappings` |
| `tests/fixtures/` | 更新或删除相关 fixture 配置 |
| `CLAUDE.md` | 更新文档，移除 parent_mappings 相关说明 |
| `README.md` | 更新文档 |

### 新增/修改测试

| 文件 | 说明 |
|------|------|
| `luban-ts/src/__examples__/inheritance.ts` | 新增：继承关系测试用例 |
| `luban-ts/luban.config.toml` | 添加 inheritance.ts 作为 source |

## generator.rs 核心改动

```rust
// 之前
fn generate_bean(&self, lines: &mut Vec<String>, class: &ClassInfo) {
    let parent = self.base_resolver.resolve(class);
    // ...
}

// 之后
fn generate_bean(&self, lines: &mut Vec<String>, class: &ClassInfo) {
    let parent = class.extends.clone().unwrap_or_default();
    // ...
}
```

## XmlGenerator 结构体简化

```rust
// 之前
pub struct XmlGenerator<'a> {
    base_resolver: &'a BaseClassResolver<'a>,
    type_mapper: &'a TypeMapper,
    table_registry: &'a TableRegistry,
    table_mapping_resolver: &'a TableMappingResolver,
    current_module: String,
}

// 之后
pub struct XmlGenerator<'a> {
    type_mapper: &'a TypeMapper,
    table_registry: &'a TableRegistry,
    table_mapping_resolver: &'a TableMappingResolver,
    current_module: String,
}
```

## 配置文件格式变化

```toml
# 之前
[defaults]
base_class = "TsClass"

[[parent_mappings]]
pattern = ".*Trigger$"
parent = "TsTriggerClass"

# 之后
# 无需配置，完全由 TypeScript extends 决定
```

## 验证步骤

1. 删除相关代码和文件
2. 运行 `cargo build` 确保编译通过
3. 运行 `cargo test` 确保测试通过
4. 在 `luban-ts` 目录运行生成和 Luban 验证
5. 确认生成的 XML parent 属性正确
