--[[
# Bootstrap 启动器核心模块

统一的启动入口，支持客户端和服务端配置

功能:
- 支持通过 ObjectValue 指定启动脚本
- 如果未指定启动脚本，自动显示 __examples__ 选择菜单
- 动态扫描 __examples__ 目录结构，支持多级 Folder 嵌套
- 提供交互式 GUI 选择界面

启动优先级:
1. ObjectValue 指定的启动脚本（最高优先级）
2. defaultScript 参数指定的默认脚本
3. 自动启动 __examples__ 选择菜单（最低优先级）
]]

local RunService = game:GetService("RunService")
local ReplicatedStorage = game:GetService("ReplicatedStorage")
local Players = game:GetService("Players")

local Bootstrap = {}

-- 默认启动脚本目录
local defaultTarget = ReplicatedStorage.TS.__examples__

-- 查找定义启动脚本
local parent = script.Parent :: ObjectValue

-- 没有配置则使用默认启动脚本
local targetShared = parent.Value :: ModuleScript | nil

-- 获取模块路径
local function getPath(moduleScript: ModuleScript)
	local currentParent = moduleScript.Parent
	local path = moduleScript.Name
	while currentParent do
		path = currentParent.Name .. "/" .. path
		currentParent = currentParent.Parent
	end
	return path
end

-- ==================== 样例选择菜单系统 ====================

-- 设置为 true，样例不会自动运行
_G.__select_example_menu__ = true

-- 菜单状态
local currentMenu = nil
local menuStack = {}
local menuGui = nil
local player = Players.LocalPlayer

-- 前置声明函数
local runExample
local showCompletionDialog
local showErrorDialog
local handleItemClick
local handleBackButton
local showHelpDialog

-- 递归扫描文件夹，返回菜单项列表
local function scanFolder(folder: Instance): {any}
	local items = {}
	local children = {}

	-- 收集所有子对象
	for _, child in pairs(folder:GetChildren()) do
		if child.Name ~= "README" then
			table.insert(children, child)
		end
	end

	-- 按名称排序
	table.sort(children, function(a, b)
		return a.Name < b.Name
	end)

	-- 分类处理
	for _, child in ipairs(children) do
		if child:IsA("Folder") then
			-- Folder: 下探一层
			table.insert(items, {
				name = child.Name,
				path = child.Name,
				isFolder = true,
				folder = child
			})
		elseif child:IsA("ModuleScript") then
			-- ModuleScript: 可执行的样例
			table.insert(items, {
				name = child.Name,
				path = child.Name,
				module = child
			})
		end
	end

	return items
end

-- 创建菜单 GUI
local function createMenuGui()
	if menuGui then
		menuGui:Destroy()
	end

	-- 创建主 GUI
	menuGui = Instance.new("ScreenGui")
	menuGui.Name = "ExampleMenuGui"
	menuGui.Parent = player:WaitForChild("PlayerGui")
	menuGui.ResetOnSpawn = false

	-- 创建主框架
	local mainFrame = Instance.new("Frame")
	mainFrame.Name = "MainFrame"
	mainFrame.Size = UDim2.new(0, 600, 0, 500)
	mainFrame.Position = UDim2.new(0.5, -300, 0.5, -250)
	mainFrame.BackgroundColor3 = Color3.fromRGB(40, 40, 40)
	mainFrame.BorderSizePixel = 0
	mainFrame.Parent = menuGui

	-- 添加圆角
	local corner = Instance.new("UICorner")
	corner.CornerRadius = UDim.new(0, 12)
	corner.Parent = mainFrame

	-- 标题栏
	local titleBar = Instance.new("Frame")
	titleBar.Name = "TitleBar"
	titleBar.Size = UDim2.new(1, 0, 0, 50)
	titleBar.Position = UDim2.new(0, 0, 0, 0)
	titleBar.BackgroundColor3 = Color3.fromRGB(30, 30, 30)
	titleBar.BorderSizePixel = 0
	titleBar.Parent = mainFrame

	local titleCorner = Instance.new("UICorner")
	titleCorner.CornerRadius = UDim.new(0, 12)
	titleCorner.Parent = titleBar

	-- 标题文本
	local titleLabel = Instance.new("TextLabel")
	titleLabel.Name = "TitleLabel"
	titleLabel.Size = UDim2.new(1, -100, 1, 0)
	titleLabel.Position = UDim2.new(0, 20, 0, 0)
	titleLabel.BackgroundTransparency = 1
	titleLabel.Text = "🎮 White Dragon Bevy 样例选择器"
	titleLabel.TextColor3 = Color3.fromRGB(255, 255, 255)
	titleLabel.TextScaled = true
	titleLabel.TextXAlignment = Enum.TextXAlignment.Left
	titleLabel.Font = Enum.Font.SourceSansBold
	titleLabel.Parent = titleBar

	-- 关闭按钮
	local closeButton = Instance.new("TextButton")
	closeButton.Name = "CloseButton"
	closeButton.Size = UDim2.new(0, 40, 0, 40)
	closeButton.Position = UDim2.new(1, -50, 0, 5)
	closeButton.BackgroundColor3 = Color3.fromRGB(220, 50, 50)
	closeButton.BorderSizePixel = 0
	closeButton.Text = "✕"
	closeButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	closeButton.TextScaled = true
	closeButton.Font = Enum.Font.SourceSansBold
	closeButton.Parent = titleBar

	local closeCorner = Instance.new("UICorner")
	closeCorner.CornerRadius = UDim.new(0, 8)
	closeCorner.Parent = closeButton

	-- 内容区域
	local contentFrame = Instance.new("Frame")
	contentFrame.Name = "ContentFrame"
	contentFrame.Size = UDim2.new(1, -20, 1, -70)
	contentFrame.Position = UDim2.new(0, 10, 0, 60)
	contentFrame.BackgroundTransparency = 1
	contentFrame.Parent = mainFrame

	-- 滚动框架
	local scrollFrame = Instance.new("ScrollingFrame")
	scrollFrame.Name = "ScrollFrame"
	scrollFrame.Size = UDim2.new(1, 0, 1, -50)
	scrollFrame.Position = UDim2.new(0, 0, 0, 0)
	scrollFrame.BackgroundTransparency = 1
	scrollFrame.BorderSizePixel = 0
	scrollFrame.ScrollBarThickness = 8
	scrollFrame.ScrollBarImageColor3 = Color3.fromRGB(100, 100, 100)
	scrollFrame.Parent = contentFrame

	-- 列表布局
	local listLayout = Instance.new("UIListLayout")
	listLayout.SortOrder = Enum.SortOrder.LayoutOrder
	listLayout.Padding = UDim.new(0, 5)
	listLayout.Parent = scrollFrame

	-- 底部按钮区域
	local buttonFrame = Instance.new("Frame")
	buttonFrame.Name = "ButtonFrame"
	buttonFrame.Size = UDim2.new(1, 0, 0, 40)
	buttonFrame.Position = UDim2.new(0, 0, 1, -40)
	buttonFrame.BackgroundTransparency = 1
	buttonFrame.Parent = contentFrame

	-- 返回按钮
	local backButton = Instance.new("TextButton")
	backButton.Name = "BackButton"
	backButton.Size = UDim2.new(0, 100, 1, 0)
	backButton.Position = UDim2.new(0, 0, 0, 0)
	backButton.BackgroundColor3 = Color3.fromRGB(70, 70, 70)
	backButton.BorderSizePixel = 0
	backButton.Text = "🔙 返回"
	backButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	backButton.TextScaled = true
	backButton.Font = Enum.Font.SourceSans
	backButton.Parent = buttonFrame

	local backCorner = Instance.new("UICorner")
	backCorner.CornerRadius = UDim.new(0, 6)
	backCorner.Parent = backButton

	-- 帮助按钮
	local helpButton = Instance.new("TextButton")
	helpButton.Name = "HelpButton"
	helpButton.Size = UDim2.new(0, 100, 1, 0)
	helpButton.Position = UDim2.new(1, -100, 0, 0)
	helpButton.BackgroundColor3 = Color3.fromRGB(50, 120, 200)
	helpButton.BorderSizePixel = 0
	helpButton.Text = "❓ 帮助"
	helpButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	helpButton.TextScaled = true
	helpButton.Font = Enum.Font.SourceSans
	helpButton.Parent = buttonFrame

	local helpCorner = Instance.new("UICorner")
	helpCorner.CornerRadius = UDim.new(0, 6)
	helpCorner.Parent = helpButton

	return mainFrame, scrollFrame, titleLabel, closeButton, backButton, helpButton
end

-- 显示菜单
local function displayMenu(menuItems, title)
	if not player then return end

	local _mainFrame, scrollFrame, titleLabel, closeButton, backButton, helpButton = createMenuGui()

	-- 更新标题
	titleLabel.Text = "🎮 " .. (title or "White Dragon Bevy 样例选择器")

	-- 清空现有内容
	for _, child in pairs(scrollFrame:GetChildren()) do
		if child:IsA("Frame") then
			child:Destroy()
		end
	end

	if #menuItems == 0 then
		local emptyLabel = Instance.new("TextLabel")
		emptyLabel.Size = UDim2.new(1, 0, 0, 50)
		emptyLabel.BackgroundTransparency = 1
		emptyLabel.Text = "❌ 没有找到任何样例"
		emptyLabel.TextColor3 = Color3.fromRGB(255, 100, 100)
		emptyLabel.TextScaled = true
		emptyLabel.Font = Enum.Font.SourceSans
		emptyLabel.Parent = scrollFrame
		return
	end

	-- 创建菜单项
	for i, item in ipairs(menuItems) do
		local itemFrame = Instance.new("Frame")
		itemFrame.Name = "Item" .. i
		itemFrame.Size = UDim2.new(1, -10, 0, 60)
		itemFrame.BackgroundColor3 = Color3.fromRGB(60, 60, 60)
		itemFrame.BorderSizePixel = 0
		itemFrame.LayoutOrder = i
		itemFrame.Parent = scrollFrame

		local itemCorner = Instance.new("UICorner")
		itemCorner.CornerRadius = UDim.new(0, 8)
		itemCorner.Parent = itemFrame

		-- 图标
		local iconLabel = Instance.new("TextLabel")
		iconLabel.Size = UDim2.new(0, 50, 1, 0)
		iconLabel.Position = UDim2.new(0, 0, 0, 0)
		iconLabel.BackgroundTransparency = 1
		iconLabel.Text = item.isFolder and "📁" or "📄"
		iconLabel.TextColor3 = Color3.fromRGB(255, 255, 255)
		iconLabel.TextScaled = true
		iconLabel.Font = Enum.Font.SourceSans
		iconLabel.Parent = itemFrame

		-- 名称
		local nameLabel = Instance.new("TextLabel")
		nameLabel.Size = UDim2.new(1, -60, 1, 0)
		nameLabel.Position = UDim2.new(0, 55, 0, 0)
		nameLabel.BackgroundTransparency = 1
		nameLabel.Text = item.name
		nameLabel.TextColor3 = Color3.fromRGB(255, 255, 255)
		nameLabel.TextScaled = true
		nameLabel.TextXAlignment = Enum.TextXAlignment.Left
		nameLabel.Font = Enum.Font.SourceSansBold
		nameLabel.Parent = itemFrame

		-- 点击按钮
		local clickButton = Instance.new("TextButton")
		clickButton.Size = UDim2.new(1, 0, 1, 0)
		clickButton.Position = UDim2.new(0, 0, 0, 0)
		clickButton.BackgroundTransparency = 1
		clickButton.Text = ""
		clickButton.Parent = itemFrame

		-- 悬停效果
		clickButton.MouseEnter:Connect(function()
			itemFrame.BackgroundColor3 = Color3.fromRGB(80, 80, 80)
		end)

		clickButton.MouseLeave:Connect(function()
			itemFrame.BackgroundColor3 = Color3.fromRGB(60, 60, 60)
		end)

		-- 点击事件
		clickButton.MouseButton1Click:Connect(function()
			handleItemClick(item)
		end)
	end

	-- 更新滚动框架大小
	local listLayout = scrollFrame:FindFirstChild("UIListLayout")
	if listLayout then
		scrollFrame.CanvasSize = UDim2.new(0, 0, 0, listLayout.AbsoluteContentSize.Y + 10)
	end

	-- 按钮事件
	closeButton.MouseButton1Click:Connect(function()
		menuGui:Destroy()
		menuGui = nil
		currentMenu = nil
	end)

	backButton.MouseButton1Click:Connect(function()
		handleBackButton()
	end)

	helpButton.MouseButton1Click:Connect(function()
		showHelpDialog()
	end)

	-- 显示/隐藏返回按钮
	backButton.Visible = #menuStack > 0
end

-- 显示帮助对话框
showHelpDialog = function()
	-- 创建帮助对话框
	local helpGui = Instance.new("ScreenGui")
	helpGui.Name = "HelpGui"
	helpGui.Parent = player:WaitForChild("PlayerGui")

	-- 背景遮罩
	local overlay = Instance.new("Frame")
	overlay.Size = UDim2.new(1, 0, 1, 0)
	overlay.BackgroundColor3 = Color3.fromRGB(0, 0, 0)
	overlay.BackgroundTransparency = 0.5
	overlay.BorderSizePixel = 0
	overlay.Parent = helpGui

	-- 帮助窗口
	local helpFrame = Instance.new("Frame")
	helpFrame.Size = UDim2.new(0, 500, 0, 400)
	helpFrame.Position = UDim2.new(0.5, -250, 0.5, -200)
	helpFrame.BackgroundColor3 = Color3.fromRGB(50, 50, 50)
	helpFrame.BorderSizePixel = 0
	helpFrame.Parent = helpGui

	local helpCorner = Instance.new("UICorner")
	helpCorner.CornerRadius = UDim.new(0, 12)
	helpCorner.Parent = helpFrame

	-- 标题
	local helpTitle = Instance.new("TextLabel")
	helpTitle.Size = UDim2.new(1, -20, 0, 40)
	helpTitle.Position = UDim2.new(0, 10, 0, 10)
	helpTitle.BackgroundTransparency = 1
	helpTitle.Text = "📖 White Dragon Bevy 样例选择器 - 帮助"
	helpTitle.TextColor3 = Color3.fromRGB(255, 255, 255)
	helpTitle.TextScaled = true
	helpTitle.Font = Enum.Font.SourceSansBold
	helpTitle.Parent = helpFrame

	-- 帮助内容
	local helpContent = Instance.new("ScrollingFrame")
	helpContent.Size = UDim2.new(1, -20, 1, -100)
	helpContent.Position = UDim2.new(0, 10, 0, 50)
	helpContent.BackgroundTransparency = 1
	helpContent.BorderSizePixel = 0
	helpContent.ScrollBarThickness = 6
	helpContent.Parent = helpFrame

	local contentLayout = Instance.new("UIListLayout")
	contentLayout.SortOrder = Enum.SortOrder.LayoutOrder
	contentLayout.Padding = UDim.new(0, 10)
	contentLayout.Parent = helpContent

	local helpTexts = {
		"🎯 功能说明:",
		"• 浏览和运行 White Dragon Bevy 框架的样例代码",
		"• 动态扫描 __examples__ 目录结构",
		"• 支持多级文件夹嵌套",
		"• 自动处理客户端/服务端样例运行",
		"",
		"🖱️ 鼠标操作:",
		"• 点击菜单项 - 选择文件夹或运行样例",
		"• 点击返回按钮 - 返回上级菜单",
		"• 点击关闭按钮 - 退出选择器",
		"• 点击帮助按钮 - 显示此帮助信息",
		"",
		"📁 菜单结构:",
		"• 📁 文件夹 - 包含子样例的分类",
		"• 📄 样例 - 可执行的代码示例",
		"",
		"🚀 样例类型:",
		"• 客户端样例 - 仅在客户端运行",
		"• 服务端样例 - 仅在服务端运行",
		"• 混合样例 - 同时在客户端和服务端运行",
		"",
		"💡 提示:",
		"• 样例运行后可选择返回菜单",
		"• 如果样例出错，请检查控制台错误信息",
		"• 部分样例可能需要特定的 Roblox 环境"
	}

	for i, text in ipairs(helpTexts) do
		local textLabel = Instance.new("TextLabel")
		textLabel.Size = UDim2.new(1, -10, 0, text == "" and 5 or 25)
		textLabel.BackgroundTransparency = 1
		textLabel.Text = text
		textLabel.TextColor3 = text:match("^[🎯🖱️📁🚀💡]") and Color3.fromRGB(100, 200, 255) or Color3.fromRGB(220, 220, 220)
		textLabel.TextScaled = true
		textLabel.TextXAlignment = Enum.TextXAlignment.Left
		textLabel.Font = text:match("^[🎯🖱️📁🚀💡]") and Enum.Font.SourceSansBold or Enum.Font.SourceSans
		textLabel.LayoutOrder = i
		textLabel.Parent = helpContent
	end

	-- 更新滚动大小
	helpContent.CanvasSize = UDim2.new(0, 0, 0, contentLayout.AbsoluteContentSize.Y + 20)

	-- 关闭按钮
	local closeHelpButton = Instance.new("TextButton")
	closeHelpButton.Size = UDim2.new(0, 100, 0, 30)
	closeHelpButton.Position = UDim2.new(0.5, -50, 1, -40)
	closeHelpButton.BackgroundColor3 = Color3.fromRGB(70, 70, 70)
	closeHelpButton.BorderSizePixel = 0
	closeHelpButton.Text = "关闭"
	closeHelpButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	closeHelpButton.TextScaled = true
	closeHelpButton.Font = Enum.Font.SourceSans
	closeHelpButton.Parent = helpFrame

	local closeHelpCorner = Instance.new("UICorner")
	closeHelpCorner.CornerRadius = UDim.new(0, 6)
	closeHelpCorner.Parent = closeHelpButton

	-- 关闭事件
	local function closeHelp()
		helpGui:Destroy()
	end

	closeHelpButton.MouseButton1Click:Connect(closeHelp)
	overlay.MouseButton1Click:Connect(closeHelp)
end

-- 显示错误对话框
showErrorDialog = function(message)
	local errorGui = Instance.new("ScreenGui")
	errorGui.Name = "ErrorGui"
	errorGui.Parent = player:WaitForChild("PlayerGui")

	local overlay = Instance.new("Frame")
	overlay.Size = UDim2.new(1, 0, 1, 0)
	overlay.BackgroundColor3 = Color3.fromRGB(0, 0, 0)
	overlay.BackgroundTransparency = 0.5
	overlay.BorderSizePixel = 0
	overlay.Parent = errorGui

	local errorFrame = Instance.new("Frame")
	errorFrame.Size = UDim2.new(0, 300, 0, 150)
	errorFrame.Position = UDim2.new(0.5, -150, 0.5, -75)
	errorFrame.BackgroundColor3 = Color3.fromRGB(60, 40, 40)
	errorFrame.BorderSizePixel = 0
	errorFrame.Parent = errorGui

	local errorCorner = Instance.new("UICorner")
	errorCorner.CornerRadius = UDim.new(0, 12)
	errorCorner.Parent = errorFrame

	local errorLabel = Instance.new("TextLabel")
	errorLabel.Size = UDim2.new(1, -20, 1, -50)
	errorLabel.Position = UDim2.new(0, 10, 0, 10)
	errorLabel.BackgroundTransparency = 1
	errorLabel.Text = "❌ " .. message
	errorLabel.TextColor3 = Color3.fromRGB(255, 150, 150)
	errorLabel.TextScaled = true
	errorLabel.Font = Enum.Font.SourceSans
	errorLabel.Parent = errorFrame

	local okButton = Instance.new("TextButton")
	okButton.Size = UDim2.new(0, 80, 0, 30)
	okButton.Position = UDim2.new(0.5, -40, 1, -40)
	okButton.BackgroundColor3 = Color3.fromRGB(70, 70, 70)
	okButton.BorderSizePixel = 0
	okButton.Text = "确定"
	okButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	okButton.TextScaled = true
	okButton.Font = Enum.Font.SourceSans
	okButton.Parent = errorFrame

	local okCorner = Instance.new("UICorner")
	okCorner.CornerRadius = UDim.new(0, 6)
	okCorner.Parent = okButton

	okButton.MouseButton1Click:Connect(function()
		errorGui:Destroy()
	end)

	overlay.MouseButton1Click:Connect(function()
		errorGui:Destroy()
	end)
end

-- 显示完成对话框
showCompletionDialog = function(examplePath)
	local completionGui = Instance.new("ScreenGui")
	completionGui.Name = "CompletionGui"
	completionGui.Parent = player:WaitForChild("PlayerGui")

	local overlay = Instance.new("Frame")
	overlay.Size = UDim2.new(1, 0, 1, 0)
	overlay.BackgroundColor3 = Color3.fromRGB(0, 0, 0)
	overlay.BackgroundTransparency = 0.5
	overlay.BorderSizePixel = 0
	overlay.Parent = completionGui

	local completionFrame = Instance.new("Frame")
	completionFrame.Size = UDim2.new(0, 400, 0, 200)
	completionFrame.Position = UDim2.new(0.5, -200, 0.5, -100)
	completionFrame.BackgroundColor3 = Color3.fromRGB(40, 60, 40)
	completionFrame.BorderSizePixel = 0
	completionFrame.Parent = completionGui

	local completionCorner = Instance.new("UICorner")
	completionCorner.CornerRadius = UDim.new(0, 12)
	completionCorner.Parent = completionFrame

	local completionLabel = Instance.new("TextLabel")
	completionLabel.Size = UDim2.new(1, -20, 0, 80)
	completionLabel.Position = UDim2.new(0, 10, 0, 10)
	completionLabel.BackgroundTransparency = 1
	completionLabel.Text = "🔄 样例运行完成！\n" .. examplePath
	completionLabel.TextColor3 = Color3.fromRGB(150, 255, 150)
	completionLabel.TextScaled = true
	completionLabel.Font = Enum.Font.SourceSans
	completionLabel.Parent = completionFrame

	local instructionLabel = Instance.new("TextLabel")
	instructionLabel.Size = UDim2.new(1, -20, 0, 40)
	instructionLabel.Position = UDim2.new(0, 10, 0, 90)
	instructionLabel.BackgroundTransparency = 1
	instructionLabel.Text = "选择下一步操作："
	instructionLabel.TextColor3 = Color3.fromRGB(200, 200, 200)
	instructionLabel.TextScaled = true
	instructionLabel.Font = Enum.Font.SourceSans
	instructionLabel.Parent = completionFrame

	-- 返回菜单按钮
	local backToMenuButton = Instance.new("TextButton")
	backToMenuButton.Size = UDim2.new(0, 120, 0, 35)
	backToMenuButton.Position = UDim2.new(0, 50, 1, -50)
	backToMenuButton.BackgroundColor3 = Color3.fromRGB(50, 120, 200)
	backToMenuButton.BorderSizePixel = 0
	backToMenuButton.Text = "返回菜单"
	backToMenuButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	backToMenuButton.TextScaled = true
	backToMenuButton.Font = Enum.Font.SourceSans
	backToMenuButton.Parent = completionFrame

	local backCorner = Instance.new("UICorner")
	backCorner.CornerRadius = UDim.new(0, 6)
	backCorner.Parent = backToMenuButton

	-- 退出按钮
	local exitButton = Instance.new("TextButton")
	exitButton.Size = UDim2.new(0, 120, 0, 35)
	exitButton.Position = UDim2.new(1, -170, 1, -50)
	exitButton.BackgroundColor3 = Color3.fromRGB(70, 70, 70)
	exitButton.BorderSizePixel = 0
	exitButton.Text = "退出"
	exitButton.TextColor3 = Color3.fromRGB(255, 255, 255)
	exitButton.TextScaled = true
	exitButton.Font = Enum.Font.SourceSans
	exitButton.Parent = completionFrame

	local exitCorner = Instance.new("UICorner")
	exitCorner.CornerRadius = UDim.new(0, 6)
	exitCorner.Parent = exitButton

	backToMenuButton.MouseButton1Click:Connect(function()
		completionGui:Destroy()
		displayMenu(currentMenu.items, currentMenu.title)
	end)

	exitButton.MouseButton1Click:Connect(function()
		completionGui:Destroy()
		if menuGui then
			menuGui:Destroy()
			menuGui = nil
		end
		currentMenu = nil
	end)
end

-- 在服务端运行样例
local function runExampleOnServer(examplePath, modulePath)
	print("🎯 [服务端] 执行样例...")
	local originalFlag = _G.__select_example_menu__
	_G.__select_example_menu__ = false

	local success, result = pcall(function()
		require(modulePath)
	end)

	_G.__select_example_menu__ = originalFlag

	if success then
		print("✅ [服务端] 样例执行完成: " .. examplePath)
		return true
	else
		warn("❌ [服务端] 样例执行失败: " .. examplePath)
		warn("   错误详情: " .. tostring(result))
		return false
	end
end

-- 运行样例
runExample = function(examplePath, modulePath)
	print("\n🚀 正在运行样例: " .. examplePath)
	print("📍 模块路径: " .. tostring(modulePath))

	-- 检查执行模式
	local isServerOnly = examplePath:match("@server$") ~= nil
	local isAll = examplePath:match("@all$") ~= nil

	if RunService:IsClient() then
		if isServerOnly then
			-- @server: 只在服务端执行
			print("🔄 检测到服务端专用模块，发送命令到服务端执行...")

			local remote = ReplicatedStorage:FindFirstChild("RunExampleRemote")
			if remote then
				remote:FireServer(examplePath, modulePath)
				print("📡 已发送服务端运行请求")
			else
				warn("⚠️  无法连接到服务端，服务端部分将不会运行")
			end

			return
		end

		if isAll then
			-- @all: 客户端和服务端同时执行
			print("🔄 检测到全端模块，同时在客户端和服务端执行...")

			local remote = ReplicatedStorage:FindFirstChild("RunExampleRemote")
			if remote then
				remote:FireServer(examplePath, modulePath)
				print("📡 已发送服务端运行请求")
			else
				warn("⚠️  无法连接到服务端，服务端部分将不会运行")
			end
		end

		-- 客户端执行（默认模式或 @all 模式）
		print("🎯 在客户端执行样例...")
		local originalFlag = _G.__select_example_menu__
		_G.__select_example_menu__ = false

		local success, result = pcall(function()
			require(modulePath)
		end)

		_G.__select_example_menu__ = originalFlag

		if success then
			print("✅ 客户端样例执行完成: " .. examplePath)
		else
			warn("❌ 客户端样例执行失败: " .. examplePath)
			warn("   错误详情: " .. tostring(result))
			print("💡 提示: 请检查样例文件是否存在语法错误")
		end
	end
end

-- 处理菜单项点击
handleItemClick = function(item)
	if item.isFolder then
		-- 文件夹：下探一层
		local subItems = scanFolder(item.folder)
		if #subItems > 0 then
			table.insert(menuStack, currentMenu)
			currentMenu = {
				items = subItems,
				title = item.name .. " - 样例列表"
			}
			displayMenu(currentMenu.items, currentMenu.title)
		else
			-- 显示错误提示
			showErrorDialog("该文件夹中没有找到样例")
		end
	else
		-- 运行样例
		runExample(item.path, item.module)

		-- 显示运行完成对话框
		showCompletionDialog(item.path)
	end
end

-- 处理返回按钮
handleBackButton = function()
	if #menuStack > 0 then
		currentMenu = table.remove(menuStack)
		displayMenu(currentMenu.items, currentMenu.title)
	end
end

-- 创建远程事件用于客户端-服务端通信
local runExampleRemote = nil

-- 初始化远程事件
local function initRemoteEvent()
	if RunService:IsServer() then
		-- 服务端创建远程事件
		runExampleRemote = Instance.new("RemoteEvent")
		runExampleRemote.Name = "RunExampleRemote"
		runExampleRemote.Parent = ReplicatedStorage
		print("🔧 [服务端] 创建远程事件: RunExampleRemote")
	elseif RunService:IsClient() then
		-- 客户端等待远程事件
		runExampleRemote = ReplicatedStorage:WaitForChild("RunExampleRemote", 5)
		if runExampleRemote then
			print("🔗 [客户端] 连接到远程事件: RunExampleRemote")
		else
			warn("⚠️  [客户端] 无法连接到远程事件，服务端功能将不可用")
		end
	end
end

-- 初始化并显示样例选择菜单
local function showExampleMenu()
	if not RunService:IsClient() then
		return
	end

	print("🎮 启动 White Dragon Bevy 样例选择器...")

	-- 扫描 __examples__ 目录
	local examplesFolder = ReplicatedStorage.TS.__examples__
	if not examplesFolder then
		showErrorDialog("未找到 __examples__ 目录")
		return
	end

	local mainMenuItems = scanFolder(examplesFolder)

	if #mainMenuItems > 0 then
		currentMenu = {
			items = mainMenuItems,
			title = "主菜单"
		}

		displayMenu(currentMenu.items, currentMenu.title)
	else
		showErrorDialog("__examples__ 目录中没有找到任何样例")
	end
end

-- ==================== Bootstrap 核心启动逻辑 ====================

-- 启动函数
-- @param defaultScript - 默认启动脚本
-- @param targetScript - 目标启动脚本
function Bootstrap.start(defaultScript: ModuleScript | nil, targetScript: ModuleScript | nil)
	-- 初始化远程事件（用于客户端-服务端通信）
	initRemoteEvent()

	-- 服务端远程事件处理
	if RunService:IsServer() then
		local remote = ReplicatedStorage:FindFirstChild("RunExampleRemote")
		if remote then
			remote.OnServerEvent:Connect(function(playerParam, examplePath, modulePath)
				print("📡 [服务端] 收到来自 " .. playerParam.Name .. " 的样例运行请求: " .. examplePath)
				runExampleOnServer(examplePath, modulePath)
			end)

			print("🔧 [服务端] 样例运行服务已启动")
		end
	end

	-- 优先级 1: 使用 ObjectValue 指定的启动脚本
	if targetShared then
		local path = getPath(targetShared)
		print(string.format("[Bootstrap] 使用共享启动脚本: %s", path))
		targetScript = targetShared
	end

	-- 优先级 2: 使用默认启动脚本
	if not targetScript then
		targetScript = defaultScript
		if targetScript then
			local path = getPath(targetScript)
			print(string.format("[Bootstrap] 使用默认启动脚本: %s", path))
		end
	end

	-- 优先级 3: 显示样例选择菜单
	if not targetScript then
		print("[Bootstrap] 未指定启动脚本，显示样例选择菜单")
		showExampleMenu()

		-- 平台
		local part = Instance.new("Part")
		part.Parent = game.Workspace
		part.Size = Vector3.new(100, 100, 100)
		part.Anchored = true
		return
	end


	


	-- 执行启动脚本
	assert(targetScript and targetScript:IsA("ModuleScript"), "moduleScript must be a ModuleScript")
	require(targetScript)
end

return Bootstrap
