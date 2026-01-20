/**
 * 继承关系测试
 */

import { Required } from "../index";

/**
 * 基础单位
 */
export class BaseUnit {
    public id: number;

    @Required()
    public name: string;
}

/**
 * 角色单位 - 继承 BaseUnit
 * Note: In roblox-ts, child classes must redeclare parent fields for proper property access
 */
export class CharacterUnit extends BaseUnit {
    // Redeclare parent fields for roblox-ts compatibility
    declare public id: number;
    declare public name: string;

    public level: number;
    public hp: number;
}

/**
 * 玩家单位 - 继承 CharacterUnit
 * Note: In roblox-ts, child classes must redeclare parent fields for proper property access
 */
export class PlayerUnit extends CharacterUnit {
    // Redeclare parent fields for roblox-ts compatibility
    declare public id: number;
    declare public name: string;
    declare public level: number;
    declare public hp: number;

    public experience: number;

    @Required()
    public accountId: string;
}

/**
 * NPC单位 - 继承 CharacterUnit
 * Note: In roblox-ts, child classes must redeclare parent fields for proper property access
 */
export class NPCUnit extends CharacterUnit {
    // Redeclare parent fields for roblox-ts compatibility
    declare public id: number;
    declare public name: string;
    declare public level: number;
    declare public hp: number;

    public dialogue: string[];
}

/**
 * 独立单位 - 无继承
 */
export class StandaloneUnit {
    public data: string;
}
