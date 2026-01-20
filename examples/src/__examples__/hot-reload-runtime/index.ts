import { ReplicatedStorage } from "@rbxts/services";
import { ConfigsProvider } from "../../configs-provider";


// 指定配置文件夹位置
const rs = ReplicatedStorage as unknown as  { configs: Folder };
const configLoader = new ConfigsProvider(rs.configs)

// 初始化
configLoader.initialize()

// 监听热更新(只在Studio模式下有效)
configLoader.onConfigReloaded.Connect((tableMata, isRemoved) => {
	print(`Config reloaded, isRemoved: ${isRemoved}, tableMata:`,tableMata)
})

export = {}