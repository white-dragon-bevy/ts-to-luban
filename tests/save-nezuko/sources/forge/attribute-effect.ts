import type { PieceAttributeType } from "./piece-attribute-type";

/**
 * 单个属性效果
 */
export interface AttributeEffect {
    /**
     * 属性类型
     */
    readonly attribute: PieceAttributeType;
    /**
     * 最小值
     * @type="float"
     */
    readonly minValue: number;
    /**
     * 最大值
     * @type="float"
     */
    readonly maxValue: number;
    /**
     * 幸运影响系数
     * @type="float"
     */
    readonly luckInfluence?: number;
    /**
     * 标准差比例(相对于范围)
     * @type="float"
     */
    readonly stdDevRatio?: number;
}
