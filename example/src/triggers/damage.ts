export interface EntityTrigger {
    num:number
}

/**
 * 伤害触发器
 */
export class DamageTrigger {
    public damage: number;
    public radius: number;
    public targetTags?: string[];
}

/**
 * 治疗触发器
 * @param healAmount - 治疗量
 * @param duration - 治疗持续时间
 */
export class HealTrigger {
    public healAmount: number;
    public duration: number;
}

/**
 * 继承
 */
export class HealTrigger2 extends  HealTrigger{
    public healAmount2: number;
    public ref:HealTrigger;
}
