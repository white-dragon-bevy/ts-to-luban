/**
 * Edge Cases and Error Handling Tests
 *
 * Tests for edge cases, boundary conditions, and error scenarios:
 * - null/undefined data
 * - missing optional fields
 * - empty arrays
 * - type coercion
 * - special characters
 */

// Import to register all creators
import "../ts-tables";

import { createBean, createAllTables } from "../ts-tables";

export = () => {
	describe("Null and Undefined Handling", () => {
		it("should handle null data gracefully", () => {
			// In Lua/roblox-ts, null doesn't exist the same way
			// Testing with nil/undefined behavior
			const item = createBean<{ id?: number; name?: string }>("Item", {});
			expect(item).to.be.ok();
		});

		it("should handle undefined fields", () => {
			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", {
				id: 1,
				name: "Player1",
				avatar: "avatar.png",
				// signature is undefined (not provided)
			});
			expect(player.name).to.equal("Player1");
			// Optional field should not cause error
		});

		it("should create bean with minimal data", () => {
			const item = createBean("Item", {});
			expect(item).to.be.ok();
		});
	});

	describe("Missing Optional Fields", () => {
		it("should handle missing optional string field", () => {
			const playerData = {
				id: 1,
				name: "Player1",
				avatar: "avatar.png",
				// signature is missing
			};
			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.name).to.equal("Player1");
			expect(player.avatar).to.equal("avatar.png");
		});

		it("should handle missing optional number field", () => {
			const monsterData = {
				id: 1,
				name: "Monster",
				level: 10,
				hp: 100,
				skills: [],
				drops: [],
			};
			const monster = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops?: unknown[];
			}>("Monster", monsterData);

			expect(monster.name).to.equal("Monster");
			expect(monster.skills.size()).to.equal(0);
		});

		it("should handle all optional fields present", () => {
			const playerData = {
				id: 1,
				name: "Player1",
				avatar: "avatar.png",
				signature: "With signature",
			};
			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.signature).to.equal("With signature");
		});
	});

	describe("Empty Arrays", () => {
		it("should handle empty skill array", () => {
			const monsterData = {
				id: 1,
				name: "NoSkills",
				level: 1,
				hp: 10,
				skills: [],
				drops: [],
			};
			const monster = createBean<{
				id: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(0);
			expect(monster.drops.size()).to.equal(0);
		});

		it("should handle empty dialogue array", () => {
			const npcData = {
				id: 1,
				name: "SilentNPC",
				level: 1,
				hp: 10,
				dialogue: [],
			};
			const npc = createBean<{ dialogue: string[] }>("NPCUnit", npcData);

			expect(npc.dialogue.size()).to.equal(0);
		});

		it("should handle empty substitutes array", () => {
			const teamData = {
				id: 1,
				members: [101, 102, 103],
				substitutes: [],
			};
			const team = createBean<{ members: number[]; substitutes: number[] }>("Team", teamData);

			expect(team.members.size()).to.equal(3);
			expect(team.substitutes.size()).to.equal(0);
		});

		it("should handle empty list mode table", () => {
			const mockLoader = (_file: string): unknown => {
				return [];
			};

			const tables = createAllTables(mockLoader);

			// List mode with empty data
			expect(tables.LeaderboardEntryTable.dataList.size()).to.equal(0);
		});

		it("should handle empty map mode table", () => {
			const mockLoader = (_file: string): unknown => {
				return {};
			};

			const tables = createAllTables(mockLoader);

			// Map mode with empty data
			expect(tables.ItemTable.dataMap.size()).to.equal(0);
			expect(tables.ItemTable.dataList.size()).to.equal(0);
		});
	});

	describe("Nested Empty Objects", () => {
		it("should handle nested empty object in drops", () => {
			const monsterData = {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: [],
				drops: [{}],
			};
			const monster = createBean<{ drops: unknown[] }>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(1);
		});

		it("should handle deeply nested empty structures", () => {
			const monsterData = {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: [],
				drops: [{ itemId: 0, count: 0, probability: 0 }],
			};
			const monster = createBean<{ drops: unknown[] }>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(1);
			const drop = monster.drops[0] as { itemId: number; count: number; probability: number };
			expect(drop.itemId).to.equal(0);
		});
	});

	describe("Type Coercion", () => {
		it("should handle string to number conversion", () => {
			// Test if string numbers are converted to numbers
			const itemData = { id: "1" as unknown as number, name: "Test", category: "weapon", stackLimit: 99 };
			const item = createBean<{ id: number; name: string }>("Item", itemData);

			// If the creator handles type coercion
			expect(item).to.be.ok();
			expect(item.name).to.equal("Test");
		});

		it("should handle number to string", () => {
			const difficultyData = {
				id: 1,
				difficultyLevel: 1,
				difficultyName: 123 as unknown as string,
			};
			const difficulty = createBean<{
				difficultyLevel: number;
				difficultyName: string;
			}>("Difficulty", difficultyData);

			expect(difficulty.difficultyLevel).to.equal(1);
			expect(difficulty).to.be.ok();
		});

		it("should handle boolean values", () => {
			const gameConfigData = {
				id: 1,
				maxPlayers: 100,
				gameVersion: "1.0.0",
				debugMode: true,
			};
			const gameConfig = createBean<{
				debugMode: boolean;
			}>("GameConfig", gameConfigData);

			expect(gameConfig.debugMode).to.equal(true);
		});

		it("should handle zero values", () => {
			const itemData = { id: 0, name: "ZeroItem", category: "weapon", stackLimit: 0 };
			const item = createBean<{ id: number; stackLimit: number }>("Item", itemData);

			expect(item.id).to.equal(0);
			expect(item.stackLimit).to.equal(0);
		});

		it("should handle negative numbers", () => {
			const itemData = { id: -1, name: "NegativeItem", category: "weapon", stackLimit: -99 };
			const item = createBean<{ id: number; stackLimit: number }>("Item", itemData);

			expect(item.id).to.equal(-1);
			expect(item.stackLimit).to.equal(-99);
		});

		it("should handle decimal numbers", () => {
			const weaponData = { id: 1, name: "Test", damage: 15, attackSpeed: 1.5 };
			const weapon = createBean<{ attackSpeed: number }>("Weapon", weaponData);

			expect(weapon.attackSpeed).to.equal(1.5);
		});
	});

	describe("Special Characters in Strings", () => {
		it("should handle strings with spaces", () => {
			const itemData = { id: 1, name: "Sword of Legends", category: "weapon", stackLimit: 99 };
			const item = createBean<{ name: string }>("Item", itemData);

			expect(item.name).to.equal("Sword of Legends");
		});

		it("should handle strings with unicode characters", () => {
			const itemData = { id: 1, name: "⚔️ Sword", category: "weapon", stackLimit: 99 };
			const item = createBean<{ name: string }>("Item", itemData);

			expect(item.name).to.equal("⚔️ Sword");
		});

		it("should handle empty strings", () => {
			const playerData = {
				id: 1,
				name: "",
				avatar: "avatar.png",
			};
			const player = createBean<{ name: string }>("Player", playerData);

			expect(player.name).to.equal("");
		});

		it("should handle strings with quotes", () => {
			const itemData = {
				id: 1,
				name: 'The "Legendary" Sword',
				category: "weapon",
				stackLimit: 99,
			};
			const item = createBean<{ name: string }>("Item", itemData);

			expect(item.name).to.equal('The "Legendary" Sword');
		});

		it("should handle very long strings", () => {
			const longString = string.rep("a", 1000);
			const playerData = {
				id: 1,
				name: longString,
				avatar: "avatar.png",
				signature: longString,
			};
			const player = createBean<{ name: string; signature: string }>("Player", playerData);

			// Use string.sub to verify length by getting full string
			expect(string.sub(player.name, 1, 1000).size()).to.equal(1000);
			expect(string.sub(player.signature, 1, 1000).size()).to.equal(1000);
		});
	});

	describe("Boundary Values", () => {
		it("should handle maximum number values", () => {
			const itemData = {
				id: 2147483647,
				name: "MaxItem",
				category: "weapon",
				stackLimit: 2147483647,
			};
			const item = createBean<{ id: number; stackLimit: number }>("Item", itemData);

			expect(item.id).to.equal(2147483647);
			expect(item.stackLimit).to.equal(2147483647);
		});

		it("should handle minimum number values", () => {
			const itemData = {
				id: -2147483648,
				name: "MinItem",
				category: "weapon",
				stackLimit: -2147483648,
			};
			const item = createBean<{ id: number; stackLimit: number }>("Item", itemData);

			expect(item.id).to.equal(-2147483648);
			expect(item.stackLimit).to.equal(-2147483648);
		});

		it("should handle very large arrays", () => {
			const largeArray: number[] = [];
			for (let i = 1; i <= 100; i++) {
				largeArray.push(i);
			}

			const teamData = {
				id: 1,
				members: largeArray,
				substitutes: [],
			};
			const team = createBean<{ members: number[] }>("Team", teamData);

			expect(team.members.size()).to.equal(100);
			expect(team.members[0]).to.equal(1);
			expect(team.members[99]).to.equal(100);
		});
	});

	describe("Invalid Data Handling", () => {
		it("should throw error for unknown bean type", () => {
			expect(() => {
				createBean("NonExistentBean", { id: 1 });
			}).to.throw();
		});

		it("should throw error for empty bean name", () => {
			expect(() => {
				createBean("", {});
			}).to.throw();
		});

		it("should handle malformed data gracefully", () => {
			// Test with data that doesn't match expected structure
			const item = createBean("Item", { unexpectedField: "value" });
			expect(item).to.be.ok();
		});
	});

	describe("Data Type Variations", () => {
		it("should handle integer vs float distinction", () => {
			const weaponData = { id: 1, name: "Test", damage: 15, attackSpeed: 1.5 };
			const weapon = createBean<{ damage: number; attackSpeed: number }>("Weapon", weaponData);

			expect(weapon.damage).to.equal(15);
			expect(weapon.attackSpeed).to.equal(1.5);
		});

		it("should handle very small decimal values", () => {
			const weaponData = { id: 1, name: "Test", damage: 1, attackSpeed: 0.01 };
			const weapon = createBean<{ attackSpeed: number }>("Weapon", weaponData);

			expect(weapon.attackSpeed).to.equal(0.01);
		});

		it("should handle scientific notation numbers", () => {
			const itemData = {
				id: 1,
				name: "Scientific",
				category: "weapon",
				stackLimit: 1e3,
			};
			const item = createBean<{ stackLimit: number }>("Item", itemData);

			expect(item.stackLimit).to.equal(1000);
		});
	});

	describe("Array Edge Cases", () => {
		it("should handle array with single element", () => {
			const monsterData = {
				id: 1,
				name: "SingleSkill",
				level: 1,
				hp: 10,
				skills: [99],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(1);
			expect(monster.skills[0]).to.equal(99);
		});

		it("should handle array with duplicate values", () => {
			const monsterData = {
				id: 1,
				name: "DuplicateSkills",
				level: 1,
				hp: 10,
				skills: [1, 1, 1],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(3);
			expect(monster.skills[0]).to.equal(1);
			expect(monster.skills[1]).to.equal(1);
			expect(monster.skills[2]).to.equal(1);
		});

		it("should handle array with zeros", () => {
			const monsterData = {
				id: 1,
				name: "ZeroSkills",
				level: 1,
				hp: 10,
				skills: [0, 0, 0],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills[0]).to.equal(0);
			expect(monster.skills[1]).to.equal(0);
			expect(monster.skills[2]).to.equal(0);
		});
	});

	describe("Mixed Valid and Invalid Data", () => {
		it("should handle bean with mix of valid and invalid fields", () => {
			const itemData = {
				id: 1,
				name: "Valid",
				category: "weapon",
				stackLimit: 99,
				// unexpectedField would be ignored
			};
			const item = createBean<{ id: number; name: string }>("Item", itemData);

			expect(item.id).to.equal(1);
			expect(item.name).to.equal("Valid");
		});
	});
};
