/**
 * Luban Runtime Loader
 *
 * Main entry point for loading and deserializing Luban configuration tables
 */

import type { AllTables } from "../types/configs/tables";
import { createLoaderMethods } from "./raw-config-parser/loader-methods";
import { loadTableData } from "./raw-config-parser/table-loader";
import { RawConfigLoader } from "./raw-config-loader";
import { RawConfigParser } from "./raw-config-parser";
import Signal from "@rbxts/rbx-better-signal";
import { TableMeta } from "../types/configs/schema";


/**
 * 配置提供者
 */
export class ConfigsProvider {

	private readonly configLoader: RawConfigLoader ;
	private readonly configParser:RawConfigParser

	private allTables: AllTables | undefined;

	private isinitialized: boolean = false;

	/**
	 * 配置变更信号
	 * 当此信号抛出时, 表示运行时的配置表发生了变更
	 */
	public readonly onConfigReloaded = new Signal<(metadata:TableMeta, isRemoved: boolean) => void>();

	constructor(configFolder: Instance, private readonly enableHotReload = false) {
		this.configLoader = new RawConfigLoader(configFolder, enableHotReload);
		this.configParser = new RawConfigParser();
	}

	/**
	 * 	Initializes the ConfigsRuntime, loading all configuration tables.
	 */
	initialize(): void {		
		if(this.isinitialized){
			error("ConfigsRuntime has been initialized");
		}
		this.isinitialized = true
		this.configLoader.initialize();

		// setup hot reload
		this.configLoader.onRawConfigReloaded.Connect((fileName, fullName, isRemoved) => {
			if (isRemoved) {
				// 配置不应该被移除
				error(" Config removal not implemented yet");
			}else{
				// Parse the updated config
				const rawConfig = this.configLoader.GetTable(fileName);
				const config = this.configParser.parseRawConfig(fileName, rawConfig);

				// Get metadata to find the correct table name
				const metadata = this.configParser.getMetadata(fileName);
				if (!metadata) {
					warn(`No metadata found for file: ${fileName}, skipping hot reload`);
					return;
				}

				// Update the table in allTables using the correct table name
				(this.allTables as unknown as Map<string,unknown>).set(metadata.name, config);

				// Fire the outer signal AFTER table is updated
				this.onConfigReloaded.Fire(metadata, isRemoved);
			}
		})

		// load all tables
		this.allTables = this.configParser.parseRawConfigs(this.configLoader.GetTables());
	}


	/**
	 * Returns all loaded configuration tables.
	 */
	getAllTables(): AllTables {
		return this.allTables!
	}

}
