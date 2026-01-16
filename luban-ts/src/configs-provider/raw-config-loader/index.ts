import Signal from "@rbxts/rbx-better-signal";
import { HotReloader } from "@rbxts/rewire";
import {   RunService } from "@rbxts/services";


/**
 * 加载所有的原始配置表
 * 支持热更新
 */
export class RawConfigLoader {
	private readonly _reloadedModules = new Map<string, ModuleScript>();
	private _tables!: Map<string, unknown>;
	private _isInitialized = false;


	private _hotReloader?: HotReloader;


	/**
	 * 构造函数
	 * @param configsFolder 配置表所在目录, 要求内部全为 ModuleScript
	 * @param enableHotReload 是否强制启用热重载（即使在非 Studio 环境）
	 */
	constructor(private readonly configsFolder: Instance, private readonly enableHotReload = false) {}

	/**
	 * 热重载事件
	 * @param fileName - 配置文件名
	 * @param fullName - 配置文件完整路径
	 * @param isRemoved - 是否为删除事件（true: 文件被删除, false: 文件被变更/添加）
	 */
	public readonly onRawConfigReloaded = new Signal<(fileName: string, fullName: string, isRemoved: boolean) => void>();


	/**
	 * 获取指定表
	 */
	public GetTable(tableName: string): unknown {
		const tables = this.GetTables();
		return tables.get(tableName);
	}

	/**
	 * 获取所有表
	 */
	public GetTables(): Map<string,unknown> {
		if(!this._isInitialized) {
			error("ConfigLoader not initialized");
		}
		return this._tables;
	}


	/**
	 * 初始化配置数据
	 * 调用后会加载所有配置表
	 * 如果是 Studio 环境，则启用热重载功能
	 */
	public initialize(): void {
		// Initialize _tables Map for both Studio and non-Studio modes
		this._tables = new Map<string, unknown>();

		if (RunService.IsStudio()) {
			this._setupHotReload();
		}
		else{
			for (const ins of this.configsFolder.GetChildren()){
				if (ins.IsA("ModuleScript")) {
					this._tables.set(ins.Name, require(ins));
				}
			}
		}

		// Mark as initialized after setup
		this._isInitialized = true;
	}



	/**
	 * 增量重载单个配置表
	 * @param fileName - 配置文件名（不含扩展名）
	 */
	private _reloadSingleTable(fileName: string): void {
		const configFile = this._reloadedModules.get(fileName) ?? this.configsFolder.FindFirstChild(fileName) as ModuleScript;
		const data =require(configFile);
		this._tables.set(fileName, data);
	}


	/**
	 * 设置热更新监听（仅在 Studio 环境下）
	 */
	private _setupHotReload(): void {
		// Load initial tables before setting up hot reload
		for (const ins of this.configsFolder.GetDescendants()){
			if (ins.IsA("ModuleScript")) {
				this._tables.set(ins.Name, require(ins));
			}
		}

		// 创建热重载器
		this._hotReloader = new HotReloader();

		// 扫描配置文件夹，监听所有 ModuleScript 的变化
		this._hotReloader.scan(
			this.configsFolder,
			(module, context) => {
				if (context.isReloading) {
					// 热重载时更新已加载的模块
					this._reloadedModules.set(module.Name, module);
					print(`[ConfigDataProvider] 🔄 Hot reloading config: ${module.Name}`);

					// 使用增量重载，只重新加载变更的配置表
					this._reloadSingleTable(module.Name);

					// 触发配置重载事件 (isRemoved = false)
					this.onRawConfigReloaded.Fire(module.Name, module.GetFullName()!, false);
				}
			},
			(module, context) => {
				// cleanup 回调：在模块变更前或删除时调用
				if (context.isReloading) {
					// 模块即将被重新加载（变更场景）
					print(`[ConfigDataProvider] 🧹 Cleaning up config before reload: ${module.Name}`);
					// 从缓存中移除旧的模块引用
					this._reloadedModules.delete(module.Name);
				} else {
					// 模块被删除（删除场景）
					print(`[ConfigDataProvider] 🗑️  Config file removed: ${module.Name}`);
					// 从缓存中移除
					this._reloadedModules.delete(module.Name);
					// 触发配置删除事件 (isRemoved = true)
					this.onRawConfigReloaded.Fire(module.Name, module.GetFullName()!, true);
				}
			},
		);

		print("[ConfigDataProvider] ✅ Hot reload enabled for config files (incremental mode)");
	}
}

