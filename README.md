# Luban Schema Generator

高性能 TypeScript 到 Luban XML Schema 生成器。

## 功能特性

- **高性能解析**: 使用 SWC 解析 TypeScript，支持并行处理
- **智能缓存**: 基于文件哈希的增量生成，跳过未修改的文件
- **灵活配置**: TOML 配置文件，支持自定义类型映射和基类映射
- **跨平台**: 支持 Windows、macOS、Linux

## 安装

### 从源码构建

```bash
cargo build --release
```

编译后的二进制文件位于 `target/release/luban-gen`。

### 从 Release 下载

前往 [Releases](../../releases) 页面下载对应平台的预编译二进制文件。

## 快速开始

### 运行示例

项目包含一个完整的示例，可以直接运行测试：

```bash
# 使用 cargo run 运行示例
cargo run -- -c example/luban.config.toml

# 或者先构建再运行
cargo build --release
./target/release/luban-gen -c example/luban.config.toml
```

运行后会在 `example/output/generated.xml` 生成结果。

### 1. 创建配置文件

在项目根目录创建 `luban.config.toml`:

```toml
[project]
tsconfig = "tsconfig.json"

[output]
path = "configs/defines/generated.xml"
cache_file = ".luban-cache.json"

# 源文件目录
[[sources]]
type = "directory"
path = "src/shared/triggers"

# 基类映射（可选）
[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

# 默认配置
[defaults]
base_class = "TsClass"

# 自定义类型映射（可选）
[type_mappings]
Vector3 = "Vector3"
Entity = "long"
```

### 2. 运行生成器

```bash
# 使用默认配置文件
luban-gen

# 指定配置文件
luban-gen -c path/to/config.toml

# 强制重新生成（忽略缓存）
luban-gen --force

# 显示详细输出
luban-gen --verbose
```

## 配置说明

### project

| 字段 | 类型 | 说明 |
|------|------|------|
| `tsconfig` | string | TypeScript 配置文件路径，用于解析路径别名 |

### output

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `path` | string | 必填 | 输出 XML 文件路径 |
| `cache_file` | string | `.luban-cache.json` | 缓存文件路径 |

### sources

支持两种源类型：

```toml
# 目录扫描
[[sources]]
type = "directory"
path = "src/triggers"

# 注册文件（暂未实现）
[[sources]]
type = "registration"
path = "src/registry.ts"
```

### base_class_mappings

根据类实现的接口确定其父类：

```toml
[[base_class_mappings]]
interface = "EntityTrigger"    # 接口名
maps_to = "TsTriggerClass"     # 映射到的父类
```

### defaults

| 字段 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `base_class` | string | `TsClass` | 默认父类名称 |

### type_mappings

自定义 TypeScript 类型到 Luban 类型的映射：

```toml
[type_mappings]
Vector3 = "Vector3"
Entity = "long"
CustomType = "string"
```

## 内置类型映射

| TypeScript | Luban |
|------------|-------|
| `number` | `int` |
| `string` | `string` |
| `boolean` | `bool` |
| `T[]` / `Array<T>` | `list,T` |
| `Map<K,V>` / `Record<K,V>` | `map,K,V` |
| `Vector3` | `Vector3` |
| `Vector2` | `Vector2` |
| `Entity` / `AnyEntity` | `long` |

## TypeScript 类示例

### 输入

```typescript
export class DamageTrigger implements EntityTrigger {
    public damage: number;
    public radius: number;
    public targets?: string[];
}
```

### 输出

```xml
<bean name="DamageTrigger" parent="TsTriggerClass">
    <var name="damage" type="int"/>
    <var name="radius" type="int"/>
    <var name="targets" type="list,string"/>
</bean>
```

## 命令行参数

```
luban-gen [OPTIONS]

Options:
  -c, --config <CONFIG>  配置文件路径 [默认: luban.config.toml]
  -f, --force            强制重新生成（忽略缓存）
  -v, --verbose          显示详细输出
  -h, --help             显示帮助信息
  -V, --version          显示版本号
```

## 开发

### 运行测试

```bash
# 运行所有测试
cargo test

# 运行特定模块测试
cargo test parser
cargo test generator

# 运行集成测试
cargo test --test integration
```

### 构建发布版本

```bash
cargo build --release
```

## 项目结构

```
├── src/
│   ├── main.rs          # CLI 入口
│   ├── lib.rs           # 库导出
│   ├── config.rs        # 配置解析
│   ├── tsconfig.rs      # TSConfig 路径解析
│   ├── parser.rs        # TypeScript AST 解析
│   ├── parser/
│   │   ├── class_info.rs   # 类信息结构
│   │   └── field_info.rs   # 字段信息结构
│   ├── type_mapper.rs   # 类型映射
│   ├── base_class.rs    # 基类解析
│   ├── generator.rs     # XML 生成
│   ├── cache.rs         # 缓存系统
│   └── scanner.rs       # 文件扫描
├── example/             # 示例项目
│   ├── luban.config.toml
│   ├── tsconfig.json
│   └── src/triggers/    # 示例 TypeScript 文件
└── tests/               # 测试
    ├── integration.rs
    └── fixtures/
```

## 许可证

MIT License
