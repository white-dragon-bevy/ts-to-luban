# Luban Schema Generator

高性能 TypeScript 到 Luban XML Schema 生成器，支持装饰器验证和 TypeScript Table 代码生成。

## 功能特性

- **高性能解析**: 使用 SWC 解析 TypeScript，Rayon 并行处理
- **智能缓存**: 基于文件哈希的增量生成，跳过未修改的文件
- **装饰器支持**: `@LubanTable`、`@Ref`、`@Range`、`@Required` 等验证器装饰器
- **TypeScript Table 代码生成**: 自动生成类型安全的 table 加载器
- **JSDoc 注释**: 自动提取 `comment`、`@alias`、`@ignore`、`@flags` 标签
- **父类解析**: 基于 TypeScript `extends` 关键字自动设置 parent
- **枚举支持**: 字符串枚举、数值枚举、位标志枚举
- **多文件输出**: 每个 source 可配置独立 `output_path` 和 `module_name`
- **跨平台**: 支持 Windows、macOS、Linux

## 安装

### 通过 npm (推荐)

```bash
# 配置 GitHub Packages registry
echo "@white-dragon-bevy:registry=https://npm.pkg.github.com" >> ~/.npmrc

# 安装
npm install @white-dragon-bevy/ts-to-luban
```

### 从源码构建

```bash
cargo build --release

# 二进制文件位于
./target/release/luban-gen
```

### 从 GitHub Releases 下载

前往 [Releases](https://github.com/white-dragon-bevy/ts-to-luban/releases) 下载对应平台的二进制文件。

## 快速开始

```bash
# 运行
cargo run -- -c luban-ts/luban.config.toml

# 强制重新生成（忽略缓存）
cargo run -- -c luban-ts/luban.config.toml -f

# 运行测试
cargo test
```

## 核心功能

### 1. XML Schema 生成

将 TypeScript 类/接口转换为 Luban bean 定义。

**输入：**
```typescript
/**
 * 怪物配置
 * @alias:怪物
 */
export class Monster extends BaseEntity {
    public id: number;
    public name: string;
    public skills: number[];
}
```

**输出：**
```xml
<bean name="Monster" parent="BaseEntity" alias="怪物" comment="怪物配置">
    <var name="id" type="double"/>
    <var name="name" type="string"/>
    <var name="skills" type="list,double"/>
</bean>
```

### 2. 父类解析

Bean 的 `parent` 属性**完全由 TypeScript 的 `extends` 关键字决定**。

```typescript
class BaseEntity { public id: number; }
class Player extends BaseEntity { public name: string; }  // parent="BaseEntity"
class Standalone { public data: string; }                 // 无 parent
```

### 3. 装饰器支持

#### @LubanTable 类装饰器

标记类为数据表，自动生成 `<table>` 元素。

```typescript
import { LubanTable } from "@white-dragon-bevy/ts-to-luban";

@LubanTable({ mode: "map", index: "id", group: "client" })
export class TbItem {
    public id: number;
    public name: string;
    public price: number;
}
```

生成：
```xml
<bean name="TbItem">
    <var name="id" type="double"/>
    <var name="name" type="string"/>
    <var name="price" type="double"/>
</bean>

<table name="TbItemTable" value="TbItem" mode="map" index="id"
       input="configs/TbItem.xlsx" output="TbItem" group="client"/>
```

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

**组合示例：**
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
    public id: number;
    public triggers: ObjectFactory<BaseTrigger>[];
}
```

生成：
```xml
<var name="triggers" type="list,BaseTrigger" tags="objectFactory"/>
```

### 4. TypeScript Table 代码生成

自动生成类型安全的 table 加载器，取代 Luban codebuild。

**配置：**
```toml
[output]
path = "configs/defines/generated.xml"
table_output_path = "out/tables"   # 启用 TS 代码生成
```

**生成的文件结构：**
```
out/tables/
├── creators/           # 每个 bean 的 creator 函数
│   ├── monster.ts
│   ├── drop-item.ts
│   └── ...
├── tables/             # 每个 table 的加载器
│   ├── monster.ts
│   └── ...
├── registry.ts         # bean 注册表
└── index.ts            # AllTables 入口
```

**使用示例：**
```typescript
import { createAllTables } from "./out/tables";

const tables = createAllTables((file) => loadJson(file));
const monster = tables.MonsterTable.get(1001);
console.log(monster?.name);
```

### 5. 枚举导出

#### 字符串枚举

```typescript
export enum ItemType {
    Role = "role",
    Consumable = "consumable"
}
```

生成：
```xml
<enum name="ItemType" tags="string">
    <var name="Role" alias="role" value="1"/>
    <var name="Consumable" alias="consumable" value="2"/>
</enum>
```

#### 数值枚举

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

#### 位标志枚举

```typescript
/**
 * 单位权限标志
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
<enum name="UnitFlag" alias="权限" flags="true" comment="单位权限标志">
    <var name="CAN_MOVE" alias="移动" value="1"/>
    <var name="CAN_ATTACK" alias="攻击" value="2"/>
    <var name="BASICS" alias="basics" value="3"/>
</enum>
```

### 6. JSDoc 标签

| 标签 | 说明 | 示例 |
|------|------|------|
| 类注释 | 生成 bean comment | `/** 伤害触发器 */` |
| `@param` | 字段注释 | `@param damage 伤害值` |
| `@alias` | 别名 | `@alias:中文名` 或 `@alias="中文名"` |
| `@ignore` | 忽略导出 | `@ignore` |
| `@flags` | 位标志枚举 | `@flags="true"` |

## 配置文件

完整配置示例 (`luban.config.toml`)：

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "configs/defines/generated.xml"     # 默认 XML 输出路径
cache_file = ".luban-cache.json"           # 缓存文件
module_name = "game"                       # 默认 module name
enum_path = "configs/defines/enums.xml"    # 枚举输出路径
bean_types_path = "configs/defines/bean_types.xml"  # bean 类型枚举
table_output_path = "out/tables"           # TypeScript table 代码输出

# === Sources ===

[[sources]]
type = "directory"
path = "src/configs"
module_name = "configs"

[[sources]]
type = "file"
path = "src/types/special.ts"
output_path = "configs/defines/special.xml"

[[sources]]
type = "files"
paths = ["src/types/a.ts", "src/types/b.ts"]
module_name = "types"

[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"
module_name = "triggers"

# === Table Mappings ===

[[table_mappings]]
pattern = "Tb.*"                           # 正则匹配类名
input = "configs/{name}.xlsx"              # {name} = 类名
output = "{name}"

[[table_mappings]]
pattern = "TbItem"                         # 精确匹配优先
input = "items/item_data.xlsx"
output = "item"

# === Ref Configs ===

[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"

# === Type Mappings ===

[type_mappings]
Vector3 = "Vector3"
Entity = "long"
```

## Source 类型

| 类型 | 字段 | 说明 |
|------|------|------|
| `directory` | `path` | 扫描目录下所有 .ts 文件 |
| `file` | `path` | 单个 .ts 文件 |
| `files` | `paths` | 多个 .ts 文件（数组） |
| `glob` | `pattern` | Glob 模式匹配 |

**通用可选字段**：
- `output_path`: 覆盖默认输出路径
- `module_name`: 覆盖默认 module name
- `scan_options`: 扫描选项（仅 directory）

**scan_options**：
```toml
[[sources]]
type = "directory"
path = "node_modules/some-lib"
scan_options = { include_dts = true, include_node_modules = true }
```

## 内置类型映射

| TypeScript | Luban | 说明 |
|------------|-------|------|
| `number` | `double` | 浮点数 |
| `string` | `string` | 字符串 |
| `boolean` | `bool` | 布尔值 |
| `int` | `int` | 整数 |
| `float` | `float` | 单精度浮点 |
| `long` | `long` | 长整数 |
| `T[]` / `Array<T>` | `list,T` | 列表 |
| `Map<K,V>` / `Record<K,V>` | `map,K,V` | 映射 |

可通过 `[type_mappings]` 添加自定义映射。

## 命令行参数

```bash
luban-gen [OPTIONS]

Options:
  -c, --config <PATH>  配置文件路径 [默认: luban.config.toml]
  -f, --force          强制重新生成（忽略缓存）
  -v, --verbose        显示详细输出
  -h, --help           显示帮助
  -V, --version        显示版本
```

## luban-ts/ 项目开发

`luban-ts/` 是一个完整的 roblox-ts 示例项目，展示如何使用 ts-to-luban 工具。

### 项目结构

```
luban-ts/
├── src/
│   ├── index.ts              # 装饰器导出
│   ├── __examples__/          # 示例配置类
│   │   ├── all-validators.ts  # 所有验证器示例
│   │   ├── items.ts           # 物品配置示例
│   │   ├── table-modes.ts     # 不同 mode 示例
│   │   └── inheritance.ts     # 继承关系示例
│   ├── __tests__/             # 测试文件
│   │   └── tables.spec.ts     # 表加载器测试
│   └── ts-tables/             # 生成的 table 加载器
├── configs/                   # Luban 数据
│   ├── defines/               # 生成的 XML 定义
│   ├── datas/                 # 配置 数据源
│   ├── tables/                # Luban 输出表
│   └── jsonConfigs/           # 配置的 JSON 配置
├── luban.config.toml          # ts-to-luban 配置
├── package.json
└── tsconfig.json
```

### 开发工作流程

#### 1. 安装依赖

```bash
cd luban-ts
npm install
```

#### 2. 编写配置类

在 `src/__examples__/` 目录下创建配置类：

```typescript
// src/__examples__/my-config.ts
import { LubanTable, Range, Required } from "../index";

@LubanTable({ mode: "map", index: "id" })
export class MyConfig {
    public id: number;

    @Required()
    public name: string;

    @Range(1, 100)
    public value: number;
}
```

#### 3. 运行 ts-to-luban 生成器

从项目根目录运行：

```bash
# 方式 1: 使用 cargo run
cargo run -- -c luban-ts/luban.config.toml

# 方式 2: 在 luban-ts/ 目录中使用 npm script
cd luban-ts
npm run ts-luban
```

这会生成：
- `configs/defines/*.xml` - Luban XML Schema
- `src/ts-tables/` - TypeScript table 加载器

#### 4. 运行 roblox-ts 编译

```bash
cd luban-ts
npm run build
```

编译 TypeScript 到 Lua，输出到 `out/` 目录。

#### 5. 运行测试

```bash
cd luban-ts
npm run config:build
npm test
```

测试位于 `src/__tests__/tables.spec.ts`，验证：
- Registry bean 创建
- 各 mode 的 table 加载
- dataMap/dataList 访问

### 可用命令

```bash
cd luban-ts

# 编译 TypeScript
npm run build

# 监听模式编译
npm run watch

# 运行测试
npm test

# 重新生成 XML 和 TS table 加载器
npm run ts-luban

# 完整流程：生成 XML + 运行 Luban 转换
npm run config:build
```

### 修改 luban.config.toml

添加新的源文件：

```toml
# 在 [[sources]] 中添加
[[sources]]
type = "file"
path = "src/__examples__/my-config.ts"
output_path = "configs/defines/my-config.xml"
module_name = "myconfig"
```

### 测试生成的 Table

```typescript
// src/__tests__/my-config.spec.ts
import { createAllTables } from "../ts-tables";

export = () => {
    describe("MyConfig", () => {
        it("should load config", () => {
            const tables = createAllTables((file) => {
                if (file === "my-config") {
                    return {
                        "1": { id: 1, name: "Test", value: 50 }
                    };
                }
                return {};
            });

            const config = tables.MyConfigTable.get(1);
            expect(config).to.be.ok();
            expect(config!.name).to.equal("Test");
        });
    });
};
```

## 开发

```bash
# 运行测试
cargo test

# 运行特定模块测试
cargo test parser
cargo test config

# 构建发布版本
cargo build --release
```

## 发布新版本

```bash
# 1. 更新版本号
# 编辑 Cargo.toml 和 package.json 中的 version

# 2. 提交并打 tag
git add .
git commit -m "release: vX.Y.Z"
git tag vX.Y.Z
git push && git push --tags

# 3. 自动流程
# - Release workflow: 构建多平台二进制并发布到 GitHub Releases
# - Publish workflow: 发布到 GitHub Packages
```

## 许可证

MIT License
