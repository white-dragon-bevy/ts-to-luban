import { Item, Skill, DropItem, Monster, Player, Difficulty, Team } from "../../__examples__/all-validators";
import { BaseUnit, CharacterUnit, PlayerUnit, NPCUnit, StandaloneUnit } from "../../__examples__/inheritance";
import { BaseTrigger, DamageTrigger, HealTrigger, CharacterConfig } from "../../__examples__/constructor-type";
import { ShapeInfo, CircleShape, RectangleShape } from "../../__examples__/discriminated-union";
import { TestPrivateFields, ITestInterface, TestReadonlyFields } from "../../__examples__/field-visibility";
import { LeaderboardEntry, GameConfig, ServerSettings } from "../../__examples__/table-modes";
import { Weapon, Armor, EquipmentSet } from "../../__examples__/items";
import { BaseEntity, Hero, Enemy, EntityConfig, ComplexConfig } from "../../__examples__/dollar-type";
import { ScalingStat, WeaponConfig, ArmorConfig } from "../../__examples__/virtual-fields";

export const Beans = {
    "constructor.BaseTrigger": BaseTrigger,
    "constructor.CharacterConfig": CharacterConfig,
    "constructor.DamageTrigger": DamageTrigger,
    "constructor.HealTrigger": HealTrigger,
    "discriminated_union.CircleShape": CircleShape,
    "discriminated_union.RectangleShape": RectangleShape,
    "discriminated_union.ShapeInfo": ShapeInfo,
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
    "field_visibility.ITestInterface": ITestInterface,
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