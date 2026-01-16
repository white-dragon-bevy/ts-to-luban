import { TestPrivateFields, TestReadonlyFields } from "../../__examples__/field-visibility";
import { ScalingStat, WeaponConfig, ArmorConfig } from "../../__examples__/virtual-fields";
import { Weapon, Armor, EquipmentSet } from "../../__examples__/items";
import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { BaseUnit, CharacterUnit, PlayerUnit, NPCUnit, StandaloneUnit } from "../../__examples__/inheritance";
import { BaseTrigger, DamageTrigger, HealTrigger, CharacterConfig } from "../../__examples__/constructor-type";
import { Item, Skill, DropItem, Monster, Player, Difficulty, Team } from "../../__examples__/all-validators";

export const Beans = {
    "constructor.BaseTrigger": BaseTrigger,
    "constructor.CharacterConfig": CharacterConfig,
    "constructor.DamageTrigger": DamageTrigger,
    "constructor.HealTrigger": HealTrigger,
    "examples.Difficulty": Difficulty,
    "examples.DropItem": DropItem,
    "examples.Item": Item,
    "examples.Monster": Monster,
    "examples.Player": Player,
    "examples.Skill": Skill,
    "examples.Team": Team,
    "field_visibility.TestPrivateFields": TestPrivateFields,
    "field_visibility.TestReadonlyFields": TestReadonlyFields,
    "inheritance.BaseUnit": BaseUnit,
    "inheritance.CharacterUnit": CharacterUnit,
    "inheritance.NPCUnit": NPCUnit,
    "inheritance.PlayerUnit": PlayerUnit,
    "inheritance.StandaloneUnit": StandaloneUnit,
    "items.Armor": Armor,
    "items.EquipmentSet": EquipmentSet,
    "items.Weapon": Weapon,
    "modes.GameConfig": GameConfig,
    "modes.LeaderboardEntry": LeaderboardEntry,
    "modes.ServerSettings": ServerSettings,
    "virtual_fields.ArmorConfig": ArmorConfig,
    "virtual_fields.ScalingStat": ScalingStat,
    "virtual_fields.WeaponConfig": WeaponConfig,
} as const;