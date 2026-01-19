-- Compiled with roblox-ts v3.0.0
-- Luban decorators and types for roblox-ts
-- === 装饰器类型定义（roblox-ts 环境没有标准 lib）===
-- eslint-disable-next-line @typescript-eslint/no-explicit-any
-- === 类装饰器 ===
--[[
	*
	 * 鲁班表装饰器
	 * @param config 
	 * @returns 
	 
]]
local function LubanTable(config)
	return function() end
end
-- === 字段装饰器 ===
local function Ref(target)
	return function() end
end
local function Range(min, max)
	return function() end
end
local function Required()
	return function() end
end
local function Size(_minOrSize, _max)
	return function() end
end
local function Set(...)
	local _values = { ... }
	return function() end
end
local function Index(field)
	return function() end
end
local function Nominal()
	return function() end
end
-- 鲁班类型标记字段
-- === 泛型类型 ===
--[[
	*
	 * Constructor type for storing class references
	 * Used for type registration and constraint validation
	 *
	 * @example
	 * export class Config {
	 *     public triggerType: Constructor<BaseTrigger>;
	 * }
	 
]]
--[[
	*
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
	 
]]
return {
	LubanTable = LubanTable,
	Ref = Ref,
	Range = Range,
	Required = Required,
	Size = Size,
	Set = Set,
	Index = Index,
	Nominal = Nominal,
}
