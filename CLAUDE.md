# Luban Schema Generator

高性能 Rust 工具，将 TypeScript 类/接口转换为 Luban XML Schema 定义，支持装饰器验证和 TypeScript Table 代码生成。

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
cargo run -- -c luban-ts/luban.config.toml

# 强制重新生成（开发时优先使用, 忽略缓存）
cargo run -- -c luban-ts/luban.config.toml -f

# Watch 模式：监控源文件变化并自动重新生成
cargo run -- -c luban-ts/luban.config.toml -w

# 运行测试
cargo test
```

## 项目结构

```
src/
├── main.rs              # CLI 入口
├── lib.rs               # 库导出
├── config.rs            # TOML 配置解析
├── parser.rs            # TypeScript AST 解析 (SWC)
├── parser/
│   ├── class_info.rs    # ClassInfo + LubanTableConfig
│   ├── enum_info.rs     # EnumInfo 结构
│   ├── field_info.rs    # FieldInfo + FieldValidators
│   └── decorator.rs     # 装饰器 AST 解析
├── type_mapper.rs       # TS → Luban 类型映射
├── generator.rs         # XML 生成 (bean, enum, table)
├── validator.rs         # 验证器语法生成
├── table_registry.rs    # @LubanTable 类注册表
├── table_mapping.rs     # table_mappings 解析
├── ts_generator/        # TypeScript 代码生成
│   ├── mod.rs           # TsCodeGenerator 入口
│   ├── creator_gen.rs   # Creator 函数生成
│   ├── table_gen.rs     # Table 加载器生成
│   ├── registry_gen.rs  # Registry 生成
│   ├── index_gen.rs     # Index 入口生成
│   └── import_resolver.rs # Import 路径解析
├── cache.rs             # 增量缓存系统
├── scanner.rs           # 文件扫描
└── tsconfig.rs          # tsconfig.json 路径解析

luban-ts/                # npm 包 (roblox-ts)
├── src/index.ts         # 装饰器定义
├── package.json
└── tsconfig.json
```

## 核心功能

### 1. 类型转换
- `number` → `double`
- `string` → `string`
- `boolean` → `bool`
- `int` / `float` / `long` → 保持原样
- `T[]` / `Array<T>` → `list,T`
- `Map<K,V>` / `Record<K,V>` → `map,K,V`
- `ObjectFactory<T>` → `T` + `tags="ObjectFactory=true"`

### 2. 父类解析

Bean 的 `parent` 属性**完全由 TypeScript 的 `extends` 关键字决定**。

**示例**：
```typescript
class BaseEntity {
    public id: number;
}

class Player extends BaseEntity {
    public name: string;
}

class Standalone {
    public data: string;
}
```

生成：
```xml
<bean name="BaseEntity">
    <var name="id" type="double"/>
</bean>

<bean name="Player" parent="BaseEntity">
    <var name="name" type="string"/>
</bean>

<bean name="Standalone">
    <var name="data" type="string"/>
</bean>
```

**规则**：
- 有 `extends` → 生成 `parent` 属性
- 无 `extends` → 无 `parent` 属性
- class 和 interface 行为一致

### 3. 装饰器支持

#### @LubanTable 类装饰器

标记类为数据表，自动生成 `<table>` 元素。

```typescript
import { LubanTable } from "@white-dragon-bevy/ts-to-luban";

@LubanTable({ mode: "map", index: "id", group: "client" })
export class TbItem {
    public id: number;
    public name: string;
}
```

生成：
```xml
<bean name="TbItem">
    <var name="id" type="double"/>
    <var name="name" type="string"/>
</bean>

<table name="TbItemTable" value="TbItem" mode="map" index="id"
       input="configs/TbItem.xlsx" output="TbItem" group="client"/>
```
**LubanTableConfig 选项**：
- `mode`: `"map"` | `"list"` | `"one"` | `"singleton"`
- `index`: 索引字段名（mode="map" 时必填）
- `group`: 可选，分组标签
- `tags`: 可选，附加标签

#### 字段验证器装饰器

| 装饰器 | 说明 | 生成的 Luban 语法 |
|--------|------|------------------|
| `@Ref(TbItem)` | 引用验证 | `type="double#ref=item.TbItem"` |
| `@Range(1, 100)` | 数值范围 | `type="double#range=[1,100]"` |
| `@Required()` | 必填 | `type="string!"` |
| `@Size(4)` | 固定大小 | `type="(list#size=4),double"` |
| `@Size(2, 5)` | 大小范围 | `type="(list#size=[2,5]),double"` |
| `@Set(1, 2, 3)` | 值集合 | `type="double#set=1,2,3"` |
| `@Index("id")` | 列表索引 | `type="(list#index=id),Foo"` |
| `@Nominal()` | 名义类型 | `nominal="true"` |

**组合示例**：
```typescript
@Ref(TbItem)
@Required()
itemId: number;
// → type="double!#ref=item.TbItem"
```

#### ObjectFactory<T> 泛型

用于延迟创建多态对象：

```typescript
import { ObjectFactory } from "@white-dragon-bevy/ts-to-luban";

export class CharacterConfig {
    public triggers: ObjectFactory<BaseTrigger>[];
}
```

生成：
```xml
<var name="triggers" type="list,BaseTrigger" tags="ObjectFactory=true"/>
```

### 4. TypeScript Table 代码生成

自动生成类型安全的 table 加载器，取代 Luban codebuild。

**配置**：
```toml
[output]
path = "configs/defines/generated.xml"
table_output_path = "out/tables"   # 启用 TS 代码生成
```

**生成的文件结构**：
```
out/tables/
├── creators/           # 每个 bean 的 creator 函数
│   ├── monster.ts
│   └── ...
├── tables/             # 每个 table 的加载器
│   ├── monster.ts
│   └── ...
├── registry.ts         # bean 注册表
└── index.ts            # AllTables 入口
```

**使用示例**：
```typescript
import { createAllTables } from "./out/tables";

const tables = createAllTables((file) => loadJson(file));
const monster = tables.MonsterTable.get(1001);
```

### 5. JSDoc 注释
- 类注释 → `<bean comment="...">`
- `@param` 标签 → `<var comment="...">`
- `@alias` 标签 → `<bean alias="...">` 或 `<enum alias="...">`
  - 支持两种格式：`@alias:别名` 或 `@alias="别名"`
- `@ignore` 标签 → 不导出该类/接口/枚举
- `@flags="true"` → 位标志枚举

### 6. 配置选项

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "output.xml"
module_name = ""                    # 默认为空字符串
enum_path = "output/enums.xml"      # 枚举输出路径
bean_types_path = "output/types.xml" # bean 类型枚举
table_output_path = "out/tables"    # TypeScript table 代码输出

[[sources]]
type = "file"
path = "src/types.ts"
module_name = "types"

[[sources]]
type = "files"
paths = ["src/a.ts", "src/b.ts"]
output_path = "output/ab.xml"

[[sources]]
type = "directory"
path = "src/triggers"
scan_options = { include_dts = true }

[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"
module_name = "triggers"

# Table 映射配置
[[table_mappings]]
pattern = "Tb.*"                    # 正则匹配类名
input = "configs/{name}.xlsx"       # {name} = 类名
output = "{name}"

[[table_mappings]]
pattern = "TbItem"                  # 精确匹配优先
input = "items/item_data.xlsx"
output = "item"

# 引用其他配置
[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"

# 自定义类型映射
[type_mappings]
Vector3 = "Vector3"
Entity = "long"
```

### 7. Source 类型
- `file`: 单个文件
- `files`: 多个文件（共享 output_path 和 module_name）
- `directory`: 目录扫描
- `glob`: Glob 模式匹配（支持 `*`, `**`, `?`, `[abc]`）
- `registration`: 注册文件（未完全实现）

### 8. Enum 导出

**字符串枚举**（新版鲁班自动检测）：
```typescript
export enum ItemType {
    Role = "role",
    Consumable = "consumable"
}
```
生成：
```xml
<enum name="ItemType">
    <var name="Role" alias="role" value="1"/>
    <var name="Consumable" alias="consumable" value="2"/>
</enum>
```

**数值枚举**：
```typescript
export enum SkillStyle {
    Attack = 1,
    Defense = 2
}
```
生成：
```xml
<enum name="SkillStyle">
    <var name="Attack" alias="attack" value="1"/>
    <var name="Defense" alias="defense" value="2"/>
</enum>
```

**位标志枚举**（使用 `@flags="true"`）：
```typescript
/**
 * @flags="true"
 * @alias:权限
 */
export enum UnitFlag {
    /** @alias="移动" */
    CAN_MOVE = 1 << 0,
    /** @alias="攻击" */
    CAN_ATTACK = 1 << 1,
    BASICS = CAN_MOVE | CAN_ATTACK,
}
```
生成：
```xml
<enum name="UnitFlag" alias="权限" flags="true">
    <var name="CAN_MOVE" alias="移动" value="1"/>
    <var name="CAN_ATTACK" alias="攻击" value="2"/>
    <var name="BASICS" alias="basics" value="3"/>
</enum>
```

**规则**：
- 支持位运算表达式：`1 << N`、`A | B`、`A & B`
- 支持枚举成员引用：`BASICS = CAN_MOVE | CAN_ATTACK`
- 字符串枚举 value 从 1 自动递增
- 数值枚举使用原始数值

### 9. Bean 类型枚举导出

将所有 bean 按 parent 分组导出为枚举：

```toml
[output]
bean_types_path = "output/bean_types.xml"
```

生成：
```xml
<enum name="TriggerBaseEnum" comment="TriggerBase 的子类型">
    <var name="DamageTrigger" alias="伤害" value="DamageTrigger"/>
    <var name="HealTrigger" value="HealTrigger"/>
</enum>
```

## 数据结构

### FieldValidators
```rust
pub struct FieldValidators {
    pub ref_target: Option<String>,      // @Ref 目标类
    pub range: Option<(f64, f64)>,       // @Range 范围
    pub required: bool,                  // @Required
    pub size: Option<SizeConstraint>,    // @Size
    pub set_values: Vec<String>,         // @Set 值集合
    pub index_field: Option<String>,     // @Index 字段
    pub nominal: bool,                   // @Nominal
}
```

### LubanTableConfig
```rust
pub struct LubanTableConfig {
    pub mode: String,           // map | list | one | singleton
    pub index: String,          // 索引字段
    pub group: Option<String>,  // 分组
    pub tags: Option<String>,   // 标签
}
```

### FieldInfo
```rust
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub is_optional: bool,
    pub validators: FieldValidators,
    pub is_object_factory: bool,
    pub factory_inner_type: Option<String>,
    pub original_type: String,
}
```

## luban-ts/ 项目开发

`luban-ts/` 是 roblox-ts 示例项目，用于测试和验证 ts-to-luban 工具。

### 项目结构

```
luban-ts/
├── src/
│   ├── index.ts              # 装饰器导出
│   ├── __examples__/          # 示例配置类
│   ├── __tests__/             # 测试
│   └── ts-tables/             # 生成的 table 加载器
├── configs/                   # Luban 数据
│   ├── defines/               # 生成的 XML
│   ├── datas/                 # 配置数据 数据
│   ├── tables/                # Luban 输出
│   └── jsonConfigs/           # 配置的 JSON 输出
└── luban.config.toml
```

### 开发工作流程

```bash
# 1. 安装依赖
cd luban-ts && npm install

# 2. 生成 XML 和 TS table 加载器 和 配置
npm run config:build

# 3. 编译 TypeScript 到 Lua
npm run build

# 4. 运行测试
npm test
```

### 配置类示例

```typescript
// src/__examples__/my-config.ts
import { LubanTable, Range, Required } from "../index";

@LubanTable({ mode: "map", index: "id" })
export class MyConfig {
    public id: number;
    @Required() public name: string;
    @Range(1, 100) public value: number;
}
```

### 测试示例

```typescript
// src/__tests__/my-config.spec.ts
import { createAllTables } from "../ts-tables";

export = () => {
    describe("MyConfig", () => {
        it("should load config", () => {
            const tables = createAllTables((file) => {
                if (file === "my-config") {
                    return { "1": { id: 1, name: "Test", value: 50 } };
                }
                return {};
            });
            const config = tables.MyConfigTable.get(1);
            expect(config!.name).to.equal("Test");
        });
    });
};
```

### 可用命令

```bash
cd luban-ts

npm run build      # 编译 TypeScript
npm run watch      # 监听模式
npm test           # 运行测试
npm run config:build   # 重新生成 XML 和 TS 代码 和配置
```

## 新增功能点开发流程 (TDD)

新增功能点遵循 TDD 驱动开发，每步必须编译通过并验证生成产物。

### 完整流程

```bash
# ==================== Rust 层开发 ====================

# 1. 编写测试（RED）
# 在 src/ 对应文件中添加测试
# → cargo test (确认失败)

# 2. 实现功能（GREEN）
# 编写最小实现代码
# → cargo test (确认通过)

# 3. 重构优化（REFACTOR）
# 在测试保护下重构
# → cargo test (确保仍通过)

# ==================== 验证生成产物 ====================

# 4. 验证编译和生成
cargo build --release           # 编译通过
cargo run -- -c luban-ts/luban.config.toml  # 生成 XML/TS

# ==================== luban-ts 层开发 ====================

# 5. 添加示例和测试
# 编辑 luban-ts/src/__examples__/xxx.ts
# 编辑 luban-ts/src/__tests__/xxx.spec.ts

# 6. 验证 TypeScript 层
cd luban-ts
npm run config:build            # 重新生成代码
npm run build                   # 编译通过
npm test                        # 测试通过
```

### 快速验证命令

```bash
# Rust 层
cargo test                      # 所有测试
cargo test test_name            # 特定测试
cargo build --release           # 编译

# luban-ts 层
cd luban-ts
npm run config:build            # 生成代码
npm run build                   # 编译
npm test                        # 测试
```

### 关键检查点

每完成一个阶段，必须执行验证：

**Rust 完成时：**
- [ ] `cargo test` 通过
- [ ] `cargo build --release` 成功
- [ ] 生成产物正确

**TypeScript 完成时：**
- [ ] `npm run config:build` 成功
- [ ] `npm run build` 成功
- [ ] `npm test` 通过

## 开发注意事项

- SWC 的 `TsUnionType` 在新版本中变为 `TsUnionOrIntersectionType::TsUnionType`
- `TsParser` 包含 `Lrc<SourceMap>` 不是 `Sync`，需要在并行闭包中创建新实例
- 注释附加在 `export` 关键字位置，需要使用 `export.span.lo` 获取
- 装饰器解析使用两遍扫描：第一遍收集 @LubanTable 类，第二遍解析 @Ref 引用
- TypeScript 代码生成使用 kebab-case 文件名

## 发布流程

```bash
# 1. 更新版本号 (Cargo.toml 和 luban-ts/package.json)
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
