// Luban decorators and types for roblox-ts

// === 装饰器类型定义（roblox-ts 环境没有标准 lib）===

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type ClassDecorator = (target: any) => any;

type PropertyDecorator = (
  target: object,
  propertyKey: string | symbol
) => void;

// === 类装饰器 ===

export interface LubanTableConfig {
  mode: "map" | "list" | "one" | "singleton";
  index?: string;
  group?: string;
  tags?: string;
}

/**
 * 鲁班表装饰器
 * @param config 
 * @returns 
 */
export function LubanTable(config: LubanTableConfig): ClassDecorator {
  return () => {};
}

// === 字段装饰器 ===

/**
 * 范围验证器
 * @example
 * @Range(1, 100)
 * level: number;  // -> double#range=[1,100]
 */
export function Range(min: number, max: number): PropertyDecorator {
  return () => {};
}

/**
 * 必填验证器
 * @example
 * @Required()
 * name: string;  // -> string!
 */
export function Required(): PropertyDecorator {
  return () => {};
}

/**
 * 大小验证器
 * @example
 * @Size(4)
 * items: number[];  // -> (list#size=4),double
 * 
 * @Size(2, 5)
 * items: number[];  // -> (list#size=[2,5]),double
 */
export function Size(size: number): PropertyDecorator;
export function Size(min: number, max: number): PropertyDecorator;
export function Size(_minOrSize: number, _max?: number): PropertyDecorator {
  return () => {};
}

/**
 * 集合验证器
 * @example
 * @Set(1, 2, 3)
 * type: number;  // -> double#set=1,2,3
 */
export function Set(..._values: (number | string)[]): PropertyDecorator {
  return () => {};
}

/**
 * 索引验证器
 * @example
 * @Index("id")
 * items: Item[];  // -> (list#index=id),Item
 */
export function Index(field: string): PropertyDecorator {
  return () => {};
}

/**
 * 名义类型标记
 */
export function Nominal(): PropertyDecorator {
  return () => {};
}

// === JSDoc 注释说明 ===
// 
// @ref - 引用验证器，用于 scalar、list 元素或 map value
// 字段类型必须是配置在 [tables] 中的 class/interface
// @example
// /**
//  * @ref
//  */
// item: Item;  // -> indexType#ref=module.ItemTable
//
// @refKey - Map Key 引用验证器，仅用于 Map 的 key
// @example
// /**
//  * @refKey
//  * @ref
//  */
// itemToSkill: Map<Item, Skill>;  // key 引用 Item 表，value 引用 Skill 表
//
// @tags="key=value,..." - 自定义标签
// @example
// /**
//  * @tags="RefOverride=true"
//  */
// itemId: number;  // -> double tags="RefOverride=true"

// 鲁班类型标记字段
export type $type<T extends object> = T & {$type: string};

// === 泛型类型 ===

export type ObjectFactory<T> = () => T;

/**
 * Constructor type for storing class references
 * Used for type registration and constraint validation
 *
 * @example
 * export class Config {
 *     public triggerType: Constructor<BaseTrigger>;
 * }
 */
export type Constructor<T> = new (...args: any[]) => T;

/**
 * Writable type to remove readonly modifiers
 * Used to make loaded config data mutable
 *
 * @example
 * interface ReadonlyConfig {
 *   readonly id: number;
 *   readonly name: string;
 * }
 *
 * const config: Writable<ReadonlyConfig> = { id: 1, name: "Test" };
 * config.name = "New"; // OK - readonly removed
 */
export type Writable<T> = {
	-readonly [P in keyof T]: T[P];
};



/**
 * 引用检查
 * 用于检查引用是否合法
 * T 为引用类型
 * 将生成 ref 验证器
 * 要求 T 类型必须拥有 id 成员且为基础类型.
 * 比如生成:
 * type="string#ref=AssetDataTable"
 * 
 * string 从 T 的id 类型推断
 * ref=AssetDataTable 为 T 的表名(必须配置)
 */
export type RefKey<T extends Identifiable> = T['id']

/**
 * 标识对象接口
 */
export interface Identifiable {
    id: string | number;
}
