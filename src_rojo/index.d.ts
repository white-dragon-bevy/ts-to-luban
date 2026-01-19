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
export declare function Ref<T>(target: new (...args: never[]) => T): PropertyDecorator;
export declare function Range(min: number, max: number): PropertyDecorator;
export declare function Required(): PropertyDecorator;
export declare function Size(size: number): PropertyDecorator;
export declare function Size(min: number, max: number): PropertyDecorator;
export declare function Set(..._values: (number | string)[]): PropertyDecorator;
export declare function Index(field: string): PropertyDecorator;
export declare function Nominal(): PropertyDecorator;
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
export {};
