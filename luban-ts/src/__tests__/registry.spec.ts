/**
 * Bean Registry Tests
 *
 * Tests for the core registry functionality:
 * - createBean(): Create bean instances by name
 * - createByType(): Create bean instances by type name
 * - registerCreator(): Register creator functions
 */

// Import to register all creators
import "../ts-tables";

import { createBean, createByType } from "../ts-tables";

export = () => {
	describe("createBean", () => {
		it("should create bean by registered name", () => {
			const itemData = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };
			const item = createBean<{ id: number; name: string }>("Item", itemData);

			expect(item).to.be.ok();
			expect(item.id).to.equal(1);
			expect(item.name).to.equal("Sword");
		});

		it("should create bean with all fields populated", () => {
			const monsterData = {
				id: 1,
				name: "Goblin",
				level: 5,
				hp: 100,
				skills: [1, 2],
				drops: [{ itemId: 1, count: 1, probability: 50 }],
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
			expect(monster.drops.size()).to.equal(1);
		});

		it("should throw error for unknown bean name", () => {
			const unknownData = { id: 1, name: "Unknown" };

			expect(() => {
				createBean("NonExistentBean", unknownData);
			}).to.throw();
		});

		it("should throw error for empty bean name", () => {
			expect(() => {
				createBean("", {});
			}).to.throw();
		});

		it("should handle empty data object", () => {
			const item = createBean<{ id?: number; name?: string }>("Item", {});

			// Should create bean even with empty data
			expect(item).to.be.ok();
		});

		it("should handle nested bean creation", () => {
			const monsterData = {
				id: 1,
				name: "Dragon",
				level: 50,
				hp: 10000,
				skills: [1, 2, 3, 4],
				drops: [
					{ itemId: 1, count: 5, probability: 100 },
					{ itemId: 2, count: 3, probability: 80 },
					{ itemId: 3, count: 1, probability: 50 },
				],
			};

			const monster = createBean<{
				id: number;
				name: string;
				drops: unknown[];
			}>("Monster", monsterData);

			// Nested drops should be created
			expect(monster.drops.size()).to.equal(3);
			const firstDrop = monster.drops[0] as { itemId: number; count: number; probability: number };
			expect(firstDrop.itemId).to.equal(1);
			expect(firstDrop.count).to.equal(5);
		});

		it("should handle bean with optional fields", () => {
			const playerData = {
				id: 1,
				name: "Player1",
				avatar: "avatar.png",
			};

			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.id).to.equal(1);
			expect(player.name).to.equal("Player1");
			expect(player.avatar).to.equal("avatar.png");
		});

		it("should handle bean with array fields", () => {
			const teamData = {
				id: 1,
				members: [101, 102, 103],
				substitutes: [201, 202],
			};

			const team = createBean<{
				id: number;
				members: number[];
				substitutes: number[];
			}>("Team", teamData);

			expect(team.members.size()).to.equal(3);
			expect(team.substitutes.size()).to.equal(2);
			expect(team.members[0]).to.equal(101);
		});

		it("should create Skill bean correctly", () => {
			const skillData = { id: 1, skillName: "Fireball", cooldown: 10 };
			const skill = createBean<{ id: number; skillName: string; cooldown: number }>(
				"Skill",
				skillData,
			);

			expect(skill.id).to.equal(1);
			expect(skill.skillName).to.equal("Fireball");
			expect(skill.cooldown).to.equal(10);
		});

		it("should create Difficulty bean correctly", () => {
			const difficultyData = { id: 1, difficultyLevel: 1, difficultyName: "Easy" };
			const difficulty = createBean<{
				id: number;
				difficultyLevel: number;
				difficultyName: string;
			}>("Difficulty", difficultyData);

			expect(difficulty.difficultyLevel).to.equal(1);
			expect(difficulty.difficultyName).to.equal("Easy");
		});

		it("should create Player bean correctly", () => {
			const playerData = { id: 1, name: "Player1", avatar: "avatar.png", signature: "Hello" };
			const player = createBean<{
				id: number;
				name: string;
				avatar: string;
				signature?: string;
			}>("Player", playerData);

			expect(player.name).to.equal("Player1");
			expect(player.signature).to.equal("Hello");
		});
	});

	describe("createByType", () => {
		it("should create bean by type name", () => {
			const itemData = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };
			const item = createByType<{ id: number; name: string }>("Item", itemData);

			expect(item).to.be.ok();
			expect(item.id).to.equal(1);
			expect(item.name).to.equal("Sword");
		});

		it("should be equivalent to createBean", () => {
			const itemData = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };

			const item1 = createBean<{ id: number; name: string }>("Item", itemData);
			const item2 = createByType<{ id: number; name: string }>("Item", itemData);

			expect(item1.id).to.equal(item2.id);
			expect(item1.name).to.equal(item2.name);
		});

		it("should throw error for unknown type name", () => {
			const unknownData = { id: 1, name: "Unknown" };

			expect(() => {
				createByType("NonExistentType", unknownData);
			}).to.throw();
		});

		it("should create Monster by type name", () => {
			const monsterData = {
				id: 1,
				name: "Goblin",
				level: 5,
				hp: 100,
				skills: [1],
				drops: [],
			};

			const monster = createByType<{
				id: number;
				name: string;
				level: number;
				hp: number;
				skills: number[];
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.name).to.equal("Goblin");
			expect(monster.level).to.equal(5);
		});
	});

	describe("Bean Registration", () => {
		it("should have all expected beans registered", () => {
			// Test that common beans can be created
			const itemData = { id: 1, name: "Test", category: "weapon", stackLimit: 99 };
			const item = createBean("Item", itemData);
			expect(item).to.be.ok();

			const skillData = { id: 1, skillName: "Test", cooldown: 10 };
			const skill = createBean("Skill", skillData);
			expect(skill).to.be.ok();

			const monsterData = {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: [],
				drops: [],
			};
			const monster = createBean("Monster", monsterData);
			expect(monster).to.be.ok();

			const playerData = { id: 1, name: "Test", avatar: "test.png" };
			const player = createBean("Player", playerData);
			expect(player).to.be.ok();
		});

		it("should create beans from different sources", () => {
			// Item from examples module
			const item = createBean("Item", { id: 1, name: "Test", category: "weapon", stackLimit: 99 });
			expect(item).to.be.ok();

			// Weapon from items module
			const weapon = createBean("Weapon", { id: 1, name: "Test", damage: 10, attackSpeed: 1.0 });
			expect(weapon).to.be.ok();

			// Armor from items module
			const armor = createBean("Armor", { id: 1, name: "Test", defense: 50 });
			expect(armor).to.be.ok();

			// BaseUnit from inheritance module
			const baseUnit = createBean("BaseUnit", { id: 1, name: "Test" });
			expect(baseUnit).to.be.ok();

			// CharacterUnit from inheritance module
			const characterUnit = createBean("CharacterUnit", { id: 1, name: "Test", level: 1, hp: 10 });
			expect(characterUnit).to.be.ok();

			// GameConfig from table-modes module
			const gameConfig = createBean("GameConfig", {
				id: 1,
				maxPlayers: 100,
				gameVersion: "1.0.0",
				debugMode: false,
			});
			expect(gameConfig).to.be.ok();
		});

		it("should create beans with inheritance", () => {
			// BaseUnit
			const baseUnit = createBean("BaseUnit", { id: 1, name: "Base" });
			expect(baseUnit).to.be.ok();

			// CharacterUnit extends BaseUnit
			const characterUnit = createBean("CharacterUnit", { id: 1, name: "Character", level: 1, hp: 10 });
			expect(characterUnit).to.be.ok();

			// PlayerUnit extends CharacterUnit
			const playerUnit = createBean("PlayerUnit", {
				id: 1,
				name: "Player",
				level: 1,
				hp: 10,
				experience: 100,
				accountId: "acc123",
			});
			expect(playerUnit).to.be.ok();

			// NPCUnit extends CharacterUnit
			const npcUnit = createBean("NPCUnit", { id: 1, name: "NPC", level: 1, hp: 10, dialogue: [] });
			expect(npcUnit).to.be.ok();

			// StandaloneUnit (no inheritance)
			const standaloneUnit = createBean("StandaloneUnit", { data: "test" });
			expect(standaloneUnit).to.be.ok();
		});
	});

	describe("Multiple Bean Creation", () => {
		it("should create multiple instances of same bean type", () => {
			const itemData1 = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };
			const itemData2 = { id: 2, name: "Shield", category: "armor", stackLimit: 50 };

			const item1 = createBean<{ id: number; name: string }>("Item", itemData1);
			const item2 = createBean<{ id: number; name: string }>("Item", itemData2);

			expect(item1.id).to.equal(1);
			expect(item2.id).to.equal(2);
			expect(item1.name).to.equal("Sword");
			expect(item2.name).to.equal("Shield");

			// Independent instances
			expect(item1).to.never.equal(item2);
		});

		it("should handle rapid bean creation", () => {
			const items: { id: number; name: string }[] = [];
			for (let i = 1; i <= 10; i++) {
				const itemData = { id: i, name: `Item${i}`, category: "weapon", stackLimit: 99 };
				items.push(createBean("Item", itemData) as { id: number; name: string });
			}

			expect(items.size()).to.equal(10);
			for (let i = 0; i < items.size(); i++) {
				const item = items[i] as { id: number; name: string };
				expect(item.id).to.equal(i + 1);
			}
		});
	});

	describe("Type Safety", () => {
		it("should return typed bean with correct type", () => {
			const itemData = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };
			const item = createBean<{ id: number; name: string }>("Item", itemData);

			// Type assertion - should have number id
			const id = item.id;
			expect(typeOf(id)).to.equal("number");

			// Type assertion - should have string name
			const name = item.name;
			expect(typeOf(name)).to.equal("string");
		});
	});
};
