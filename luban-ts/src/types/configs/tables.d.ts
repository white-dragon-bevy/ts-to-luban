import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { Item, Skill, Monster, Player, Difficulty, Team } from "../../__examples__/all-validators";
import { CharacterConfig } from "../../__examples__/constructor-type";
import { Weapon, Armor } from "../../__examples__/items";
import { WeaponConfig, ArmorConfig } from "../../__examples__/virtual-fields";

export interface AllTables {
    ItemTable: Map<number, Item>;
    SkillTable: Map<number, Skill>;
    MonsterTable: Map<number, Monster>;
    PlayerTable: Map<number, Player>;
    DifficultyTable: Map<number, Difficulty>;
    TeamTable: Map<number, Team>;
    WeaponTable: Map<number, Weapon>;
    ArmorTable: Map<number, Armor>;
    LeaderboardEntryTable: LeaderboardEntry[];
    GameConfigTable: GameConfig;
    ServerSettingsTable: ServerSettings;
    CharacterConfigTable: Map<number, CharacterConfig>;
    WeaponConfigTable: Map<number, WeaponConfig>;
    ArmorConfigTable: Map<number, ArmorConfig>;
}