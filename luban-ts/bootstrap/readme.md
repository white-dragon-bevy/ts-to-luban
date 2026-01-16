# runtime 启动器

## 功能
1. 默认加载 ReplicatedStorage.TS
2. 允许客户端配置启动脚本: 
2. 允许服务单配置启动脚本: 
3. 允许配置共享启动脚本, 覆盖客户端和服务端配置.


## 如何启动

**默认**
- 修改默认启动脚本: `bootstrap\shared\start.lua`
- 当客户端和共享均为空时生效

**客户端**
- 修改客户端: 修改 `StarterPlayer.StarterPlayerScripts.bootstrap` 的 `Value` 值, 默认为空.
- 使用共享脚本启动: 修改 `ReplicatedStorage.bootstrap` 的 `Value` 值, 默认为空.
- 优先级: 共享脚本 > 客户端

**服务端**
- 修改服务端: 修改 `ServerScriptService.bootstrap` 的 `Value` 值, 默认为空.
- 使用共享脚本启动: 修改 `ReplicatedStorage.bootstrap` 的 `Value` 值, 默认为空.
- 优先级: 共享脚本 > 服务端