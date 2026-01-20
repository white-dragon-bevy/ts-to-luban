/**
 * 属性类型
 * @tags="string"
 */
export enum PieceAttributeType {
    /**
     * 生命值
     */
    Health = "health",
    /**
     * 攻击力
     */
    Attack = "attack",
    /**
     * 速度
     */
    Speed = "speed",
    /**
     * 技能冷却缩减
     */
    CdReduction = "cdReduction",
    /**
     * 暴击率
     */
    Critical = "critical",
    /**
     * 攻击速度
     */
    AttackSpeed = "attackSpeed",
    /**
     * 增伤
     */
    DamageBonus = "damageBonus",
    /**
     * 幸运
     */
    Luck = "luck",
    /**
     * 金币加成
     */
    CoinBonus = "coinBonus",
    /**
     * 百分比生命值
     */
    HealthPercent = "healthPercent",
    /**
     * 百分比攻击力
     */
    AttackPercent = "attackPercent",
    /**
     * 增幅
     */
    Amplify = "amplify",
    /**
     * 来财, N(minValue)个回合,每回合获取M(maxValue)个金币
     */
    Wealth = "wealth",
}
