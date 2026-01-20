/**
 * 抽技能配置
 * @table="one"
 * @input="../datas/roll-skill"
 */
export interface RollSkillConfig {
    /**
     * 每次选择的技能数量
     * @type="int"
     * @default="3"
     */
    readonly selectionCount: number;
    /**
     * 角色相关技能权重乘数
     * @type="float"
     * @default="5"
     */
    readonly characterMultiplier: number;
    /**
     * 协同技能权重乘数
     * @type="float"
     * @default="4"
     */
    readonly synergyMultiplier: number;
    /**
     * 每日等级权重配置列表
     * @type="list,(list,int)"
     */
    readonly dailyLevelWeights: number[][];
    /**
     * 各等级技能的价格
     * @type="list,int"
     */
    readonly levelPrices: number[];
}
