export interface AttackPointInfo {
    /**
     * 触发时间
     * @type="float"
     */
    readonly Time: number;
    /**
     * 伤害倍率
     * @type="float"
     */
    readonly Multiple?: number;
    /**
     * 震屏配置
     */
    readonly ShakeInfo?: string;
}
