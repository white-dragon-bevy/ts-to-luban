/**
 * Configs Provider Integration Tests
 *
 * Validates that configs in configs/jsonConfigs are correctly loaded and deserialized
 * into typed configuration objects by RawConfigParser.
 */

import { RawConfigParser } from "../../configs-provider/raw-config-parser";
import { AllTables } from "../../types/configs/tables";

// Extended interface for test mock data tables
interface TestTables extends AllTables {
	ItemTable: Map<string, unknown>;
	MonsterTable: Map<string, unknown>;
	PlayerTable: Map<string, unknown>;
	SkillTable: Map<string, unknown>;
	DifficultyTable: Map<string, unknown>;
	TeamTable: Map<string, unknown>;
}

// Define test data inline to avoid JSON import issues in roblox-ts environment
// These match the actual configs/jsonConfigs/*.json files
const itemConfig = {
	"1": { id: 1, name: "铁剑", category: "weapon", stackLimit: 1 },
	"2": { id: 2, name: "生命药水", category: "consumable", stackLimit: 99 },
	"3": { id: 3, name: "铁矿石", category: "material", stackLimit: 999 },
};

const skillConfig = {
	"1": { id: 1, skillName: "火球术", cooldown: 5 },
	"2": { id: 2, skillName: "治疗术", cooldown: 10 },
};

const monsterConfig = {
	"1": { id: 1, name: "史莱姆", level: 1, hp: 100, skills: [1], drops: [{ itemId: 3, count: 2, probability: 50 }] },
	"2": {
		id: 2,
		name: "哥布林",
		level: 5,
		hp: 500,
		skills: [1, 2],
		drops: [
			{ itemId: 1, count: 1, probability: 10 },
			{ itemId: 2, count: 1, probability: 30 },
		],
	},
};

const playerConfig = {
	"1": { id: 1, name: "玩家1", avatar: "avatar1.png" },
};

const difficultyConfig = {
	"1": { id: 1, difficultyLevel: 1, difficultyName: "简单" },
	"2": { id: 2, difficultyLevel: 2, difficultyName: "普通" },
};

const teamConfig = {
	"1": { id: 1, members: [1, 2, 3], substitutes: [] },
};

const weaponConfig = {
	"1": { id: 1, name: "铁剑", damage: 10, attackSpeed: 1 },
};

const armorConfig = {
	"1": { id: 1, name: "皮甲", defense: 5 },
};

const leaderboardConfig: never[] = [];

const gameConfig = { id: 1, maxPlayers: 100, gameVersion: "1.0.0", debugMode: false };

const serverSettingsConfig = { id: 1, serverName: "MainServer", tickRate: 60 };

export = () => {
	describe("ConfigsProvider Integration", () => {
		let parser: RawConfigParser;
		let allConfigs: Map<string, unknown>;

		beforeEach(() => {
			parser = new RawConfigParser();
			allConfigs = new Map<string, unknown>([
				["examples_itemtable", itemConfig],
				["examples_skilltable", skillConfig],
				["examples_monstertable", monsterConfig],
				["examples_playertable", playerConfig],
				["examples_difficultytable", difficultyConfig],
				["examples_teamtable", teamConfig],
				["items_weapontable", weaponConfig],
				["items_armortable", armorConfig],
				["modes_leaderboardentrytable", leaderboardConfig],
				["modes_gameconfigtable", gameConfig],
				["modes_serversettingstable", serverSettingsConfig],
			]);
		});

		it("should initialize parser successfully", () => {
			expect(parser).to.be.ok();
		});

		it("should parse Item table correctly", () => {
			const tables = parser.parseRawConfigs(allConfigs) as TestTables;

			expect(tables).to.be.ok();
			expect(tables.ItemTable).to.be.ok();

			const itemTable = tables.ItemTable;
			expect(itemTable.size()).to.equal(3);

			const item = itemTable.get("1");
			expect(item).to.be.ok();
			expect((item as { id: number }).id).to.equal(1);
			expect((item as { name: string }).name).to.equal("铁剑");
			expect((item as { category: string }).category).to.equal("weapon");
		});

		it("should parse Monster table with nested structures", () => {
			const tables = parser.parseRawConfigs(allConfigs) as TestTables;
			const monsterTable = tables.MonsterTable;

			expect(monsterTable).to.be.ok();
			expect(monsterTable.size()).to.equal(2);

			const monster = monsterTable.get("1");
			expect(monster).to.be.ok();
			expect((monster as { id: number }).id).to.equal(1);
			expect((monster as { name: string }).name).to.equal("史莱姆");
			expect((monster as { level: number }).level).to.equal(1);
			expect((monster as { hp: number }).hp).to.equal(100);

			// Check skills array
			const skills = (monster as { skills: number[] }).skills;
			expect(skills).to.be.ok();
			expect(skills.size()).to.equal(1);
			expect(skills[0]).to.equal(1);

			// Check drops array
			const drops = (monster as { drops: unknown[] }).drops;
			expect(drops).to.be.ok();
			expect(drops.size()).to.equal(1);
			const drop = drops[0] as { itemId: number; count: number; probability: number };
			expect(drop.itemId).to.equal(3);
			expect(drop.count).to.equal(2);
			expect(drop.probability).to.equal(50);
		});

		it("should parse Player table", () => {
			const tables = parser.parseRawConfigs(allConfigs) as TestTables;
			const playerTable = tables.PlayerTable;

			expect(playerTable).to.be.ok();
			expect(playerTable.size()).to.equal(1);

			const player = playerTable.get("1");
			expect(player).to.be.ok();
			expect((player as { id: number }).id).to.equal(1);
			expect((player as { name: string }).name).to.equal("玩家1");
		});

		it("should parse Weapon table", () => {
			const tables = parser.parseRawConfigs(allConfigs);
			const weaponTable = tables.WeaponTable as unknown as Map<string, unknown>;

			expect(weaponTable).to.be.ok();
			expect(weaponTable.size()).to.equal(1);

			const weapon = weaponTable.get("1");
			expect(weapon).to.be.ok();
			expect((weapon as { id: number }).id).to.equal(1);
			expect((weapon as { name: string }).name).to.equal("铁剑");
		});

		it("should parse Armor table", () => {
			const tables = parser.parseRawConfigs(allConfigs);
			const armorTable = tables.ArmorTable as unknown as Map<string, unknown>;

			expect(armorTable).to.be.ok();
			expect(armorTable.size()).to.equal(1);

			const armor = armorTable.get("1");
			expect(armor).to.be.ok();
			expect((armor as { id: number }).id).to.equal(1);
			expect((armor as { name: string }).name).to.equal("皮甲");
		});

		it("should parse GameConfig as single object", () => {
			const tables = parser.parseRawConfigs(allConfigs);
			const gameConfigData = tables.GameConfigTable as unknown;

			expect(gameConfigData).to.be.ok();
			expect((gameConfigData as { id: number }).id).to.equal(1);
			expect((gameConfigData as { maxPlayers: number }).maxPlayers).to.equal(100);
			expect((gameConfigData as { gameVersion: string }).gameVersion).to.equal("1.0.0");
			expect((gameConfigData as { debugMode: boolean }).debugMode).to.equal(false);
		});

		it("should parse ServerSettings as singleton object", () => {
			const tables = parser.parseRawConfigs(allConfigs);
			const serverSettingsData = tables.ServerSettingsTable as unknown;

			expect(serverSettingsData).to.be.ok();
			expect((serverSettingsData as { id: number }).id).to.equal(1);
			expect((serverSettingsData as { serverName: string }).serverName).to.equal("MainServer");
			expect((serverSettingsData as { tickRate: number }).tickRate).to.equal(60);
		});

		it("should parse LeaderboardEntry table as empty array", () => {
			const tables = parser.parseRawConfigs(allConfigs);
			const leaderboardTable = tables.LeaderboardEntryTable as unknown[];

			expect(leaderboardTable).to.be.ok();
			expect(leaderboardTable.size()).to.equal(0);
		});

		describe("getMetadata", () => {
			it("should return metadata for valid table", () => {
				const metadata = parser.getMetadata("examples_itemtable");

				expect(metadata).to.be.ok();
				expect(metadata!.name).to.equal("ItemTable");
				expect(metadata!.file).to.equal("examples_itemtable");
				expect(metadata!.mode).to.equal("map");
				expect(metadata!.index).to.equal("id");
				expect(metadata!.value_type).to.equal("examples.Item");
			});

			it("should return undefined for invalid table", () => {
				const metadata = parser.getMetadata("non_existent_table");
				expect(metadata).to.equal(undefined);
			});

			it("should return correct metadata for list mode table", () => {
				const metadata = parser.getMetadata("modes_leaderboardentrytable");

				expect(metadata).to.be.ok();
				expect(metadata!.mode).to.equal("list");
				expect(metadata!.value_type).to.equal("modes.LeaderboardEntry");
			});

			it("should return correct metadata for one mode table", () => {
				const metadata = parser.getMetadata("modes_gameconfigtable");

				expect(metadata).to.be.ok();
				expect(metadata!.mode).to.equal("one");
			});
		});

		describe("Error Handling", () => {
			it("should error when parsing non-existent table", () => {
				expect(() => {
					parser.parseRawConfig("non_existent_table", {});
				}).to.throw();
			});

			it("should error when table data is undefined", () => {
				expect(() => {
					parser.parseRawConfig("examples_itemtable", undefined);
				}).to.throw();
			});
		});
	});
};
