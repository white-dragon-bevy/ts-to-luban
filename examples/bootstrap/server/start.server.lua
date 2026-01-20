-- 启动器

-- 默认启动脚本
local defaultScript = nil

local ReplicatedStorage = game:GetService("ReplicatedStorage")

-- 等待 bootstrap 模块加载
local bootstrapFolder = ReplicatedStorage:WaitForChild("bootstrap")
local startModule = bootstrapFolder:WaitForChild("start")

-- 加载 Bootstrap 核心模块
local bootstrap = require(startModule)

-- 查找客户端自定义启动脚本
local parent = script.Parent :: ObjectValue

-- 执行启动
bootstrap.start(defaultScript, parent.Value)
