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
--[[
	*
	 * 范围验证器
	 * @example
	 * @Range(1, 100)
	 * level: number;  // -> double#range=[1,100]
	 
]]
local function Range(min, max)
	return function() end
end
--[[
	*
	 * 必填验证器
	 * @example
	 * @Required()
	 * name: string;  // -> string!
	 
]]
local function Required()
	return function() end
end
--[[
	*
	 * 大小验证器
	 * @example
	 * @Size(4)
	 * items: number[];  // -> (list#size=4),double
	 * 
	 * @Size(2, 5)
	 * items: number[];  // -> (list#size=[2,5]),double
	 
]]
local function Size(_minOrSize, _max)
	return function() end
end
--[[
	*
	 * 集合验证器
	 * @example
	 * @Set(1, 2, 3)
	 * type: number;  // -> double#set=1,2,3
	 
]]
local function Set(...)
	local _values = { ... }
	return function() end
end
--[[
	*
	 * 索引验证器
	 * @example
	 * @Index("id")
	 * items: Item[];  // -> (list#index=id),Item
	 
]]
local function Index(field)
	return function() end
end
--[[
	*
	 * 名义类型标记
	 
]]
local function Nominal()
	return function() end
end
--[[
	*
	 * 引用验证器（已废弃，请使用 JSDoc @ref 代替）
	 * @deprecated Use JSDoc @ref tag instead
	 
]]
-- eslint-disable-next-line @typescript-eslint/no-explicit-any
local function Ref(_target)
	return function() end
end
--[[
	*
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
	 
]]
local function RefReplace()
	return function() end
end
-- === JSDoc 注释说明 ===
-- 
-- @ref - 引用验证器，用于 scalar、list 元素或 map value
-- 字段类型必须是配置在 [tables] 中的 class/interface
-- @example
-- /**
--  * @ref
--  */
-- item: Item;  // -> indexType#ref=module.ItemTable
--
-- @refKey - Map Key 引用验证器，仅用于 Map 的 key
-- @example
-- /**
--  * @refKey
--  * @ref
--  */
-- itemToSkill: Map<Item, Skill>;  // key 引用 Item 表，value 引用 Skill 表
--
-- @tags="key=value,..." - 自定义标签
-- @example
-- /**
--  * @tags="RefOverride=true"
--  */
-- itemId: number;  // -> double tags="RefOverride=true"
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
--[[
	*
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
	 
]]
return {
	LubanTable = LubanTable,
	Range = Range,
	Required = Required,
	Size = Size,
	Set = Set,
	Index = Index,
	Nominal = Nominal,
	Ref = Ref,
	RefReplace = RefReplace,
}
