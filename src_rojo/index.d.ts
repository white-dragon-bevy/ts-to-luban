type ClassDecorator = (target: any) => any;
type PropertyDecorator = (target: object, propertyKey: string | symbol) => void;
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
export declare function LubanTable(config: LubanTableConfig): ClassDecorator;
/**
 * 范围验证器
 * @example
 * @Range(1, 100)
 * level: number;  // -> double#range=[1,100]
 */
export declare function Range(min: number, max: number): PropertyDecorator;
/**
 * 必填验证器
 * @example
 * @Required()
 * name: string;  // -> string!
 */
export declare function Required(): PropertyDecorator;
/**
 * 大小验证器
 * @example
 * @Size(4)
 * items: number[];  // -> (list#size=4),double
 *
 * @Size(2, 5)
 * items: number[];  // -> (list#size=[2,5]),double
 */
export declare function Size(size: number): PropertyDecorator;
export declare function Size(min: number, max: number): PropertyDecorator;
/**
 * 集合验证器
 * @example
 * @Set(1, 2, 3)
 * type: number;  // -> double#set=1,2,3
 */
export declare function Set(..._values: (number | string)[]): PropertyDecorator;
/**
 * 索引验证器
 * @example
 * @Index("id")
 * items: Item[];  // -> (list#index=id),Item
 */
export declare function Index(field: string): PropertyDecorator;
/**
 * 名义类型标记
 */
export declare function Nominal(): PropertyDecorator;
/**
 * 引用验证器（已废弃，请使用 JSDoc @ref 代替）
 * @deprecated Use JSDoc @ref tag instead
 */
export declare function Ref(_target: any): PropertyDecorator;
/**
 * 引用替换装饰器
 * 1. 提供 luban 引用
 * 2. 生成 RefReplace tag
 *
 * @example
 * @RefReplace<Item,"itemName">()
 * 生成的 xml 为 `type="string#ref=ItemTable", tags="RefReplace=itemName"`
 * type="string" 从 Item 的 index 推断
 *
 * @returns
 */
export declare function RefReplace<T extends object, X extends keyof T>(): PropertyDecorator;
export type $type<T extends object> = T & {
    $type: string;
};
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
export type RefKey<T extends {
    id: string | number;
}> = T['id'];
export {};
