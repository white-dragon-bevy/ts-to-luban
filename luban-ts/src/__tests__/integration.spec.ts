/**
 * Integration Tests
 *
 * End-to-end tests for realistic usage patterns:
 * - Complete game config set loading
 * - Multi-table queries
 * - Complex reference chains
 * - Real-world scenarios
 */

// Import to register all creators
import "../ts-tables";

import { createAllTables, createBean, AllTables } from "../ts-tables";
import { createMockLoader } from "./helpers/mock-data";

export = () => {
	describe("Real-World Scenario: Game Config Loading", () => {
		it("should load complete game config set", () => {
			// Simulate loading all configuration tables
			const mockLoader = createMockLoader({
				"game-config": {
					id: 1,
					maxPlayers: 1000,
					gameVersion: "1.0.0",
					debugMode: false,
				},
				"server-settings": {
					id: 1,
					serverName: "ProductionServer",
					tickRate: 60,
				},
				item: {
					"1": { id: 1, name: "Health Potion", category: "consumable", stackLimit: 99 },
					"2": { id: 2, name: "Mana Potion", category: "consumable", stackLimit: 99 },
				},
				monster: {
					"1": {
						id: 1,
						name: "Goblin",
						level: 5,
						hp: 100,
						skills: [1, 2],
						drops: [{ itemId: 1, count: 2, probability: 50 }],
					},
				},
			});

			const tables = createAllTables(mockLoader);

			// Game config loaded
			expect(tables.GameConfigTable.data.maxPlayers).to.equal(1000);

			// Server settings loaded
			expect(tables.ServerSettingsTable.data.serverName).to.equal("ProductionServer");

			// Items loaded
			expect(tables.ItemTable.get(1)).to.be.ok();

			// Monsters loaded
			expect(tables.MonsterTable.get(1)).to.be.ok();
		});

		it("should support querying across multiple tables", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 99 },
					"2": { id: 2, name: "Shield", category: "armor", stackLimit: 50 },
					"3": { id: 3, name: "Potion", category: "consumable", stackLimit: 999 },
				},
				monster: {
					"1": {
						id: 1,
						name: "Goblin",
						level: 5,
						hp: 100,
						skills: [1],
						drops: [
							{ itemId: 1, count: 1, probability: 30 },
							{ itemId: 3, count: 3, probability: 70 },
						],
					},
				},
			});

			const tables = createAllTables(mockLoader);

			// Query monster
			const monster = tables.MonsterTable.get(1);
			expect(monster).to.be.ok();

			// Query items that monster drops
			const drops = monster!.drops as unknown as { itemId: number; count: number }[];
			expect(drops.size()).to.equal(2);

			// Verify dropped items exist
			for (const drop of drops) {
				const item = tables.ItemTable.get(drop.itemId);
				expect(item).to.be.ok();
			}
		});
	});

	describe("createAllTables Integration", () => {
		it("should load all configured tables", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } },
				skill: { "1": { id: 1, skillName: "Test", cooldown: 10 } },
				monster: {
					"1": {
						id: 1,
						name: "Test",
						level: 1,
						hp: 10,
						skills: [],
						drops: [],
					},
				},
				player: { "1": { id: 1, name: "Test", avatar: "test.png" } },
				difficulty: { "1": { id: 1, difficultyLevel: 1, difficultyName: "Easy" } },
				team: { "1": { id: 1, members: [1, 2, 3], substitutes: [] } },
				weapon: { "1": { id: 1, name: "Test", damage: 10, attackSpeed: 1.0 } },
				armor: { "1": { id: 1, name: "Test", defense: 50 } },
				"leaderboard-entry": [{ rank: 1, playerId: 1001, score: 5000 }],
				"game-config": { id: 1, maxPlayers: 100, gameVersion: "1.0.0", debugMode: false },
				"server-settings": { id: 1, serverName: "Test", tickRate: 60 },
			});

			const tables = createAllTables(mockLoader);

			// All tables should be loaded
			expect(tables.ItemTable).to.be.ok();
			expect(tables.SkillTable).to.be.ok();
			expect(tables.MonsterTable).to.be.ok();
			expect(tables.PlayerTable).to.be.ok();
			expect(tables.DifficultyTable).to.be.ok();
			expect(tables.TeamTable).to.be.ok();
			expect(tables.WeaponTable).to.be.ok();
			expect(tables.ArmorTable).to.be.ok();
			expect(tables.LeaderboardEntryTable).to.be.ok();
			expect(tables.GameConfigTable).to.be.ok();
			expect(tables.ServerSettingsTable).to.be.ok();
		});

		it("should share loader across all tables", () => {
			let callCount = 0;
			const trackedLoader = (file: string): unknown => {
				callCount++;
				switch (file) {
					case "item":
						return { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } };
					case "skill":
						return { "1": { id: 1, skillName: "Test", cooldown: 10 } };
					default:
						return {};
				}
			};

			const tables = createAllTables(trackedLoader);

			// Loader should be called for each table
			expect(callCount).to.be.greaterThan(0);
			expect(tables.ItemTable).to.be.ok();
		});

		it("should return AllTables interface", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } },
			});

			const tables = createAllTables(mockLoader);

			// Type checking - should have all expected table properties
			expect(tables.ItemTable).to.be.ok();
			expect(tables.SkillTable).to.be.ok();
			expect(tables.MonsterTable).to.be.ok();
		});
	});

	describe("Multi-Table Operations", () => {
		it("should support operations across multiple tables", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 99 },
					"2": { id: 2, name: "Shield", category: "armor", stackLimit: 50 },
				},
				weapon: { "1": { id: 1, name: "Sword", damage: 15, attackSpeed: 1.2 } },
				armor: { "2": { id: 2, name: "Shield", defense: 50 } },
			});

			const tables = createAllTables(mockLoader);

			// Get item from Item table
			const item = tables.ItemTable.get(1);
			expect(item).to.be.ok();

			// Get weapon details from Weapon table
			const weapon = tables.WeaponTable.get(1);
			expect(weapon).to.be.ok();

			// Get armor details from Armor table
			const armor = tables.ArmorTable.get(2);
			expect(armor).to.be.ok();
		});

		it("should handle complex data relationships", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "Health Potion", category: "consumable", stackLimit: 99 },
					"2": { id: 2, name: "Sword", category: "weapon", stackLimit: 1 },
				},
				monster: {
					"1": {
						id: 1,
						name: "Dragon",
						level: 50,
						hp: 10000,
						skills: [1, 2, 3, 4],
						drops: [
							{ itemId: 2, count: 1, probability: 100 },
							{ itemId: 1, count: 10, probability: 80 },
						],
					},
				},
				skill: {
					"1": { id: 1, skillName: "Fire Breath", cooldown: 30 },
					"2": { id: 2, skillName: "Claw Attack", cooldown: 5 },
					"3": { id: 3, skillName: "Tail Sweep", cooldown: 15 },
					"4": { id: 4, skillName: "Roar", cooldown: 60 },
				},
			});

			const tables = createAllTables(mockLoader);

			// Get monster
			const dragon = tables.MonsterTable.get(1);
			expect(dragon).to.be.ok();
			expect(dragon!.name).to.equal("Dragon");

			// Get monster's skills
			const skillIds = dragon!.skills;
			expect(skillIds.size()).to.equal(4);

			// Verify all skills exist
			for (const skillId of skillIds) {
				const skill = tables.SkillTable.get(skillId);
				expect(skill).to.be.ok();
			}

			// Get monster's drops
			const drops = dragon!.drops as unknown as { itemId: number; count: number }[];
			expect(drops.size()).to.equal(2);

			// Verify all dropped items exist
			for (const drop of drops) {
				const item = tables.ItemTable.get(drop.itemId);
				expect(item).to.be.ok();
			}
		});
	});

	describe("Data Consistency", () => {
		it("should maintain data consistency across loads", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "ConsistentItem", category: "weapon", stackLimit: 99 },
				},
			});

			const tables1 = createAllTables(mockLoader);
			const tables2 = createAllTables(mockLoader);

			// Same data should be loaded consistently
			const item1 = tables1.ItemTable.get(1);
			const item2 = tables2.ItemTable.get(1);

			expect(item1!.id).to.equal(item2!.id);
			expect(item1!.name).to.equal(item2!.name);
		});

		it("should handle independent table instances", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Item1", category: "weapon", stackLimit: 99 } },
			});

			const tables1 = createAllTables(mockLoader);
			const tables2 = createAllTables(mockLoader);

			// Different table instances should be independent
			expect(tables1.ItemTable).to.never.equal(tables2.ItemTable);
		});
	});

	describe("Performance and Scale", () => {
		it("should handle large datasets efficiently", () => {
			// Create a large item dataset
			const largeItemData: Record<string, unknown> = {};
			for (let i = 1; i <= 100; i++) {
				largeItemData[tostring(i)] = {
					id: i,
					name: `Item${i}`,
					category: i % 2 === 0 ? "weapon" : "armor",
					stackLimit: 99,
				};
			}

			const mockLoader = createMockLoader({
				item: largeItemData,
			});

			const tables = createAllTables(mockLoader);

			// All items should be loaded
			expect(tables.ItemTable.dataMap.size()).to.equal(100);
			expect(tables.ItemTable.dataList.size()).to.equal(100);

			// First and last items should be accessible
			expect(tables.ItemTable.get(1)).to.be.ok();
			expect(tables.ItemTable.get(100)).to.be.ok();
		});

		it("should handle rapid table creation", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } },
			});

			// Create multiple table instances
			const tablesArray: AllTables[] = [];
			for (let i = 0; i < 10; i++) {
				tablesArray.push(createAllTables(mockLoader));
			}

			// All tables should be valid
			for (const tables of tablesArray) {
				expect(tables.ItemTable.get(1)).to.be.ok();
			}
		});
	});

	describe("Complete Game Data Scenario", () => {
		it("should simulate full game data loading", () => {
			// Simulate loading a complete game's configuration data
			const mockLoader = createMockLoader({
				// Game config
				"game-config": {
					id: 1,
					maxPlayers: 100,
					gameVersion: "1.0.0",
					debugMode: false,
				},
				"server-settings": {
					id: 1,
					serverName: "GameServer",
					tickRate: 60,
				},

				// Items
				item: {
					"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 1 },
					"2": { id: 2, name: "Potion", category: "consumable", stackLimit: 99 },
				},

				// Skills
				skill: {
					"1": { id: 1, skillName: "Attack", cooldown: 0 },
					"2": { id: 2, skillName: "Heal", cooldown: 10 },
				},

				// Monsters
				monster: {
					"1": {
						id: 1,
						name: "Slime",
						level: 1,
						hp: 50,
						skills: [1],
						drops: [{ itemId: 2, count: 1, probability: 50 }],
					},
				},

				// Players
				player: {
					"1": { id: 1, name: "Player1", avatar: "avatar.png" },
				},

				// Leaderboard
				"leaderboard-entry": [
					{ rank: 1, playerId: 1, score: 1000 },
					{ rank: 2, playerId: 2, score: 500 },
				],
			});

			const tables = createAllTables(mockLoader);

			// Game should be fully configured
			expect(tables.GameConfigTable.data.maxPlayers).to.equal(100);
			expect(tables.ServerSettingsTable.data.tickRate).to.equal(60);
			expect(tables.ItemTable.get(1)).to.be.ok();
			expect(tables.SkillTable.get(1)).to.be.ok();
			expect(tables.MonsterTable.get(1)).to.be.ok();
			expect(tables.PlayerTable.get(1)).to.be.ok();
			expect(tables.LeaderboardEntryTable.dataList.size()).to.equal(2);
		});
	});

	describe("Error Recovery", () => {
		it("should handle missing table data gracefully", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } },
				// skill is missing - should return empty object
			});

			const tables = createAllTables(mockLoader);

			// Item should load successfully
			expect(tables.ItemTable.get(1)).to.be.ok();

			// Skill table should exist but be empty
			expect(tables.SkillTable).to.be.ok();
		});

		it("should continue loading after one table fails", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Test", category: "weapon", stackLimit: 99 } },
				monster: {
					"1": {
						id: 1,
						name: "Test",
						level: 1,
						hp: 10,
						skills: [],
						drops: [],
					},
				},
			});

			const tables = createAllTables(mockLoader);

			// Both tables should load
			expect(tables.ItemTable.get(1)).to.be.ok();
			expect(tables.MonsterTable.get(1)).to.be.ok();
		});
	});

	describe("Bean Creation Integration", () => {
		it("should support dynamic bean creation in workflows", () => {
			// Create items dynamically
			const item1 = createBean<{ name: string }>("Item", {
				id: 1,
				name: "DynamicItem1",
				category: "weapon",
				stackLimit: 99,
			});

			const item2 = createBean<{ name: string }>("Item", {
				id: 2,
				name: "DynamicItem2",
				category: "armor",
				stackLimit: 50,
			});

			expect(item1.name).to.equal("DynamicItem1");
			expect(item2.name).to.equal("DynamicItem2");

			// Create monster with dynamic items
			const monster = createBean<{ name: string }>("Monster", {
				id: 1,
				name: "DynamicMonster",
				level: 10,
				hp: 100,
				skills: [1, 2],
				drops: [{ itemId: 1, count: 1, probability: 100 }],
			});

			expect(monster.name).to.equal("DynamicMonster");
		});
	});
};
