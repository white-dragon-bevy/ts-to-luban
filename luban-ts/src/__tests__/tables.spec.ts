// Test for generated table code
import "../ts-tables"; // This imports and registers all creators

import {
	createAllTables,
	createBean,
	AllTables,
} from "../ts-tables";

// Mock JSON data for testing
const mockGameConfigJson = {
	id: 1,
	maxPlayers: 100,
	gameVersion: "1.0.0",
	debugMode: false,
};

const mockServerSettingsJson = {
	id: 1,
	serverName: "TestServer",
	tickRate: 60,
};

const mockLeaderboardEntriesJson = [
	{ rank: 1, playerId: 1001, score: 5000 },
	{ rank: 2, playerId: 1002, score: 4500 },
	{ rank: 3, playerId: 1003, score: 4000 },
];

const mockItemJson = {
	"1": { id: 1, name: "Sword", price: 100 },
	"2": { id: 2, name: "Shield", price: 150 },
};

export = () => {
	describe("Registry", () => {
		it("should create beans using registry", () => {
			const gameConfig = createBean<{ id: number }>("GameConfig", mockGameConfigJson);
			expect(gameConfig.id).to.equal(1);
		});

		it("should throw error for unknown bean", () => {
			expect(() => {
				createBean("UnknownBean", {});
			}).to.throw();
		});
	});

	describe("createAllTables", () => {
		let tables: AllTables;

		beforeAll(() => {
			// Create a mock loader function
			const mockLoader = (file: string): unknown => {
				switch (file) {
					case "game-config":
						return mockGameConfigJson;
					case "server-settings":
						return mockServerSettingsJson;
					case "leaderboard-entry":
						return mockLeaderboardEntriesJson;
					case "item":
						return mockItemJson;
					default:
						return {};
				}
			};

			tables = createAllTables(mockLoader);
		});

		describe("One/Singleton Tables", () => {
			it("should load GameConfig (one mode)", () => {
				expect(tables.GameConfigTable).to.be.ok();
				expect(tables.GameConfigTable.data).to.be.ok();
				expect(tables.GameConfigTable.data.id).to.equal(1);
				expect(tables.GameConfigTable.data.maxPlayers).to.equal(100);
				expect(tables.GameConfigTable.data.gameVersion).to.equal("1.0.0");
			});

			it("should load ServerSettings (singleton mode)", () => {
				expect(tables.ServerSettingsTable).to.be.ok();
				expect(tables.ServerSettingsTable.data).to.be.ok();
				expect(tables.ServerSettingsTable.data.serverName).to.equal("TestServer");
				expect(tables.ServerSettingsTable.data.tickRate).to.equal(60);
			});
		});

		describe("List Tables", () => {
			it("should load LeaderboardEntry (list mode)", () => {
				expect(tables.LeaderboardEntryTable).to.be.ok();
				expect(tables.LeaderboardEntryTable.dataList).to.be.ok();
				expect(tables.LeaderboardEntryTable.dataList.size()).to.equal(3);
			});

			it("should preserve order in list", () => {
				const first = tables.LeaderboardEntryTable.dataList[0];
				expect(first.rank).to.equal(1);
				expect(first.score).to.equal(5000);
			});
		});

		describe("Map Tables", () => {
			it("should load Item (map mode)", () => {
				expect(tables.ItemTable).to.be.ok();
				expect(tables.ItemTable.dataMap).to.be.ok();
				expect(tables.ItemTable.dataList).to.be.ok();
			});

			it("should provide get() for map tables", () => {
				const item = tables.ItemTable.get(1);
				expect(item).to.be.ok();
				expect(item!.name).to.equal("Sword");
			});

			it("should return undefined for non-existent key", () => {
				const item = tables.ItemTable.get(999);
				expect(item).to.equal(undefined);
			});
		});
	});
};
