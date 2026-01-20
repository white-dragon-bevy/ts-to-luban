/**
 * 抽卡消耗
 */
export interface CostConfig {
    /**
     * 单抽: currencyId->数量
     * @type="(map#sep=,|),string,int"
     */
    readonly single: Map<string, number>;
    /**
     * 十连(未填则=单抽*10)
     * @type="(map#sep=,|),string,int"
     */
    readonly multi10: Map<string, number>;
}
