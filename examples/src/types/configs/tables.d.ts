import { Weapon, Armor } from "../../__examples__/items";
import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { CharacterConfig } from "../../__examples__/constructor-type";
import { Item, Skill, Monster, Player, Difficulty, Team } from "../../__examples__/all-validators";
import { WeaponConfig, ArmorConfig } from "../../__examples__/virtual-fields";

export interface AllTables {
    ArmorConfigTable: Map<number, ArmorConfig>;
    ArmorTable: Map<number, Armor>;
    CharacterConfigTable: Map<number, CharacterConfig>;
    DifficultyTable: Map<number, Difficulty>;
    GameConfigTable: GameConfig;
    ItemTable: Map<number, Item>;
    LeaderboardEntryTable: LeaderboardEntry[];
    MonsterTable: Map<number, Monster>;
    PlayerTable: Map<number, Player>;
    ServerSettingsTable: ServerSettings;
    SkillTable: Map<number, Skill>;
    TeamTable: Map<number, Team>;
    WeaponConfigTable: Map<number, WeaponConfig>;
    WeaponTable: Map<number, Weapon>;
}