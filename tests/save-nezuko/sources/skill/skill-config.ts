import type { ResourceConfig } from "../resource/resource-config";
import type { SkillStyle } from "../enums/skill-style";

/**
 * 技能配置
 * @table="map,id"
 * @input="../datas/skill-hufan"
 */
export interface SkillConfig extends ResourceConfig {
    /**
     * 技能最大等级
     * @type="int"
     * @default="5"
     */
    readonly maxLevel: number;
    /**
     * 基础权重 (用于随机选择时的权重计算)
     * @type="float"
     * @default="1"
     */
    readonly baseWeight: number;
    /**
     * 技能标签列表
     * @sep="|"
     */
    readonly tags: string[];
    /**
     * 技能类型列表
     * @sep="|"
     */
    readonly styles: SkillStyle[];
}
