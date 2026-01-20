-- 技能配置数据
-- 这是独立的 Lua 配置文件，可以直接编辑
-- 使用方法：将此文件的内容复制到 ServerScriptService/HotReloadExample/Skills ModuleScript 的 Source 中

return {
	[1] = { id = 1, name = "Fireball", damage = 50, cooldown = 5 },
	[2] = { id = 2, name = "IceBall", damage = 40, cooldown = 4 },
	[3] = { id = 3, name = "Heal", healAmount = 100, cooldown = 10 },
}
