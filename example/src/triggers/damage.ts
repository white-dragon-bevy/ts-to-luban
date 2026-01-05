/**
 * 基础触发器接口（无 parent）
 */
export interface BaseTrigger {
    /** 触发器ID */
    id: number;
}

/**
 * 实体触发器接口，继承 BaseTrigger（有 parent）
 */
export interface EntityTrigger extends BaseTrigger {
    /** 实体数量 */
    num: number;
}

/**
 * 高级触发器接口，继承 EntityTrigger（有 parent）
 */
export interface AdvancedTrigger extends EntityTrigger {
    /** 优先级 */
    priority: number;
    /** 是否启用 */
    enabled: boolean;
}

/**
 * 伤害触发器
 * @alias:伤害
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
export class HealTrigger implements AdvancedTrigger {
    priority: number;
    enabled: boolean;
    num: number;
    id: number;
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

/**
 * 内部使用的辅助类，不导出到 Luban
 * @ignore
 */
export class InternalHelper {
    public helperData: string;
}

/**
 * 内部接口，不导出到 Luban
 * @ignore
 */
export interface InternalInterface {
    internalField: number;
}
