// Example of Constructor<T> type for type registration

import { Constructor, LubanTable, ObjectFactory } from "../index";

/**
 * Base trigger class
 */
export class BaseTrigger {
    public id: number;
    public priority: number;
}

/**
 * Damage trigger implementation
 */
export class DamageTrigger extends BaseTrigger {
    public damage: number;
}

/**
 * Heal trigger implementation
 */
export class HealTrigger extends BaseTrigger {
    public healAmount: number;
}

/**
 * Character configuration with constructor type field
 */
@LubanTable({ mode: "map", index: "id" })
export class CharacterConfig {
    public id: number;
    public name: string;

    // Store constructor reference for trigger
    // In Excel, store class name like "DamageTrigger" or "HealTrigger"
    public triggerType: Constructor<BaseTrigger>;

}
