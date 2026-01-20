import type { AttributePiece } from "./attribute-piece";

/**
 * 锻造配置
 * @input="../datas/forge"
 */
export interface ForgeConfig {
    /**
     * 锻造天数
     * @type="int"
     */
    readonly day: number;
    /**
     * 属性碎片列表
     */
    readonly attributePieces: AttributePiece[];
}
