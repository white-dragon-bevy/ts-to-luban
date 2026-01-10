# TypeScript Table 代码生成设计

## 概述

由 ts-to-luban (Rust) 直接生成 TypeScript table 代码，完全取代 Luban codebuild。

## 工作流程

```
用户 TypeScript 源码（class + @LubanTable 装饰器）
    ↓
ts-to-luban 生成：
    1. Luban XML Schema（用于验证 Excel/JSON）
    2. TypeScript table 代码（类型安全的加载器）
    ↓
Luban 只做：Excel/JSON → JSON 数据
    ↓
运行时：table 代码加载 JSON → 类型安全的 class 实例
```

## 生成的文件结构

```
{table_output_path}/           # 配置指定
├── creators/                  # 每个 bean 的 creator
│   ├── drop-item.ts
│   ├── monster.ts
│   └── ...
├── tables/                    # 每个 table 的加载器
│   ├── monster-table.ts
│   ├── item-table.ts
│   └── ...
├── registry.ts                # $type → creator 映射
└── index.ts                   # AllTables 入口
```

## Creator 生成

### 基本 Creator

每个 bean (class/interface) 生成一个 creator 函数：

```typescript
// creators/drop-item.ts
import { DropItem } from "../../src/__examples__/all-validators";

export function createDropItem(json: any): DropItem {
    const obj = new DropItem();
    obj.itemId = json.itemId;
    obj.count = json.count;
    obj.probability = json.probability;
    return obj;
}
```

### 嵌套 Bean

```typescript
// creators/monster.ts
import { Monster } from "../../src/__examples__/all-validators";
import { createBean } from "../registry";

export function createMonster(json: any): Monster {
    const obj = new Monster();
    obj.id = json.id;
    obj.name = json.name;
    obj.level = json.level;
    obj.hp = json.hp;
    obj.skills = json.skills as number[];
    // 嵌套 bean 通过 registry 创建
    obj.drops = (json.drops as any[]).map(item => createBean("DropItem", item));
    return obj;
}
```

### ObjectFactory 字段

```typescript
// creators/character-config.ts
import { CharacterConfig } from "../../src/character";
import { createByType } from "../registry";

export function createCharacterConfig(json: any): CharacterConfig {
    const obj = new CharacterConfig();
    obj.id = json.id;
    // ObjectFactory<TsClass>[] → 生成工厂函数数组
    obj.attachs = (json.attachs as any[]).map(item => {
        const data = item;  // 闭包捕获数据
        return () => createByType(data.$type, data);
    });
    return obj;
}
```

## Registry 生成

### 统一 registry 解决循环依赖

```typescript
// registry.ts
type Creator<T> = (json: any) => T;
const beanRegistry: Record<string, Creator<any>> = {};

export function registerCreator(name: string, creator: Creator<any>) {
    beanRegistry[name] = creator;
}

export function createBean<T>(name: string, json: any): T {
    const creator = beanRegistry[name];
    if (!creator) {
        error(`Unknown bean: ${name}`);
    }
    return creator(json);
}

export function createByType<T>(typeName: string, json: any): T {
    return createBean(typeName, json);
}
```

### 命名规则

- `$type` 值 = class 名称（如 `"PlayerInput"`）
- creator 函数名 = `create` + class 名称（如 `createPlayerInput`）

## Table 生成

### Map Mode（按 index 索引）

JSON 格式: `{ "1": {...}, "2": {...} }`

```typescript
// tables/item-table.ts
import { Item } from "../../src/__examples__/all-validators";
import { createBean } from "../registry";

export interface ItemTable {
    readonly dataMap: Map<number, Item>;
    readonly dataList: readonly Item[];
    get(key: number): Item | undefined;
}

export function createItemTable(json: any): ItemTable {
    const dataMap = new Map<number, Item>();
    const dataList: Item[] = [];

    for (const [_key, item] of pairs(json)) {
        const obj = createBean<Item>("Item", item);
        dataList.push(obj);
        dataMap.set(obj.id, obj);
    }

    return {
        dataMap,
        dataList,
        get(key: number) { return dataMap.get(key); }
    };
}
```

### List Mode（纯数组）

JSON 格式: `[{...}, {...}]`

```typescript
// tables/leaderboard-table.ts
export interface LeaderboardEntryTable {
    readonly dataList: readonly LeaderboardEntry[];
}

export function createLeaderboardEntryTable(json: any): LeaderboardEntryTable {
    const dataList = (json as any[]).map(item =>
        createBean<LeaderboardEntry>("LeaderboardEntry", item)
    );
    return { dataList };
}
```

### One / Singleton Mode（单条记录）

JSON 格式: `{...}`

```typescript
// tables/game-config-table.ts
export interface GameConfigTable {
    readonly data: GameConfig;
}

export function createGameConfigTable(json: any): GameConfigTable {
    const data = createBean<GameConfig>("GameConfig", json);
    return { data };
}
```

## AllTables 入口

```typescript
// index.ts
import { registerCreator, createBean, createByType } from "./registry";

// 注册所有 creators（打破循环依赖）
import { createItem } from "./creators/item";
import { createMonster } from "./creators/monster";
import { createDropItem } from "./creators/drop-item";
// ...

registerCreator("Item", createItem);
registerCreator("Monster", createMonster);
registerCreator("DropItem", createDropItem);
// ...

// 导出 table 创建函数
import { createItemTable, ItemTable } from "./tables/item-table";
import { createMonsterTable, MonsterTable } from "./tables/monster-table";
// ...

export interface AllTables {
    readonly ItemTable: ItemTable;
    readonly MonsterTable: MonsterTable;
    readonly GameConfigTable: GameConfigTable;
    // ...
}

export function createAllTables(loader: (file: string) => unknown): AllTables {
    return {
        ItemTable: createItemTable(loader("item-table")),
        MonsterTable: createMonsterTable(loader("monster-table")),
        GameConfigTable: createGameConfigTable(loader("game-config-table")),
        // ...
    };
}

// 导出供外部使用
export { createBean, createByType } from "./registry";
```

## 类型处理规则

### 基础类型映射

| TypeScript 类型 | 处理方式 |
|----------------|---------|
| `number` | 直接赋值 `obj.x = json.x` |
| `string` | 直接赋值 |
| `boolean` | 直接赋值 |
| `T[]` / `Array<T>` | `json.x.map(...)` |
| `Map<K,V>` | 遍历构建 Map |
| `T?` (可选) | `json.x !== undefined ? ... : undefined` |

### Bean 类型

| 类型 | 处理方式 |
|------|---------|
| `SomeBean` | `createBean("SomeBean", json.x)` |
| `SomeBean[]` | `json.x.map(item => createBean("SomeBean", item))` |
| `ObjectFactory<T>` | `() => createByType(json.x.$type, json.x)` |
| `ObjectFactory<T>[]` | `json.x.map(item => () => createByType(item.$type, item))` |

### 多态类型（有 $type）

```typescript
// JSON: { "$type": "PlayerInput", "inputPresetId": "..." }

// 普通多态字段
obj.controller = createByType(json.controller.$type, json.controller);

// ObjectFactory 多态字段
obj.factory = () => createByType(json.factory.$type, json.factory);
```

## 配置扩展

```toml
# luban.config.toml

[output]
path = "configs/defines/examples.xml"
module_name = "examples"
table_output_path = "out/tables"   # 新增：table 代码输出目录

[[sources]]
type = "file"
path = "src/__examples__/all-validators.ts"
```

## Import 路径解析

1. 读取 `tsconfig.json` 获取 `baseUrl` 和 `paths`
2. 如果源文件在 `node_modules` → 保持包名（如 `@white-dragon-bevy/ts-to-luban`）
3. 如果是本地文件 → 计算相对路径
