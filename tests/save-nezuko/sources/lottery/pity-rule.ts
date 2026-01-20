/**
 * 保底（硬+软）
 */
export interface PityRule {
    /**
     * 硬保底最大抽数
     * @type="int"
     * @default="0"
     */
    readonly maxDraws: number;
    /**
     * 软保底起始抽数
     * @type="int"
     * @default="0"
     */
    readonly softStart: number;
    /**
     * 软保底每超出1抽提高倍率增量
     * @type="float"
     * @default="0"
     */
    readonly softStepRate: number;
    /**
     * 目标筛选规则
     */
    readonly targetFilter?: string;
    /**
     * 命中目标后是否重置保底计数
     * @default="true"
     */
    readonly resetOnHit: boolean;
}
