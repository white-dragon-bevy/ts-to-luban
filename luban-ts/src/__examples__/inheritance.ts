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
 */
export class CharacterUnit extends BaseUnit {
    public level: number;
    public hp: number;
}

/**
 * 玩家单位 - 继承 CharacterUnit
 */
export class PlayerUnit extends CharacterUnit {
    public experience: number;

    @Required()
    public accountId: string;
}

/**
 * NPC单位 - 继承 CharacterUnit
 */
export class NPCUnit extends CharacterUnit {
    public dialogue: string[];
}

/**
 * 独立单位 - 无继承
 */
export class StandaloneUnit {
    public data: string;
}
