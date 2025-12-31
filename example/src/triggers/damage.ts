interface EntityTrigger {
    
}

/**
 * 伤害触发器
 */
export class DamageTrigger implements EntityTrigger {
    public damage: number;
    public radius: number;
    public targetTags?: string[];
}

/**
 * 治疗触发器
 */
export class HealTrigger implements EntityTrigger {
    public healAmount: number;
    public duration: number;
}



/**
 * 继承
 */
export class HealTrigger2 extends  HealTrigger{
    public healAmount2: number;
}
