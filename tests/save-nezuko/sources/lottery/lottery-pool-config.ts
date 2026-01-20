import type { CostConfig } from "./cost-config";
import type { PityRule } from "./pity-rule";
import type { PrizeConfig } from "./prize-config";
import type { ResourceConfig } from "../resource/resource-config";

/**
 * 奖池
 * @table="map,id"
 * @input="../datas/lottery"
 */
export interface LotteryPoolConfig extends ResourceConfig {
    /**
     * 开始时间
     * @type="long"
     * @default="0"
     */
    readonly openAt: number;
    /**
     * 结束时间
     * @type="long"
     * @default="0"
     */
    readonly closeAt: number;
    /**
     * 奖品列表
     */
    readonly prizes: PrizeConfig[];
    /**
     * 保底
     */
    readonly pity: PityRule;
    /**
     * 消耗
     */
    readonly cost: CostConfig;
    /**
     * 是否允许跳过演出
     * @default="true"
     */
    readonly allowSkip: boolean;
}
