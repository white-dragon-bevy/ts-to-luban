# Luban Schema Generator

高性能 TypeScript 到 Luban XML Schema 生成器。

## 功能特性

- **高性能解析**: 使用 SWC 解析 TypeScript，Rayon 并行处理
- **智能缓存**: 基于文件哈希的增量生成，跳过未修改的文件
- **JSDoc 注释**: 自动提取类和字段的 JSDoc 注释生成 `comment` 属性
- **灵活配置**: TOML 配置文件，支持正则匹配父类
- **配置导入**: 支持 `ref_configs` 导入其他包的配置，自动合并
- **多文件输出**: 每个 source 可配置独立 `output_path` 和 `module_name`
- **嵌套类型**: 支持类之间的嵌套引用（A → B → C → D）
- **支持 .d.ts**: 可扫描 TypeScript 声明文件
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
# 运行示例
cargo run -- -c example/luban.config.toml

# 强制重新生成（忽略缓存）
cargo run -- -c example/luban.config.toml -f

# 运行测试
cargo test
```

## 配置文件

创建 `luban.config.toml`:

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "output/generated.xml"      # 默认输出路径
cache_file = ".luban-cache.json"   # 缓存文件
module_name = ""                   # 默认 module name（空字符串）

# === Sources ===

# 目录扫描
[[sources]]
type = "directory"
path = "src/triggers"
module_name = "triggers"           # 可选：覆盖默认 module_name

# 单个文件
[[sources]]
type = "file"
path = "src/types/special.ts"
output_path = "output/special.xml" # 可选：独立输出路径

# 多个文件（共享 output_path 和 module_name）
[[sources]]
type = "files"
paths = ["src/types/a.ts", "src/types/b.ts"]
output_path = "output/types.xml"
module_name = "types"

# 扫描 .d.ts 文件
[[sources]]
type = "directory"
path = "node_modules/some-lib"
scan_options = { include_dts = true, include_node_modules = true }

# === Parent Mappings ===

# 正则匹配类名设置 parent
[[parent_mappings]]
pattern = ".*Trigger$"
parent = "TsTriggerClass"

[[parent_mappings]]
pattern = ".*Component$"
parent = "TsComponentClass"

# === Ref Configs ===

# 导入其他包的配置
[[ref_configs]]
path = "./shared-pkg/ts-luban.config.toml"

# === Defaults ===

[defaults]
base_class = "TsClass"             # 默认父类

# === Type Mappings ===

[type_mappings]
Vector3 = "Vector3"
Entity = "long"
CustomType = "string"
```

## Source 类型

| 类型 | 字段 | 说明 |
|------|------|------|
| `directory` | `path` | 扫描目录下所有 .ts 文件 |
| `file` | `path` | 单个 .ts 文件 |
| `files` | `paths` | 多个 .ts 文件（数组） |
| `glob` | `pattern` | Glob 模式匹配文件 |

**通用可选字段**：
- `output_path`: 覆盖默认输出路径
- `module_name`: 覆盖默认 module name（允许空字符串）

### Glob 模式

使用 `glob` 类型可以通过通配符匹配文件：

```toml
[[sources]]
type = "glob"
pattern = "src/**/*Trigger.ts"    # 匹配所有 Trigger 文件
output_path = "output/triggers.xml"
module_name = "triggers"
```

**支持的通配符**：
- `*` - 匹配任意字符（不含路径分隔符）
- `**` - 匹配任意层级目录
- `?` - 匹配单个字符
- `[abc]` - 匹配括号内的任意字符

**示例**：
- `src/**/*.ts` - src 目录下所有 .ts 文件
- `src/**/events/*.ts` - src 下任意层级的 events 目录中的 .ts 文件
- `src/triggers/*Trigger.ts` - triggers 目录下以 Trigger.ts 结尾的文件

## 父类解析

**优先级**: `parent_mappings` 正则匹配 > `defaults.base_class`

> **注意**: TypeScript 的 `extends` 关键字被忽略，parent 完全由配置决定。

```toml
[[parent_mappings]]
pattern = ".*Trigger$"    # 正则匹配类名
parent = "TsTriggerClass" # 匹配成功使用此 parent
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
| `double` | `double` | 双精度浮点 |
| `T[]` / `Array<T>` | `list,T` | 列表 |
| `Map<K,V>` / `Record<K,V>` | `map,K,V` | 映射 |
| `Vector3` | `Vector3` | 三维向量 |
| `Vector2` | `Vector2` | 二维向量 |
| `Entity` / `AnyEntity` | `long` | 实体 ID |

可通过 `[type_mappings]` 添加自定义映射。

## 生成规则

### 类 (class)

```typescript
/**
 * 伤害触发器
 * @param damage 伤害值
 */
export class DamageTrigger {
    public damage: number;
    public targets?: string[];
}
```

生成：

```xml
<bean name="DamageTrigger" parent="TsTriggerClass" comment="伤害触发器">
    <var name="damage" type="double" comment="伤害值"/>
    <var name="targets" type="list,string"/>
</bean>
```

### 接口 (interface)

接口默认**没有 parent 属性**，但如果 extends 其他接口则会继承：

```typescript
// 无 extends → 无 parent
export interface Config {
    id: number;
    name: string;
}

// 有 extends → parent = 父接口名
export interface SpecialConfig extends Config {
    extra: string;
}
```

生成：

```xml
<bean name="Config">
    <var name="id" type="double"/>
    <var name="name" type="string"/>
</bean>

<bean name="SpecialConfig" parent="Config">
    <var name="extra" type="string"/>
</bean>
```

### 嵌套引用

支持类之间的嵌套引用：

```typescript
export interface DConfig { id: number; }
export class CComponent { config: DConfig; }
export class BModule { component: CComponent; }
export class ASystem { module: BModule; modules: BModule[]; }
```

生成：

```xml
<bean name="DConfig">
    <var name="id" type="double"/>
</bean>
<bean name="CComponent" parent="TsClass">
    <var name="config" type="DConfig"/>
</bean>
<bean name="BModule" parent="TsClass">
    <var name="component" type="CComponent"/>
</bean>
<bean name="ASystem" parent="TsClass">
    <var name="module" type="BModule"/>
    <var name="modules" type="list,BModule"/>
</bean>
```

## Ref Configs

导入其他包的配置，自动合并：

```toml
[[ref_configs]]
path = "../shared-pkg/ts-luban.config.toml"
```

**合并规则**：
- `sources`: 路径相对于被引用配置文件位置解析
- `parent_mappings`: 根配置优先，相同 pattern 不覆盖
- 支持递归引用

## 命令行

```bash
luban-gen [OPTIONS]

Options:
  -c, --config <PATH>  配置文件路径 [默认: luban.config.toml]
  -f, --force          强制重新生成（忽略缓存）
  -v, --verbose        显示详细输出
  -h, --help           显示帮助
  -V, --version        显示版本
```

## 项目结构

```
src/
├── main.rs          # CLI 入口
├── config.rs        # TOML 配置解析
├── parser.rs        # TypeScript AST 解析 (SWC)
├── parser/
│   ├── class_info.rs   # ClassInfo 结构
│   └── field_info.rs   # FieldInfo 结构
├── type_mapper.rs   # 类型映射
├── base_class.rs    # 父类解析（忽略 extends）
├── generator.rs     # XML 生成
├── cache.rs         # 增量缓存
├── scanner.rs       # 文件扫描
└── tsconfig.rs      # tsconfig.json 解析

example/
├── luban.config.toml
├── src/triggers/
│   ├── damage.ts
│   ├── spawn.ts
│   └── nested.ts    # 嵌套引用示例
└── shared-pkg/      # ref_configs 示例
```

## 开发

```bash
# 运行测试
cargo test

# 运行特定模块测试
cargo test parser
cargo test type_mapper

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
