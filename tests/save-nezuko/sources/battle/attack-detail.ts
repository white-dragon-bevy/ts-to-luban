import type { AttackPointInfo } from "./attack-point-info";

export interface AttackDetail {
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
     * 震屏配置
     */
    readonly ShakeInfo?: string;
    /**
     * 攻击时间点列表
     * @sep=";"
     */
    readonly Points: AttackPointInfo[];
}
