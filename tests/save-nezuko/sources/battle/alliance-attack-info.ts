import type { AttackInfo } from "./attack-info";

/**
 * @table="map,Id"
 * @input="../datas/battle/我方普通攻击配置表.xlsx"
 */
export interface AllianceAttackInfo {
    /**
     * 角色编号
     */
    readonly Id: string;
    /**
     * 角色普通攻击列表
     * @sep=";"
     */
    readonly Attacks: AttackInfo[];
}
