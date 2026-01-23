import { CharacterConfig } from "../../__examples__/constructor-type";
import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { Weapon, Armor } from "../../__examples__/items";

export interface AllTables {
    ArmorTable: Map<number, Armor>;
    CharacterConfigTable: Map<number, CharacterConfig>;
    GameConfigTable: GameConfig;
    LeaderboardEntryTable: LeaderboardEntry[];
    ServerSettingsTable: ServerSettings;
    WeaponTable: Map<number, Weapon>;
}