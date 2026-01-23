import { Weapon, Armor, EquipmentSet } from "../../__examples__/items";
import { BaseUnit, CharacterUnit, PlayerUnit, NPCUnit, StandaloneUnit } from "../../__examples__/inheritance";
import { CircleShape, RectangleShape } from "../../__examples__/discriminated-union";
import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { Item, Skill, DropItem, Monster, Player, Difficulty, Team } from "../../__examples__/all-validators";
import { BaseTrigger, DamageTrigger, HealTrigger, CharacterConfig } from "../../__examples__/constructor-type";
import { TestPrivateFields, TestReadonlyFields } from "../../__examples__/field-visibility";
import { BaseEntity, Hero, Enemy, EntityConfig, ComplexConfig } from "../../__examples__/dollar-type";

export const Beans = {
    "constructor.BaseTrigger": BaseTrigger,
    "constructor.CharacterConfig": CharacterConfig,
    "constructor.DamageTrigger": DamageTrigger,
    "constructor.HealTrigger": HealTrigger,
    "discriminated_union.CircleShape": CircleShape,
    "discriminated_union.RectangleShape": RectangleShape,
    "dollar_type.BaseEntity": BaseEntity,
    "dollar_type.ComplexConfig": ComplexConfig,
    "dollar_type.Enemy": Enemy,
    "dollar_type.EntityConfig": EntityConfig,
    "dollar_type.Hero": Hero,
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
} as const;