/**
 * Table Mode Tests
 *
 * Tests for all four Luban table modes:
 * - map: Key-value lookup with dataMap and get()
 * - list: Ordered array with dataList
 * - one: Single record table
 * - singleton: Single instance table
 */

// Import to register all creators
import "../ts-tables";

import { createAllTables, AllTables } from "../ts-tables";
import { createMockLoader } from "./helpers/mock-data";

export = () => {
	describe("Map Mode Tables", () => {
		let tables: AllTables;

		beforeAll(() => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 99 },
					"2": { id: 2, name: "Shield", category: "armor", stackLimit: 50 },
					"3": { id: 3, name: "Potion", category: "consumable", stackLimit: 999 },
				},
			});
			tables = createAllTables(mockLoader);
		});

		it("should load map mode table with dataMap and dataList", () => {
			expect(tables.ItemTable.dataMap).to.be.ok();
			expect(tables.ItemTable.dataList).to.be.ok();
		});

		it("should have correct dataMap size", () => {
			expect(tables.ItemTable.dataMap.size()).to.equal(3);
		});

		it("should have correct dataList size", () => {
			expect(tables.ItemTable.dataList.size()).to.equal(3);
		});

		it("should get items by key", () => {
			const item1 = tables.ItemTable.get(1);
			expect(item1).to.be.ok();
			expect(item1!.name).to.equal("Sword");

			const item2 = tables.ItemTable.get(2);
			expect(item2).to.be.ok();
			expect(item2!.name).to.equal("Shield");

			const item3 = tables.ItemTable.get(3);
			expect(item3).to.be.ok();
			expect(item3!.name).to.equal("Potion");
		});

		it("should return undefined for non-existent keys", () => {
			const item = tables.ItemTable.get(999);
			expect(item).to.equal(undefined);
		});

		it("should return undefined for zero key", () => {
			const item = tables.ItemTable.get(0);
			expect(item).to.equal(undefined);
		});

		it("should return undefined for negative key", () => {
			const item = tables.ItemTable.get(-1);
			expect(item).to.equal(undefined);
		});

		it("should iterate all items via dataList", () => {
			const items = tables.ItemTable.dataList;
			let count = 0;
			for (const _item of items) {
				count++;
			}
			expect(count).to.equal(3);
		});

		it("should access dataList by index", () => {
			const first = tables.ItemTable.dataList[0];
			expect(first.name).to.be.ok();

			const second = tables.ItemTable.dataList[1];
			expect(second.name).to.be.ok();
		});

		it("should handle empty map", () => {
			const mockLoader = createMockLoader({
				item: {},
			});
			const emptyTables = createAllTables(mockLoader);

			expect(emptyTables.ItemTable.dataMap.size()).to.equal(0);
			expect(emptyTables.ItemTable.dataList.size()).to.equal(0);
			expect(emptyTables.ItemTable.get(1)).to.equal(undefined);
		});
	});

	describe("List Mode Tables", () => {
		let tables: AllTables;

		beforeAll(() => {
			const mockLoader = createMockLoader({
				"leaderboard-entry": [
					{ rank: 1, playerId: 1001, score: 5000 },
					{ rank: 2, playerId: 1002, score: 4500 },
					{ rank: 3, playerId: 1003, score: 4000 },
					{ rank: 4, playerId: 1004, score: 3500 },
					{ rank: 5, playerId: 1005, score: 3000 },
				],
			});
			tables = createAllTables(mockLoader);
		});

		it("should load list mode table preserving order", () => {
			expect(tables.LeaderboardEntryTable.dataList).to.be.ok();
			expect(tables.LeaderboardEntryTable.dataList.size()).to.equal(5);
		});

		it("should access items by index in correct order", () => {
			const first = tables.LeaderboardEntryTable.dataList[0];
			expect(first.rank).to.equal(1);
			expect(first.score).to.equal(5000);

			const third = tables.LeaderboardEntryTable.dataList[2];
			expect(third.rank).to.equal(3);
			expect(third.score).to.equal(4000);

			const last = tables.LeaderboardEntryTable.dataList[4];
			expect(last.rank).to.equal(5);
			expect(last.score).to.equal(3000);
		});

		it("should have correct dataList size", () => {
			expect(tables.LeaderboardEntryTable.dataList.size()).to.equal(5);
		});

		it("should iterate items in order", () => {
			const list = tables.LeaderboardEntryTable.dataList;
			let expectedRank = 1;
			for (const entry of list) {
				expect(entry.rank).to.equal(expectedRank);
				expectedRank++;
			}
		});

		it("should handle empty list", () => {
			const mockLoader = createMockLoader({
				"leaderboard-entry": [],
			});
			const emptyTables = createAllTables(mockLoader);

			expect(emptyTables.LeaderboardEntryTable.dataList.size()).to.equal(0);
		});

		it("should handle single item list", () => {
			const mockLoader = createMockLoader({
				"leaderboard-entry": [{ rank: 1, playerId: 1001, score: 5000 }],
			});
			const singleTables = createAllTables(mockLoader);

			expect(singleTables.LeaderboardEntryTable.dataList.size()).to.equal(1);
			expect(singleTables.LeaderboardEntryTable.dataList[0].rank).to.equal(1);
		});
	});

	describe("One Mode Tables", () => {
		let tables: AllTables;

		beforeAll(() => {
			const mockLoader = createMockLoader({
				"game-config": {
					id: 1,
					maxPlayers: 100,
					gameVersion: "1.0.0",
					debugMode: false,
				},
			});
			tables = createAllTables(mockLoader);
		});

		it("should load single record table", () => {
			expect(tables.GameConfigTable.data).to.be.ok();
		});

		it("should access data property", () => {
			expect(tables.GameConfigTable.data.id).to.equal(1);
			expect(tables.GameConfigTable.data.maxPlayers).to.equal(100);
			expect(tables.GameConfigTable.data.gameVersion).to.equal("1.0.0");
			expect(tables.GameConfigTable.data.debugMode).to.equal(false);
		});

		it("should have all expected fields", () => {
			const config = tables.GameConfigTable.data;
			expect(config.id).to.be.a("number");
			expect(config.maxPlayers).to.be.a("number");
			expect(config.gameVersion).to.be.a("string");
			expect(config.debugMode).to.be.a("boolean");
		});

		it("should handle numeric fields correctly", () => {
			expect(tables.GameConfigTable.data.maxPlayers).to.equal(100);
		});

		it("should handle string fields correctly", () => {
			expect(tables.GameConfigTable.data.gameVersion).to.equal("1.0.0");
		});

		it("should handle boolean fields correctly", () => {
			expect(tables.GameConfigTable.data.debugMode).to.equal(false);
		});
	});

	describe("Singleton Mode Tables", () => {
		let tables: AllTables;

		beforeAll(() => {
			const mockLoader = createMockLoader({
				"server-settings": {
					id: 1,
					serverName: "TestServer",
					tickRate: 60,
				},
			});
			tables = createAllTables(mockLoader);
		});

		it("should load singleton table", () => {
			expect(tables.ServerSettingsTable.data).to.be.ok();
		});

		it("should access data property", () => {
			expect(tables.ServerSettingsTable.data.id).to.equal(1);
			expect(tables.ServerSettingsTable.data.serverName).to.equal("TestServer");
			expect(tables.ServerSettingsTable.data.tickRate).to.equal(60);
		});

		it("should have all expected fields", () => {
			const settings = tables.ServerSettingsTable.data;
			expect(settings.id).to.be.ok();
			expect(settings.serverName).to.be.ok();
			expect(settings.tickRate).to.be.ok();
		});

		it("should return same instance across multiple loads", () => {
			const data1 = tables.ServerSettingsTable.data;
			const data2 = tables.ServerSettingsTable.data;

			// Same reference
			expect(data1).to.equal(data2);
		});
	});

	describe("Multiple Tables of Different Modes", () => {
		it("should load map and list tables together", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "Item1", category: "weapon", stackLimit: 99 },
				},
				"leaderboard-entry": [{ rank: 1, playerId: 1001, score: 5000 }],
			});

			const tables = createAllTables(mockLoader);

			expect(tables.ItemTable.get(1)).to.be.ok();
			expect(tables.LeaderboardEntryTable.dataList[0]).to.be.ok();
		});

		it("should load all four mode types together", () => {
			const mockLoader = createMockLoader({
				item: { "1": { id: 1, name: "Item1", category: "weapon", stackLimit: 99 } },
				"leaderboard-entry": [{ rank: 1, playerId: 1001, score: 5000 }],
				"game-config": { id: 1, maxPlayers: 100, gameVersion: "1.0.0", debugMode: false },
				"server-settings": { id: 1, serverName: "Server", tickRate: 60 },
			});

			const tables = createAllTables(mockLoader);

			// Map mode
			expect(tables.ItemTable.get(1)).to.be.ok();

			// List mode
			expect(tables.LeaderboardEntryTable.dataList[0]).to.be.ok();

			// One mode
			expect(tables.GameConfigTable.data).to.be.ok();

			// Singleton mode
			expect(tables.ServerSettingsTable.data).to.be.ok();
		});
	});

	describe("Table Data Access Patterns", () => {
		it("should support forEach iteration on map dataList", () => {
			const mockLoader = createMockLoader({
				item: {
					"1": { id: 1, name: "A", category: "weapon", stackLimit: 99 },
					"2": { id: 2, name: "B", category: "armor", stackLimit: 50 },
				},
			});

			const tables = createAllTables(mockLoader);
			let count = 0;
			for (const item of tables.ItemTable.dataList) {
				expect(item.id).to.be.ok();
				expect(item.name).to.be.ok();
				count++;
			}
			expect(count).to.equal(2);
		});

		it("should support direct index access on list", () => {
			const mockLoader = createMockLoader({
				"leaderboard-entry": [
					{ rank: 1, playerId: 1001, score: 5000 },
					{ rank: 2, playerId: 1002, score: 4500 },
				],
			});

			const tables = createAllTables(mockLoader);

			expect(tables.LeaderboardEntryTable.dataList[0].rank).to.equal(1);
			expect(tables.LeaderboardEntryTable.dataList[1].rank).to.equal(2);
		});

		it("should get() returns undefined before and after valid key", () => {
			const mockLoader = createMockLoader({
				item: {
					"2": { id: 2, name: "Middle", category: "weapon", stackLimit: 99 },
				},
			});

			const tables = createAllTables(mockLoader);

			expect(tables.ItemTable.get(1)).to.equal(undefined);
			expect(tables.ItemTable.get(2)).to.be.ok();
			expect(tables.ItemTable.get(3)).to.equal(undefined);
		});
	});
};
