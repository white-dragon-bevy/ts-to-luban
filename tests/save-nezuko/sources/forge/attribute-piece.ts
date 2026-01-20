import type { AttributeEffect } from "./attribute-effect";
import type { PieceType } from "./piece-type";

/**
 * 属性碎片, 从effects中获取固定属性, randomEffects中获取随机属性, specialEffect用于特殊效果, 只能选取一个, 优先级依次降低
 */
export interface AttributePiece {
    /**
     * 属性碎片ID
     * @type="int"
     */
    readonly id: number;
    /**
     * 碎片类型(基础/传说/神话/增幅)
     */
    readonly pieceType: PieceType;
    /**
     * 固定属性效果列表, 最高优先级,
     */
    readonly effects: AttributeEffect[];
    /**
     * 随机属性配置
     */
    readonly randomEffects: AttributeEffect[];
    /**
     * 权重
     * @type="float"
     */
    readonly weight: number;
}
