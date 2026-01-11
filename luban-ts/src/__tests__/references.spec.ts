/**
 * Cross-Table Reference Tests
 *
 * Tests for @Ref decorator behavior in generated code:
 * - EquipmentSet with @Ref(Weapon) and @Ref(Armor)
 * - DropItem with @Ref(Item)
 * - Monster with @Ref(Skill)[]
 */

// Import to register all creators
import "../ts-tables";

import { createBean, createAllTables } from "../ts-tables";

export = () => {
	describe("DropItem with @Ref(Item)", () => {
		it("should create bean with reference field", () => {
			const dropData = { itemId: 1, count: 5, probability: 50 };
			const dropItem = createBean<{ itemId: number; count: number; probability: number }>(
				"DropItem",
				dropData,
			);

			expect(dropItem.itemId).to.equal(1);
			expect(dropItem.count).to.equal(5);
			expect(dropItem.probability).to.equal(50);
		});

		it("should handle reference to different item IDs", () => {
			const itemIds = [1, 2, 3, 99, 999];

			for (const itemId of itemIds) {
				const dropData = { itemId, count: 1, probability: 100 };
				const dropItem = createBean<{ itemId: number }>("DropItem", dropData);

				expect(dropItem.itemId).to.equal(itemId);
			}
		});

		it("should handle zero reference ID", () => {
			const dropData = { itemId: 0, count: 1, probability: 50 };
			const dropItem = createBean<{ itemId: number }>("DropItem", dropData);

			expect(dropItem.itemId).to.equal(0);
		});

		it("should handle boundary probability values with reference", () => {
			const dropData = { itemId: 1, count: 1, probability: 0 };
			const dropItem1 = createBean<{ probability: number }>("DropItem", dropData);
			expect(dropItem1.probability).to.equal(0);

			const dropData2 = { itemId: 1, count: 1, probability: 100 };
			const dropItem2 = createBean<{ probability: number }>("DropItem", dropData2);
			expect(dropItem2.probability).to.equal(100);
		});
	});

	describe("EquipmentSet with multiple @Ref fields", () => {
		it("should create bean with @Ref(Weapon) and @Ref(Armor)", () => {
			const equipmentData = { weaponId: 1, armorId: 2 };
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>(
				"EquipmentSet",
				equipmentData,
			);

			expect(equipmentSet.weaponId).to.equal(1);
			expect(equipmentSet.armorId).to.equal(2);
		});

		it("should handle both references independently", () => {
			const equipmentData = { weaponId: 10, armorId: 20 };
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>(
				"EquipmentSet",
				equipmentData,
			);

			expect(equipmentSet.weaponId).to.equal(10);
			expect(equipmentSet.armorId).to.equal(20);

			// Verify independence
			equipmentSet.weaponId = 15;
			expect(equipmentSet.weaponId).to.equal(15);
			expect(equipmentSet.armorId).to.equal(20); // unchanged
		});

		it("should handle zero reference IDs", () => {
			const equipmentData = { weaponId: 0, armorId: 0 };
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>(
				"EquipmentSet",
				equipmentData,
			);

			expect(equipmentSet.weaponId).to.equal(0);
			expect(equipmentSet.armorId).to.equal(0);
		});

		it("should handle same ID for both references", () => {
			const equipmentData = { weaponId: 5, armorId: 5 };
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>(
				"EquipmentSet",
				equipmentData,
			);

			expect(equipmentSet.weaponId).to.equal(5);
			expect(equipmentSet.armorId).to.equal(5);
		});
	});

	describe("Monster with @Ref(Skill) array", () => {
		it("should create monster with skill reference array", () => {
			const monsterData = {
				id: 1,
				name: "Goblin",
				level: 5,
				hp: 100,
				skills: [1, 2],
				drops: [],
			};
			const monster = createBean<{
				id: number;
				name: string;
				skills: number[];
			}>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(2);
			expect(monster.skills[0]).to.equal(1);
			expect(monster.skills[1]).to.equal(2);
		});

		it("should handle single skill reference", () => {
			const monsterData = {
				id: 1,
				name: "Slime",
				level: 1,
				hp: 20,
				skills: [1],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(1);
			expect(monster.skills[0]).to.equal(1);
		});

		it("should handle max skills (4)", () => {
			const monsterData = {
				id: 1,
				name: "Boss",
				level: 50,
				hp: 10000,
				skills: [1, 2, 3, 4],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(4);
			expect(monster.skills[0]).to.equal(1);
			expect(monster.skills[3]).to.equal(4);
		});

		it("should handle empty skill array", () => {
			const monsterData = {
				id: 1,
				name: "WeakMonster",
				level: 1,
				hp: 10,
				skills: [],
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(0);
		});

		it("should handle various skill IDs", () => {
			const skillIds = [1, 10, 100, 999];
			const monsterData = {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: skillIds,
				drops: [],
			};
			const monster = createBean<{ skills: number[] }>("Monster", monsterData);

			expect(monster.skills.size()).to.equal(4);
			expect(monster.skills[0]).to.equal(1);
			expect(monster.skills[1]).to.equal(10);
			expect(monster.skills[2]).to.equal(100);
			expect(monster.skills[3]).to.equal(999);
		});
	});

	describe("Monster with nested DropItem[] containing @Ref", () => {
		it("should create monster with drops containing item references", () => {
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
				name: string;
				drops: unknown[];
			}>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(3);

			const firstDrop = monster.drops[0] as { itemId: number; count: number; probability: number };
			expect(firstDrop.itemId).to.equal(1);
			expect(firstDrop.count).to.equal(5);
			expect(firstDrop.probability).to.equal(100);

			const secondDrop = monster.drops[1] as { itemId: number; count: number; probability: number };
			expect(secondDrop.itemId).to.equal(2);

			const thirdDrop = monster.drops[2] as { itemId: number; count: number; probability: number };
			expect(thirdDrop.itemId).to.equal(3);
		});

		it("should handle empty drops array", () => {
			const monsterData = {
				id: 1,
				name: "PoorMonster",
				level: 1,
				hp: 10,
				skills: [],
				drops: [],
			};
			const monster = createBean<{ drops: unknown[] }>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(0);
		});

		it("should handle single drop", () => {
			const monsterData = {
				id: 1,
				name: "LuckyMonster",
				level: 10,
				hp: 100,
				skills: [1],
				drops: [{ itemId: 99, count: 1, probability: 100 }],
			};
			const monster = createBean<{ drops: unknown[] }>("Monster", monsterData);

			expect(monster.drops.size()).to.equal(1);
			const drop = monster.drops[0] as { itemId: number };
			expect(drop.itemId).to.equal(99);
		});
	});

	describe("Cross-Table Integration", () => {
		it("should load referenced tables independently", () => {
			const mockLoader = (file: string): unknown => {
				switch (file) {
					case "item":
						return {
							"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 99 },
							"2": { id: 2, name: "Shield", category: "armor", stackLimit: 50 },
						};
					case "monster":
						return {
							"1": {
								id: 1,
								name: "Goblin",
								level: 5,
								hp: 100,
								skills: [1, 2],
								drops: [
									{ itemId: 1, count: 1, probability: 50 },
									{ itemId: 2, count: 1, probability: 30 },
								],
							},
						};
					case "skill":
						return {
							"1": { id: 1, skillName: "Attack", cooldown: 0 },
							"2": { id: 2, skillName: "Defend", cooldown: 5 },
						};
					default:
						return {};
				}
			};

			const tables = createAllTables(mockLoader);

			// Item table should be loaded
			expect(tables.ItemTable.get(1)).to.be.ok();
			expect(tables.ItemTable.get(1)!.name).to.equal("Sword");

			// Monster table should be loaded with references
			expect(tables.MonsterTable.get(1)).to.be.ok();
			const monster = tables.MonsterTable.get(1)!;
			expect(monster.name).to.equal("Goblin");
			expect(monster.skills.size()).to.equal(2);
			expect((monster.drops[0] as { itemId: number }).itemId).to.equal(1);

			// Skill table should be loaded
			expect(tables.SkillTable.get(1)).to.be.ok();
		});

		it("should handle multiple reference chains", () => {
			const mockLoader = (file: string): unknown => {
				switch (file) {
					case "weapon":
						return { "1": { id: 1, name: "Sword", damage: 10, attackSpeed: 1.0 } };
					case "armor":
						return { "1": { id: 1, name: "Plate", defense: 50 } };
					default:
						return {};
				}
			};

			const tables = createAllTables(mockLoader);

			// Both Weapon and Armor tables should be loaded
			expect(tables.WeaponTable.get(1)).to.be.ok();
			expect(tables.ArmorTable.get(1)).to.be.ok();

			// EquipmentSet references should work
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>("EquipmentSet", {
				weaponId: 1,
				armorId: 1,
			});
			expect(equipmentSet.weaponId).to.equal(1);
			expect(equipmentSet.armorId).to.equal(1);
		});
	});

	describe("Reference Field Types", () => {
		it("should handle numeric reference IDs", () => {
			const dropItem = createBean<{ itemId: number }>("DropItem", {
				itemId: 123,
				count: 1,
				probability: 50,
			});
			expect(typeOf(dropItem.itemId)).to.equal("number");
		});

		it("should handle reference arrays", () => {
			const monster = createBean<{ skills: unknown[] }>("Monster", {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: [1, 2, 3],
				drops: [],
			});
			expect(monster.skills).to.be.a("table");
			expect(monster.skills.size()).to.equal(3);
		});

		it("should handle nested reference objects", () => {
			const monster = createBean<{ drops: unknown[] }>("Monster", {
				id: 1,
				name: "Test",
				level: 1,
				hp: 10,
				skills: [],
				drops: [{ itemId: 1, count: 1, probability: 100 }],
			});

			const drop = monster.drops[0] as { itemId: number; count: number; probability: number };
			expect(drop.itemId).to.equal(1);
			expect(drop.count).to.equal(1);
			expect(drop.probability).to.equal(100);
		});
	});

	describe("Multiple Reference Types in Same Bean", () => {
		it("should handle EquipmentSet with two different reference types", () => {
			const equipmentSet = createBean<{ weaponId: number; armorId: number }>("EquipmentSet", {
				weaponId: 1,
				armorId: 2,
			});

			// Both should be number types (reference IDs)
			expect(typeOf(equipmentSet.weaponId)).to.equal("number");
			expect(typeOf(equipmentSet.armorId)).to.equal("number");

			// Should be independent values
			expect(equipmentSet.weaponId).to.never.equal(equipmentSet.armorId);
		});
	});
};
