import { ReplicatedStorage } from "@rbxts/services";
import { ConfigsProvider } from "../../configs-provider";


// 指定配置文件夹位置
const rs = ReplicatedStorage as unknown as  { configs: Folder };
const configLoader = new ConfigsProvider(rs.configs)

// 初始化
configLoader.initialize()

export = {}