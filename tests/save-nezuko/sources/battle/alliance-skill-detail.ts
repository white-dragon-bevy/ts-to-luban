import type { AllianceSkillPointInfo } from "./alliance-skill-point-info";

export interface AllianceSkillDetail {
    /**
     * 持续时间
     * @type="float"
     */
    readonly Duration: number;
    /**
     * 最小持续时间
     * @type="float"
     */
    readonly MinDuration?: number;
    /**
     * 伤害倍率
     * @type="float"
     */
    readonly Multiple?: number;
    /**
     * 震屏配置
     */
    readonly ShakeInfo?: string;
    /**
     * 技能时间点列表
     * @sep=";"
     */
    readonly Points: AllianceSkillPointInfo[];
}
