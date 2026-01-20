import type { ItemType } from "../enums/item-type";
import type { QualityType } from "../enums/quality-type";

/**
 * 奖品
 */
export interface PrizeConfig {
    /**
     * 奖品ID(唯一)
     * @type="int"
     */
    readonly prizeId: number;
    /**
     * 类型
     */
    readonly type: ItemType;
    /**
     * 关联资源Key
     */
    readonly itemId: string;
    /**
     * 数量(默认1)
     * @type="int"
     * @default="1"
     */
    readonly amount: number;
    /**
     * 品质(默认1)
     * @default="1"
     */
    readonly quality: QualityType;
    /**
     * 基础权重
     * @type="int"
     */
    readonly weight: number;
    /**
     * 标签(如:SSR|LIMITED)
     * @sep="|"
     */
    readonly tags: string[];
    /**
     * 是否前端展示
     * @default="false"
     */
    readonly show: boolean;
}
