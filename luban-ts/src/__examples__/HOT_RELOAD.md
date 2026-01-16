# 热更新示例

两个独立的热更新示例，可在 Studio 中手动验证。

## 快速开始

1. 编译项目：`npm run build`
2. 在 Studio 中打开项目
3. 选择一个示例文件夹：
   - `hot-reload-loader/` - ConfigLoader 示例
   - `hot-reload-runtime/` - ConfigsRuntime 示例
4. 将文件夹复制到 `ReplicatedStorage/TS/`
5. 运行对应的 `.lua` 主脚本
6. 编辑 `.lua` 配置文件来测试热更新

## ConfigLoader 示例

**文件**：`hot-reload-loader.lua`

**配置文件**：
- `items-config.lua` - 物品配置
- `skills-config.lua` - 技能配置

**使用**：
```lua
-- 1. 编辑 items-config.lua
-- 2. 重新运行 hot-reload-loader.lua
-- 或直接修改 ModuleScript.Source（如果 Studio 允许）
```

## ConfigsRuntime 示例

**文件**：`hot-reload-runtime.lua`

**配置文件**：
- `item-config.lua` - 物品配置（Luban schema）
- `game-config.lua` - 游戏全局配置

**使用**：
```lua
-- 1. 编辑 item-config.lua 或 game-config.lua
-- 2. 重新运行 hot-reload-runtime.lua
-- 或直接修改 ModuleScript.Source（如果 Studio 允许）
```

## 热更新触发方式

### 方式 1：编辑配置文件
1. 直接编辑 `.lua` 配置文件
2. 重新运行主脚本
3. 配置被重新加载

### 方式 2：修改 ModuleScript.Source（如果允许）
1. 在 Studio 中找到 ModuleScript
2. 右键 → View Source
3. 修改内容 → 保存
4. 热更新自动触发

## 测试场景

- 修改物品价格
- 添加新物品
- 修改游戏配置（最大玩家数、调试模式等）
- 观察控制台日志
- 验证定时器访问配置的数据是否更新

详细说明见 `hot-reload-examples/README.md`
