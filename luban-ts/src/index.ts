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

export function LubanTable(config: LubanTableConfig): ClassDecorator {
  return () => {};
}

// === 字段装饰器 ===

export function Ref<T>(target: new (...args: never[]) => T): PropertyDecorator {
  return () => {};
}

export function Range(min: number, max: number): PropertyDecorator {
  return () => {};
}

export function Required(): PropertyDecorator {
  return () => {};
}

export function Size(size: number): PropertyDecorator;
export function Size(min: number, max: number): PropertyDecorator;
export function Size(_minOrSize: number, _max?: number): PropertyDecorator {
  return () => {};
}

export function Set(..._values: (number | string)[]): PropertyDecorator {
  return () => {};
}

export function Index(field: string): PropertyDecorator {
  return () => {};
}

export function Nominal(): PropertyDecorator {
  return () => {};
}

// === 泛型类型 ===

export type ObjectFactory<T> = () => T;
