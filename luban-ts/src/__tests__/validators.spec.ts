/**
 * Validator Bean Tests
 *
 * Tests that beans with various validators are created correctly from JSON data.
 * These tests use the actual example classes from __examples__/all-validators.ts
 */

// Import to register all creators
import "../ts-tables";

import { createBean, createAllTables } from "../ts-tables";

export = () => {
	describe("Item Bean with @Required, @Set, @Range", () => {
		it("should create Item with all fields", () => {
			const itemData = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };
			const item = createBean<{ id: number; name: string; category: string; stackLimit: number }>(
				"Item",
				itemData,
			);

			expect(item.id).to.equal(1);
			expect(item.name).to.equal("Sword");
			expect(item.category).to.equal("weapon");
			expect(item.stackLimit).to.equal(99);
		});

		it("should handle different category values", () => {
			const itemData = { id: 2, name: "Shield", category: "armor", stackLimit: 50 };
			const item = createBean<{ id: number; name: string; category: string; stackLimit: number }>(
				"Item",
				itemData,
			);

			expect(item.category).to.equal("armor");
		});

		it("should handle boundary range values", () => {
			const itemData = { id: 3, name: "Potion", category: "consumable", stackLimit: 1 };
			const item = createBean<{ id: number; name: string; category: string; stackLimit: number }>(
				"Item",
				itemData,
			);

			expect(item.stackLimit).to.equal(1);
		});
	});

	describe("Skill Bean with @Required and @Range", () => {
		it("should create Skill with all fields", () => {
			const skillData = { id: 1, skillName: "Fireball", cooldown: 10 };
			const skill = createBean<{ id: number; skillName: string; cooldown: number }>(
				"Skill",
				skillData,
			);

			expect(skill.id).to.equal(1);
			expect(skill.skillName).to.equal("Fireball");
			expect(skill.cooldown).to.equal(10);
		});

		it("should handle zero cooldown", () => {
			const skillData = { id: 2, skillName: "Passive", cooldown: 0 };
			const skill = createBean<{ id: number; skillName: string; cooldown: number }>(
				"Skill",
				skillData,
			);

			expect(skill.cooldown).to.equal(0);
		});
	});

	describe("DropItem Bean with @Ref and @Range", () => {
		it("should create DropItem with nested structure", () => {
			const dropData = { itemId: 1, count: 5, probability: 50 };
			const dropItem = createBean<{ itemId: number; count: number; probability: number }>(
				"DropItem",
				dropData,
			);

			expect(dropItem.itemId).to.equal(1);
			expect(dropItem.count).to.equal(5);
			expect(dropItem.probability).to.equal(50);
		});

		it("should handle boundary probability values", () => {
			const dropData = { itemId: 2, count: 1, probability: 100 };
			const dropItem = createBean<{ itemId: number; count: number; probability: number }>(
				"DropItem",
				dropData,
			);

			expect(dropItem.probability).to.equal(100);
		});

		it("should handle zero probability", () => {
			const dropData = { itemId: 3, count: 1, probability: 0 };
			const dropItem = createBean<{ itemId: number; count: number; probability: number }>(
				"DropItem",
				dropData,
			);

			expect(dropItem.probability).to.equal(0);
		});
	});

	describe("Monster Bean with @Ref, @Size, and nested DropItem[]", () => {
		it("should create Monster with nested drops array", () => {
			const monsterData = {
				id: 1,
				name: "Goblin",
				level: 5,
				hp: 100,
				skills: [1, 2],
				drops: [
					{ itemId: 1, count: 1, probability: 50 },
					{ itemId: 2, count: 1, probability: 30 },
				],
			};

			const monster = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.id).to.equal(1);
			expect(monster.name).to.equal("Goblin");
			expect(monster.level).to.equal(5);
			expect(monster.hp).to.equal(100);
			expect(monster.skills.size()).to.equal(2);
			expect(monster.drops.size()).to.equal(2);
		});

		it("should handle skills array correctly", () => {
			const monsterData = {
				id: 2,
				name: "Orc",
				level: 10,
				hp: 200,
				skills: [1, 2, 3],
				drops: [{ itemId: 1, count: 2, probability: 80 }],
			};

			const monster = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.skills[0]).to.equal(1);
			expect(monster.skills[1]).to.equal(2);
			expect(monster.skills[2]).to.equal(3);
		});

		it("should handle single skill", () => {
			const monsterData = {
				id: 3,
				name: "Slime",
				level: 1,
				hp: 20,
				skills: [1],
				drops: [],
			};

			const monster = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(1);
			expect(monster.drops.size()).to.equal(0);
		});

		it("should handle empty drops array", () => {
			const monsterData = {
				id: 4,
				name: "Ghost",
				level: 15,
				hp: 150,
				skills: [1, 2],
				drops: [],
			};

			const monster = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(0);
		});
	});

	describe("Player Bean with multiple @Required fields", () => {
		it("should create Player with required fields", () => {
			const playerData = {
				id: 1,
				name: "Player1",
				avatar: "avatar1.png",
				signature: "Hello World",
			};

			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.id).to.equal(1);
			expect(player.name).to.equal("Player1");
			expect(player.avatar).to.equal("avatar1.png");
			expect(player.signature).to.equal("Hello World");
		});

		it("should handle Player without optional signature", () => {
			const playerData = {
				id: 2,
				name: "Player2",
				avatar: "avatar2.png",
			};

			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.name).to.equal("Player2");
			expect(player.avatar).to.equal("avatar2.png");
		});
	});

	describe("Difficulty Bean with @Range", () => {
		it("should create Difficulty with range validation", () => {
			const difficultyData = {
				id: 1,
				difficultyLevel: 1,
				difficultyName: "Easy",
			};

			const difficulty = createBean<{
				id: number;
				difficultyLevel: number;
				difficultyName: string;
			}>("Difficulty", difficultyData);

			expect(difficulty.difficultyLevel).to.equal(1);
			expect(difficulty.difficultyName).to.equal("Easy");
		});

		it("should handle all valid difficulty levels", () => {
			const levels = [1, 2, 3];
			const names = ["Easy", "Normal", "Hard"];

			for (let i = 0; i < levels.size(); i++) {
				const difficultyData = {
					id: i + 1,
					difficultyLevel: levels[i],
					difficultyName: names[i],
				};

				const difficulty = createBean<{
					id: number;
					difficultyLevel: number;
					difficultyName: string;
				}>("Difficulty", difficultyData);

				expect(difficulty.difficultyLevel).to.equal(levels[i]);
			}
		});
	});

	describe("Team Bean with @Size (fixed and range)", () => {
		it("should create Team with fixed size members array", () => {
			const teamData = {
				id: 1,
				members: [101, 102, 103],
				substitutes: [201],
			};

			const team = createBean<{
				id: number;
				members: number[];
				substitutes: number[];
			}>("Team", teamData);

			expect(team.members.size()).to.equal(3);
			expect(team.members[0]).to.equal(101);
			expect(team.members[1]).to.equal(102);
			expect(team.members[2]).to.equal(103);
		});

		it("should handle substitutes with size range", () => {
			const teamData = {
				id: 2,
				members: [104, 105, 106],
				substitutes: [],
			};

			const team = createBean<{
				id: number;
				members: number[];
				substitutes: number[];
			}>("Team", teamData);

			expect(team.substitutes.size()).to.equal(0);
		});

		it("should handle max substitutes", () => {
			const teamData = {
				id: 3,
				members: [107, 108, 109],
				substitutes: [201, 202],
			};

			const team = createBean<{
				id: number;
				members: number[];
				substitutes: number[];
			}>("Team", teamData);

			expect(team.substitutes.size()).to.equal(2);
		});
	});

	describe("createAllTables with Validator Beans", () => {
		it("should load all tables with validator data", () => {
			const mockLoader = (file: string): unknown => {
				if (file === "item") {
					return {
						"1": {
							id: 1,
							name: "TestItem",
							category: "weapon",
							stackLimit: 50,
						},
					};
				}
				// Return empty data for other tables
				return {};
			};

			const tables = createAllTables(mockLoader);

			expect(tables.ItemTable).to.be.ok();
			expect(tables.ItemTable.get(1)).to.be.ok();
		});
	});
};
