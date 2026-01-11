/**
 * Inheritance Tests
 *
 * Tests for bean inheritance behavior:
 * - BaseUnit → CharacterUnit → PlayerUnit / NPCUnit
 * - StandaloneUnit (no inheritance)
 * - Validators in parent classes
 */

// Import to register all creators
import "../ts-tables";

import { createBean } from "../ts-tables";

export = () => {
	describe("BaseUnit (Base Class)", () => {
		it("should create base unit with all fields", () => {
			const baseUnitData = { id: 1, name: "BaseUnit1" };
			const baseUnit = createBean<{ id: number; name: string }>("BaseUnit", baseUnitData);

			expect(baseUnit.id).to.equal(1);
			expect(baseUnit.name).to.equal("BaseUnit1");
		});

		it("should have id field", () => {
			const baseUnitData = { id: 100, name: "Test" };
			const baseUnit = createBean<{ id: number }>("BaseUnit", baseUnitData);

			expect(baseUnit.id).to.equal(100);
		});

		it("should have name field", () => {
			const baseUnitData = { id: 1, name: "TestName" };
			const baseUnit = createBean<{ name: string }>("BaseUnit", baseUnitData);

			expect(baseUnit.name).to.equal("TestName");
		});
	});

	describe("CharacterUnit (extends BaseUnit)", () => {
		it("should create child class with inherited fields", () => {
			const characterUnitData = { id: 1, name: "Character1", level: 10, hp: 100 };
			const characterUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
			}>("CharacterUnit", characterUnitData);

			// Inherited from BaseUnit
			expect(characterUnit.id).to.equal(1);
			expect(characterUnit.name).to.equal("Character1");

			// Own fields
			expect(characterUnit.level).to.equal(10);
			expect(characterUnit.hp).to.equal(100);
		});

		it("should have level field", () => {
			const characterUnitData = { id: 1, name: "Test", level: 50, hp: 100 };
			const characterUnit = createBean<{ level: number }>("CharacterUnit", characterUnitData);

			expect(characterUnit.level).to.equal(50);
		});

		it("should have hp field", () => {
			const characterUnitData = { id: 1, name: "Test", level: 10, hp: 999 };
			const characterUnit = createBean<{ hp: number }>("CharacterUnit", characterUnitData);

			expect(characterUnit.hp).to.equal(999);
		});

		it("should have all four fields (id, name, level, hp)", () => {
			const characterUnitData = { id: 5, name: "Warrior", level: 25, hp: 500 };
			const characterUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
			}>("CharacterUnit", characterUnitData);

			expect(characterUnit.id).to.equal(5);
			expect(characterUnit.name).to.equal("Warrior");
			expect(characterUnit.level).to.equal(25);
			expect(characterUnit.hp).to.equal(500);
		});
	});

	describe("PlayerUnit (extends CharacterUnit extends BaseUnit)", () => {
		it("should create grandchild class with all ancestors' fields", () => {
			const playerUnitData = {
				id: 1,
				name: "Player1",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "acc123",
			};
			const playerUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				experience: number;
				accountId: string;
			}>("PlayerUnit", playerUnitData);

			// Inherited from BaseUnit
			expect(playerUnit.id).to.equal(1);
			expect(playerUnit.name).to.equal("Player1");

			// Inherited from CharacterUnit
			expect(playerUnit.level).to.equal(10);
			expect(playerUnit.hp).to.equal(100);

			// Own fields
			expect(playerUnit.experience).to.equal(500);
			expect(playerUnit.accountId).to.equal("acc123");
		});

		it("should have experience field", () => {
			const playerUnitData = {
				id: 1,
				name: "Player1",
				level: 10,
				hp: 100,
				experience: 9999,
				accountId: "acc123",
			};
			const playerUnit = createBean<{ experience: number }>("PlayerUnit", playerUnitData);

			expect(playerUnit.experience).to.equal(9999);
		});

		it("should have accountId field", () => {
			const playerUnitData = {
				id: 1,
				name: "Player1",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "test_account",
			};
			const playerUnit = createBean<{ accountId: string }>("PlayerUnit", playerUnitData);

			expect(playerUnit.accountId).to.equal("test_account");
		});

		it("should have all six fields", () => {
			const playerUnitData = {
				id: 100,
				name: "Hero",
				level: 99,
				hp: 9999,
				experience: 10000,
				accountId: "hero123",
			};
			const playerUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				experience: number;
				accountId: string;
			}>("PlayerUnit", playerUnitData);

			expect(playerUnit.id).to.equal(100);
			expect(playerUnit.name).to.equal("Hero");
			expect(playerUnit.level).to.equal(99);
			expect(playerUnit.hp).to.equal(9999);
			expect(playerUnit.experience).to.equal(10000);
			expect(playerUnit.accountId).to.equal("hero123");
		});
	});

	describe("NPCUnit (extends CharacterUnit extends BaseUnit)", () => {
		it("should create with all ancestors' fields", () => {
			const npcUnitData = {
				id: 1,
				name: "NPC1",
				level: 5,
				hp: 50,
				dialogue: ["Hello", "World"],
			};
			const npcUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				dialogue: string[];
			}>("NPCUnit", npcUnitData);

			// Inherited from BaseUnit
			expect(npcUnit.id).to.equal(1);
			expect(npcUnit.name).to.equal("NPC1");

			// Inherited from CharacterUnit
			expect(npcUnit.level).to.equal(5);
			expect(npcUnit.hp).to.equal(50);

			// Own field
			expect(npcUnit.dialogue.size()).to.equal(2);
		});

		it("should have dialogue field as array", () => {
			const npcUnitData = {
				id: 1,
				name: "NPC1",
				level: 5,
				hp: 50,
				dialogue: ["Greeting", "Quest", "Goodbye"],
			};
			const npcUnit = createBean<{ dialogue: string[] }>("NPCUnit", npcUnitData);

			expect(npcUnit.dialogue.size()).to.equal(3);
			expect(npcUnit.dialogue[0]).to.equal("Greeting");
			expect(npcUnit.dialogue[1]).to.equal("Quest");
			expect(npcUnit.dialogue[2]).to.equal("Goodbye");
		});

		it("should handle empty dialogue array", () => {
			const npcUnitData = { id: 1, name: "NPC1", level: 5, hp: 50, dialogue: [] };
			const npcUnit = createBean<{ dialogue: string[] }>("NPCUnit", npcUnitData);

			expect(npcUnit.dialogue.size()).to.equal(0);
		});

		it("should have all five fields", () => {
			const npcUnitData = {
				id: 50,
				name: "Merchant",
				level: 20,
				hp: 200,
				dialogue: ["Buy something!", "Thanks!"],
			};
			const npcUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				dialogue: string[];
			}>("NPCUnit", npcUnitData);

			expect(npcUnit.id).to.equal(50);
			expect(npcUnit.name).to.equal("Merchant");
			expect(npcUnit.level).to.equal(20);
			expect(npcUnit.hp).to.equal(200);
			expect(npcUnit.dialogue.size()).to.equal(2);
		});
	});

	describe("StandaloneUnit (no inheritance)", () => {
		it("should handle standalone class without inheritance", () => {
			const standaloneUnitData = { data: "standalone data" };
			const standaloneUnit = createBean<{ data: string }>("StandaloneUnit", standaloneUnitData);

			expect(standaloneUnit.data).to.equal("standalone data");
		});

		it("should only have its own fields", () => {
			const standaloneUnitData = { data: "test data" };
			const standaloneUnit = createBean<{ data: string }>("StandaloneUnit", standaloneUnitData);

			expect(standaloneUnit.data).to.equal("test data");
			// Should not have id, name, level, hp, etc.
		});

		it("should handle different data values", () => {
			const testData = ["data1", "data2", "data3"];

			for (const data of testData) {
				const unit = createBean<{ data: string }>("StandaloneUnit", { data });
				expect(unit.data).to.equal(data);
			}
		});
	});

	describe("Inheritance Chain Comparison", () => {
		it("should create all inheritance levels correctly", () => {
			// BaseUnit - 2 fields
			const baseUnit = createBean<{ id: number; name: string }>("BaseUnit", {
				id: 1,
				name: "Base",
			});
			expect(baseUnit.id).to.be.ok();
			expect(baseUnit.name).to.be.ok();

			// CharacterUnit - 4 fields
			const characterUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
			}>("CharacterUnit", {
				id: 1,
				name: "Character",
				level: 10,
				hp: 100,
			});
			expect(characterUnit.id).to.be.ok();
			expect(characterUnit.name).to.be.ok();
			expect(characterUnit.level).to.be.ok();
			expect(characterUnit.hp).to.be.ok();

			// PlayerUnit - 6 fields
			const playerUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
				experience: number;
				accountId: string;
			}>("PlayerUnit", {
				id: 1,
				name: "Player",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "acc123",
			});
			expect(playerUnit.id).to.be.ok();
			expect(playerUnit.name).to.be.ok();
			expect(playerUnit.level).to.be.ok();
			expect(playerUnit.hp).to.be.ok();
			expect(playerUnit.experience).to.be.ok();
			expect(playerUnit.accountId).to.be.ok();
		});

		it("should maintain field values across inheritance chain", () => {
			const sharedData = {
				id: 1,
				name: "TestUnit",
				level: 25,
				hp: 250,
			};

			const characterUnit = createBean<{
				id: number;
				name: string;
				level: number;
				hp: number;
			}>("CharacterUnit", sharedData);

			expect(characterUnit.id).to.equal(1);
			expect(characterUnit.name).to.equal("TestUnit");
			expect(characterUnit.level).to.equal(25);
			expect(characterUnit.hp).to.equal(250);
		});
	});

	describe("Validators in Parent Classes", () => {
		it("should handle @Required in parent class", () => {
			// BaseUnit has @Required on name field
			const baseUnit = createBean<{ name: string }>("BaseUnit", { id: 1, name: "Test" });
			expect(baseUnit.name).to.equal("Test");
		});

		it("should handle validators in child classes", () => {
			// PlayerUnit has @Required on accountId (inherited from CharacterUnit)
			const playerUnit = createBean<{ accountId: string }>("PlayerUnit", {
				id: 1,
				name: "Player",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "test123",
			});
			expect(playerUnit.accountId).to.equal("test123");
		});
	});

	describe("Multiple Instances of Inherited Classes", () => {
		it("should create multiple instances of same class", () => {
			const player1 = createBean<{
				id: number;
				accountId: string;
			}>("PlayerUnit", {
				id: 1,
				name: "Player1",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "acc1",
			});

			const player2 = createBean<{
				id: number;
				accountId: string;
			}>("PlayerUnit", {
				id: 2,
				name: "Player2",
				level: 20,
				hp: 200,
				experience: 1000,
				accountId: "acc2",
			});

			expect(player1.id).to.equal(1);
			expect(player2.id).to.equal(2);
			expect(player1.accountId).to.equal("acc1");
			expect(player2.accountId).to.equal("acc2");
		});

		it("should create instances from different inheritance levels", () => {
			const base = createBean("BaseUnit", { id: 1, name: "Base" });
			const character = createBean("CharacterUnit", { id: 1, name: "Char", level: 10, hp: 100 });
			const player = createBean("PlayerUnit", {
				id: 1,
				name: "Player",
				level: 10,
				hp: 100,
				experience: 500,
				accountId: "acc123",
			});
			const npc = createBean("NPCUnit", { id: 1, name: "NPC", level: 5, hp: 50, dialogue: [] });

			expect(base).to.be.ok();
			expect(character).to.be.ok();
			expect(player).to.be.ok();
			expect(npc).to.be.ok();
		});
	});
};
