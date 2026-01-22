# Luban Schema Generator

高性能 Rust 工具，将 TypeScript 类/接口转换为 Luban XML Schema 定义。

## 项目概述

- **语言**: Rust + SWC (TypeScript 解析)
- **npm 包**: `@white-dragon-bevy/ts-to-luban`
- **仓库**: https://github.com/white-dragon-bevy/ts-to-luban

## 快速命令

```bash
# 构建
cargo build --release

# 运行（强制重新生成）
cargo run -- -c examples/luban.config.toml -f

# Watch 模式
cargo run -- -c examples/luban.config.toml -w

# 测试
cargo test
```

## 项目结构

```
src/
├── main.rs              # CLI 入口
├── config.rs            # TOML 配置解析
├── parser.rs            # TypeScript AST 解析
├── parser/
│   ├── class_info.rs    # ClassInfo + LubanTableConfig
│   ├── enum_info.rs     # EnumInfo 结构
│   ├── field_info.rs    # FieldInfo + FieldValidators
│   └── decorator.rs     # 装饰器解析
├── type_mapper.rs       # TS → Luban 类型映射
├── generator.rs         # XML 生成
├── validator.rs         # 验证器语法
├── ts_generator/        # TypeScript 代码生成
│   ├── beans_gen.rs     # Beans 字典
│   └── tables_simple_gen.rs # Tables 类型
├── cache.rs             # 增量缓存
└── scanner.rs           # 文件扫描

luban-ts/                # 示例项目 (roblox-ts), 目录名为 examples/
├── src/
│   ├── index.ts         # 装饰器定义
│   ├── __examples__/    # 示例配置类
│   └── __tests__/       # 测试
├── rokit.toml           # Rokit 工具配置 (rojo)
└── luban.config.toml
```

## 核心功能

### 类型转换

| TypeScript | Luban |
|------------|-------|
| `number` | `double` |
| `string` | `string` |
| `boolean` | `bool` |
| `int`/`float`/`long` | 保持原样 |
| `T[]` | `list,T` |
| `Map<K,V>` | `map,K,V` |
| `ObjectFactory<T>` | `T` + `tags="ObjectFactory=true"` |

### 装饰器

**类装饰器**：
- `@LubanTable({ mode, index, group?, tags? })` - 标记为数据表

**字段装饰器**：
| 装饰器 | 生成 |
|--------|------|
| `@Range(1, 100)` | `type="double#range=[1,100]"` |
| `@Required()` | `type="string!"` |
| `@Size(4)` / `@Size(2, 5)` | `type="(list#size=4),double"` |
| `@Set(1, 2, 3)` | `type="double#set=1,2,3"` |
| `@Index("id")` | `type="(list#index=id),Foo"` |
| `@Nominal()` | `nominal="true"` |

### JSDoc 修饰符

**字段级别**：
- `@type="int"` - 类型覆盖
- `@default="0"` - 默认值
- `@sep="|"` - 列表分隔符
- `@mapsep=",|"` - Map 分隔符
- `@ref` - 引用验证器（自动发现目标表）
- `@refKey` - Map Key 引用验证器（仅用于 Map）
- `@tags="key=value,..."` - 自定义标签

**类/接口级别**：`@table="map,id"`, `@input="path"`

**枚举级别**：`@tags="string"`, `@flags="true"`, `@alias:别名`

### 引用验证器 (@ref/@refKey)

使用 JSDoc 注释标记引用字段，字段类型必须是配置在 `[tables]` 中的 class/interface：

```typescript
/**
 * @ref
 */
item: Item;  // -> indexType#ref=module.ItemTable

/**
 * @refKey
 * @ref
 */
itemToSkill: Map<Item, Skill>;  // key 引用 Item 表，value 引用 Skill 表

/**
 * @tags="RefOverride=true"
 */
itemId: number;  // -> double tags="RefOverride=true"
```

### 父类解析

`extends` 关键字 → `parent` 属性（class 和 interface 一致）

## 数据结构

```rust
// 字段验证器
pub struct FieldValidators {
    pub has_ref: bool,           // @ref JSDoc tag
    pub has_ref_key: bool,       // @refKey JSDoc tag
    pub range: Option<(f64, f64)>,
    pub required: bool,
    pub size: Option<SizeConstraint>,
    pub set_values: Vec<String>,
    pub index_field: Option<String>,
    pub nominal: bool,
}

// 字段信息
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub validators: FieldValidators,
    pub default_value: Option<String>,
    pub type_override: Option<String>,
    pub custom_tags: Option<String>,  // @tags JSDoc tag
    // ...
}

// 类信息
pub struct ClassInfo {
    pub name: String,
    pub fields: Vec<FieldInfo>,
    pub extends: Option<String>,
    pub luban_table: Option<LubanTableConfig>,
    pub table_config: Option<JsDocTableConfig>,
    // ...
}
```

## TDD 开发流程

```bash
# 1. Rust 层
cargo test                    # RED → GREEN → REFACTOR
cargo build --release

# 2. 验证生成
cargo run -- -c examples/luban.config.toml -f

# 3. examples 层 (roblox-ts)
cd examples
rokit install                 # 安装 rojo 工具
npm install                   # 安装依赖
npm run config:build          # 生成代码
npm run build                 # 编译
npm test                      # 测试 (需要 Roblox Cloud 环境)
```

**检查点**：
- [ ] `cargo test` 通过
- [ ] `cargo build --release` 成功
- [ ] `npm run config:build && npm run build && npm test` 通过

## 开发注意事项

- SWC `TsUnionType` → `TsUnionOrIntersectionType::TsUnionType`
- `TsParser` 不是 `Sync`，并行闭包中需创建新实例
- 注释在 `export.span.lo` 位置
- 装饰器两遍扫描：先收集 @LubanTable，再解析 @Ref

## 发布流程

```bash
# 更新版本号 (Cargo.toml + luban-ts/package.json)
git add . && git commit -m "release: vX.Y.Z"
git tag vX.Y.Z && git push && git push --tags
```

自动触发 GitHub Actions 构建和发布。
