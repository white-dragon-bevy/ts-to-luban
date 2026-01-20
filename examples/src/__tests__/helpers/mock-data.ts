/**
 * Mock data helpers for ts-luban tests
 * Provides reusable mock data and loader functions
 */

/**
 * Creates a mock loader function for testing
 * @param dataMap Map of file names to mock data
 * @returns Loader function compatible with createAllTables
 */
export function createMockLoader(dataMap: Record<string, unknown>): (file: string) => unknown {
	return (file: string): unknown => {
		return dataMap[file] ?? {};
	};
}

// ============================================================================
// Item Table Mock Data
// ============================================================================

export const mockItemData = {
	"1": { id: 1, name: "Sword", category: "weapon", stackLimit: 99 },
	"2": { id: 2, name: "Shield", category: "armor", stackLimit: 50 },
	"3": { id: 3, name: "Health Potion", category: "consumable", stackLimit: 999 },
};

export const mockItemSingle = { id: 1, name: "Sword", category: "weapon", stackLimit: 99 };

// ============================================================================
// Skill Table Mock Data
// ============================================================================

export const mockSkillData = {
	"1": { id: 1, skillName: "Fireball", cooldown: 10 },
	"2": { id: 2, skillName: "Ice Shard", cooldown: 5 },
	"3": { id: 3, skillName: "Heal", cooldown: 30 },
};

// ============================================================================
// Monster Table Mock Data
// ============================================================================

export const mockMonsterData = {
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
	"2": {
		id: 2,
		name: "Orc",
		level: 10,
		hp: 200,
		skills: [1, 2, 3],
		drops: [{ itemId: 1, count: 2, probability: 80 }],
	},
};

export const mockMonsterSingle = {
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

// ============================================================================
// Player Table Mock Data
// ============================================================================

export const mockPlayerData = {
	"1": { id: 1, name: "Player1", avatar: "avatar1.png", signature: "Hello" },
	"2": { id: 2, name: "Player2", avatar: "avatar2.png" }, // No signature (optional)
};

// ============================================================================
// Difficulty Table Mock Data
// ============================================================================

export const mockDifficultyData = {
	"1": { id: 1, difficultyLevel: 1, difficultyName: "Easy" },
	"2": { id: 2, difficultyLevel: 2, difficultyName: "Normal" },
	"3": { id: 3, difficultyLevel: 3, difficultyName: "Hard" },
};

// ============================================================================
// Team Table Mock Data
// ============================================================================

export const mockTeamData = {
	"1": { id: 1, members: [101, 102, 103], substitutes: [201] },
	"2": { id: 2, members: [104, 105, 106], substitutes: [] },
};

// ============================================================================
// Weapon Table Mock Data
// ============================================================================

export const mockWeaponData = {
	"1": { id: 1, name: "Iron Sword", damage: 15, attackSpeed: 1.2 },
	"2": { id: 2, name: "Steel Sword", damage: 25, attackSpeed: 1.0 },
};

// ============================================================================
// Armor Table Mock Data
// ============================================================================

export const mockArmorData = {
	"1": { id: 1, name: "Iron Armor", defense: 50 },
	"2": { id: 2, name: "Steel Armor", defense: 100 },
};

// ============================================================================
// Leaderboard Entry Mock Data (List Mode)
// ============================================================================

export const mockLeaderboardData = [
	{ rank: 1, playerId: 1001, score: 5000 },
	{ rank: 2, playerId: 1002, score: 4500 },
	{ rank: 3, playerId: 1003, score: 4000 },
	{ rank: 4, playerId: 1004, score: 3500 },
	{ rank: 5, playerId: 1005, score: 3000 },
];

// ============================================================================
// Game Config Mock Data (One Mode)
// ============================================================================

export const mockGameConfigData = {
	id: 1,
	maxPlayers: 100,
	gameVersion: "1.0.0",
	debugMode: false,
};

// ============================================================================
// Server Settings Mock Data (Singleton Mode)
// ============================================================================

export const mockServerSettingsData = {
	id: 1,
	serverName: "TestServer",
	tickRate: 60,
};

// ============================================================================
// Inheritance Mock Data
// ============================================================================

export const mockBaseUnitData = { id: 1, name: "BaseUnit1" };

export const mockCharacterUnitData = { id: 1, name: "Character1", level: 10, hp: 100 };

export const mockPlayerUnitData = {
	id: 1,
	name: "Player1",
	level: 10,
	hp: 100,
	experience: 500,
	accountId: "acc123",
};

export const mockNPCUnitData = { id: 1, name: "NPC1", level: 5, hp: 50, dialogue: ["Hello", "World"] };

export const mockStandaloneUnitData = { data: "standalone data" };

// ============================================================================
// Combined Data Map for Common Tests
// ============================================================================

export const commonDataMap: Record<string, unknown> = {
	item: mockItemData,
	skill: mockSkillData,
	monster: mockMonsterData,
	player: mockPlayerData,
	difficulty: mockDifficultyData,
	team: mockTeamData,
	weapon: mockWeaponData,
	armor: mockArmorData,
	"leaderboard-entry": mockLeaderboardData,
	"game-config": mockGameConfigData,
	"server-settings": mockServerSettingsData,
};
