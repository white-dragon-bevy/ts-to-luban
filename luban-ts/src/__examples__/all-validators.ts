/**
 * Luban 装饰器完整示例
 * 演示所有验证器的生成
 *
 * 命名规则：
 * - TypeScript 类名: Item (bean 名称)
 * - 生成的 table 名称: ItemTable
 * - @Ref(Item) 生成: #ref=examples.ItemTable
 */

import { LubanTable, Range, Required, Size, Set, Index, Ref } from "../index";

// ============================================
// 基础表定义
// ============================================

/**
 * 道具配置
 * @alias:道具
 */
@LubanTable({ mode: "map", index: "id" })
export class Item {
    public id: number;

    @Required()
    public name: string;

    @Set("weapon", "armor", "consumable", "material")
    public category: string;

    @Range(1, 999)
    public stackLimit: number;

}

/**
 * 技能配置
 * @alias:技能
 */
@LubanTable({ mode: "map", index: "id" })
export class Skill {
    public id: number;

    @Required()
    public skillName: string;

    @Range(0, 100)
    public cooldown: number;
}

// ============================================
// 复杂结构
// ============================================

/**
 * 掉落物品
 */
export class DropItem {
    /** 引用道具表 -> #ref=examples.ItemTable */
    @Ref(Item)
    public itemId: number;

    @Range(1, 100)
    public count: number;

    @Range(0, 100)
    public probability: number;
}

/**
 * 怪物配置
 * @alias:怪物
 */
@LubanTable({ mode: "map", index: "id" })
export class Monster {
    public id: number;

    @Required()
    public name: string;

    @Range(1, 100)
    public level: number;

    @Range(1, 999999)
    public hp: number;

    /**
     * 技能列表 - 最多4个技能，引用技能表
     * -> (list#size=[1,4]),double#ref=examples.SkillTable
     */
    @Size(1, 4)
    @Ref(Skill)
    public skills: number[];

    /**
     * 掉落列表 - 按道具ID索引
     */
    @Index("itemId")
    public drops: DropItem[];
}

// ============================================
// 其他验证器示例
// ============================================

/**
 * 玩家信息
 */
@LubanTable({ mode: "map", index: "id" })
export class Player {
    public id: number;

    @Required()
    public name: string;

    @Required()
    public avatar: string;

    /** 可选签名 */
    public signature?: string;
}

/**
 * 难度配置
 */
@LubanTable({ mode: "map", index: "id" })
export class Difficulty {
    public id: number;

    @Range(1, 3)
    public difficultyLevel: number;

    public difficultyName: string;
}

/**
 * 队伍配置
 */
@LubanTable({ mode: "map", index: "id" })
export class Team {
    public id: number;

    /** 固定3人 */
    @Size(3)
    public members: number[];

    /** 0-2人替补 */
    @Size(0, 2)
    public substitutes: number[];
}
