
# 产品需求文档 (PRD): Luban Schema 生成器重构 (Rust 版)

## 1. 文档概述

* **项目名称**：Luban Schema Generator (Rust Edition)
* **目标**：替代基于 Node.js 的旧版脚本，通过 Rust 和 SWC 引擎实现超高性能的 TypeScript 到 Luban XML 定义转换。
* **核心价值**：消除构建等待感，简化 CI/CD 环境，确保数据生成的一致性。

## 2. 业务背景与挑战

目前的生成器:`legacy\gen-luban-schema.mjs`, 采用 TypeScript Compiler API，面临以下核心问题：

* **初始化开销大**：TS Compiler 必须加载全量类型上下文，解析速度随项目规模呈线性增长（15s+）。
* **缓存风险**：为提速而引入的 Hash 增量机制在处理类继承、接口变更时容易产生“缓存幻读”，导致生成的 Schema 错误。
* **部署复杂**：Node.js 运行时及庞大的 `node_modules` 增加了环境配置成本。

## 3. 系统架构设计

### 3.1 核心架构图

### 3.2 关键技术栈

* **解析引擎**：`SWC` (基于 Rust 的高性能 TS/JS 解析器)。
* **并行计算**：`Rayon` (数据并行库，用于多核加速文件扫描)。
* **配置解析**：`TOML` + `serde` (易读的配置管理)。
* **命令行交互**：`Clap` (成熟的 Rust CLI 构建工具)。

## 4. 功能需求

### 4.1 输入与配置管理

* **多源输入**：支持在配置文件中定义 `registration` 文件（注册表模式）和 `directory`（全量扫描模式）。
* **路径别名**：必须解析 `tsconfig.json` 中的 `paths` 配置，确保跨目录的 `import` 能够被正确映射到物理磁盘路径。
* **自动忽略**：默认排除 `node_modules` 和 `.d.ts` 文件。

### 4.2 并行解析引擎

采用一次完整解析 + 并行化的方案：

* **并行扫描**：使用 Rayon 并行扫描所有 TS 文件。
* **完整解析**：一次性提取类/接口的所有信息（名称、字段、注释、继承关系）。
* **继承树构建**：在内存中构建全局继承树，用于判定基类关系。
* **按需过滤**：根据配置的基类映射规则，筛选符合条件的类生成 XML。


### 4.3 转换逻辑 (Transformation)

* **JSDoc 提取**：提取成员变量上方的注释，映射为 XML 的 `comment` 属性。
* **泛型支持**：支持识别 `Array<T>` → `list,T`, `Map<K, V>` → `map,K,V`。
* **Union Type**：对于 `A | B` 类型，只取第一个类型。
* **自定义映射**：支持在 TOML 中配置特殊类型的转换逻辑。

### 4.4 基类映射配置

支持在 TOML 中配置接口到基类的映射关系：

```toml
[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

[[base_class_mappings]]
interface = "Component"
maps_to = "TsComponentClass"

[defaults]
base_class = "TsClass"  # 未匹配任何 interface 时使用
```

### 4.5 缓存策略

* **独立缓存文件**：使用 `.luban-cache.json` 存储文件 hash，不侵入生成的 XML 文件。
* **增量编译**：基于源文件 hash 判断是否需要重新生成。
* **缓存可忽略**：缓存文件应加入 `.gitignore`，生成的 XML 保持干净可提交。

## 5. 输出格式规范

### 5.1 XML 输出格式

```xml
<?xml version="1.0" encoding="utf-8"?>
<module name="" comment="自动生成的 ts class Bean 定义">

    <bean name="ClassName" parent="TsTriggerClass" comment="类注释">
        <var name="fieldName" type="int" comment="字段注释"/>
        <var name="optionalField" type="string?"/>
        <var name="listField" type="list,int"/>
        <var name="mapField" type="map,string,int"/>
    </bean>

</module>
```

**规则说明**：
* `parent` 属性根据基类映射配置决定（见 4.4）
* 可选字段类型后缀 `?`（如 `string?`）
* `list` 类型不加 `?` 后缀（空数组等价于 undefined）

### 5.2 缓存文件格式

```json
{
  "version": 1,
  "generated_at": "2024-01-15T10:30:00Z",
  "entries": {
    "HighHealthTrigger": {
      "source": "src/triggers/health.ts",
      "hash": "3486abe51ddd0c83606a93e2a952c98b"
    }
  }
}
```

## 6. 非功能需求

### 6.1 性能指标

* **冷启动速度**：1000 个源文件量级下，全量扫描及生成耗时应 **< 500ms**。
* **内存控制**：峰值内存占用不超过 **200MB**。
* **IO 优化**：只有当新生成的 XML 与原文件内容不一致时，才触发磁盘写入，避免触动外部构建系统的监听器。

### 6.2 易用性与部署

* **零依赖**：交付单个二进制文件，无需安装 Node.js 或 TypeScript。
* **日志系统**：提供详细的错误日志（如：`Error: Class 'A' extends 'B', but 'B' cannot be found in paths.`）。
* **错误处理**：收集所有类型错误后统一报告并终止，便于一次性修复。

### 6.3 分发

* **自动化分发**：GitHub Releases + CI
这是最专业的做法。配置 GitHub Actions，每当你推送到 main 分支时：

自动构建：Actions 自动在 Windows/macOS/Linux 下编译出二进制。

自动发布：直接把 .exe 挂到 GitHub 的 Release 页面。

配合工具：用户可以用 eget 这种工具一键下载： eget your-org/luban-gen

## 7. 配置文件规范

### 7.1 luban.config.toml 完整 Schema

```toml
[project]
tsconfig = "tsconfig.json"  # tsconfig.json 路径

[output]
path = "configs/defines/reflect/generated.xml"  # 输出 XML 路径
cache_file = ".luban-cache.json"                # 缓存文件路径

# 输入源配置（支持多个）
[[sources]]
type = "directory"                              # 目录扫描模式
path = "src/shared/bevy/visual/trigger"

[[sources]]
type = "registration"                           # 注册文件模式
path = "src/types/reflect/registrations.ts"

# 基类映射配置
[[base_class_mappings]]
interface = "EntityTrigger"
maps_to = "TsTriggerClass"

# 默认配置
[defaults]
base_class = "TsClass"                          # 默认基类

# 类型映射扩展（补充内置映射）
[type_mappings]
Vector3 = "Vector3"
Vector2 = "Vector2"
CFrame = "CFrame"
Color3 = "Color3"
AnyEntity = "long"
Entity = "long"
EntityId = "long"
AssetPath = "string"
CastActionTarget = "CastActionTarget"
CastContext = "CastContext"
```

## 8. 方案对比总结

| 特性 | 旧版 (TS API) | 新版 (Rust + SWC) |
| --- | --- | --- |
| **解析方式** | 全量类型检查 (语义解析) | 极速 AST 遍历 (语法解析) |
| **并发支持** | 单线程 | 多线程并行 (Rayon) |
| **缓存机制** | 强依赖 Hash 缓存 | 纯内存全量扫描 |
| **产物** | JS 脚本 | 原生二进制 |
| **缓存存储** | 嵌入 XML 注释 | 独立缓存文件 |

---

## 9. 下一步计划 (Next Steps)

这份 PRD 已经明确了技术路径。按照以下步骤推进：

1. **项目初始化**：创建 Rust 项目，配置 SWC、Rayon、Clap、Serde 依赖。
2. **配置解析**：实现 `luban.config.toml` 解析模块。
3. **路径解析**：集成 tsconfig paths 别名解析。
4. **AST 解析**：使用 SWC 并行解析所有 TS 文件，提取类/接口信息。
5. **类型转换**：实现 TS 类型到 Luban 类型的映射逻辑。
6. **XML 生成**：基于模板生成 Luban XML 输出。
7. **缓存系统**：实现独立缓存文件的读写与增量编译。
8. **CLI 接口**：实现命令行参数解析与帮助信息。