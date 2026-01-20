import type { RoleGrowthAttributes } from "./role-growth-attributes";

/**
 * 等级配置
 */
export interface LevelConfig {
    /**
     * 等级
     * @type="int"
     */
    readonly level: number;
    /**
     * 升级所需经验(总经验,非等级段经验)
     * @type="float"
     */
    readonly requiredExp: number;
    /**
     * 成长属性(增加了多少)
     */
    readonly growthAttributes: RoleGrowthAttributes;
    /**
     * 技能初始等级配置(局内游戏技能初始等级)
     * @type="(map#sep=,|),string,int"
     */
    readonly skillLevels: Map<string, number>;
    /**
     * 角色特质
     * @sep="|"
     */
    readonly trait: string[];
}
