import type { AllianceSkillDetail } from "./alliance-skill-detail";

/**
 * @table="map,Id"
 * @input="../datas/battle/我方技能配置表.xlsx"
 */
export interface AllianceSkillInfo {
    /**
     * 技能Id
     */
    readonly Id: string;
    /**
     * 解锁等级
     * @type="int"
     */
    readonly UnlockLevel: number;
    /**
     * 图标资产
     */
    readonly IconAsset: string;
    /**
     * 能量消耗
     * @type="int"
     */
    readonly EnergyCost: number;
    /**
     * 技能详情
     */
    readonly Points: AllianceSkillDetail;
}
