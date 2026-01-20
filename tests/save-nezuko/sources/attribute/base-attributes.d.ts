/**
 * 基础属性
 */
export interface BaseAttributes {
    /**
     * 生命值
     * @type="float"
     * @default="0"
     */
    readonly health: number;
    /**
     * 攻击力
     * @type="float"
     * @default="0"
     */
    readonly attack: number;
    /**
     * 速度
     * @type="float"
     * @default="0"
     */
    readonly speed: number;
    /**
     * 技能冷却缩减
     * @type="float"
     * @default="0"
     */
    readonly cdReduction: number;
    /**
     * 暴击率
     * @type="float"
     * @default="0"
     */
    readonly critical: number;
    /**
     * 攻击速度
     * @type="float"
     * @default="0"
     */
    readonly attackSpeed: number;
    /**
     * 增伤
     * @type="float"
     * @default="0"
     */
    readonly damageBonus: number;
    /**
     * 幸运
     * @type="float"
     * @default="0"
     */
    readonly luck: number;
    /**
     * 金币加成
     * @type="float"
     * @default="0"
     */
    readonly coinBonus: number;
}
