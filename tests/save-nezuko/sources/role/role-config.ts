import type { LevelConfig } from "./level-config";
import type { ResourceConfig } from "../resource/resource-config";
import type { RoleBaseAttributes } from "./role-base-attributes";
import type { SkillConfig } from "./skill-config";

/**
 * 角色基础配置
 * @table="map,id"
 * @input="../datas/role"
 */
export interface RoleConfig extends ResourceConfig {
    /**
     * 角色ID
     * @type="int"
     */
    readonly roleId: number;
    /**
     * 最大等级,(初版最大配置10)
     * @type="int"
     */
    readonly maxLevel: number;
    /**
     * 角色基础属性
     */
    readonly baseAttributes: RoleBaseAttributes;
    /**
     * 经验转换值
     * @type="float"
     */
    readonly roleExp: number;
    /**
     * 等级配置列表
     */
    readonly levels: LevelConfig[];
    /**
     * 技能配置列表
     */
    readonly skills: SkillConfig[];
}
