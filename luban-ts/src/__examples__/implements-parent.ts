/**
 * Example demonstrating implements → parent feature
 *
 * When a class has NO extends but implements exactly ONE interface,
 * its parent is set to that interface.
 */

import { LubanTable, Range, Required } from "../index";

/**
 * Base entity interface
 */
export interface IEntity {
    /** Entity ID */
    id: number;
}

/**
 * Intermediate interface extending base
 */
export interface IEffect extends IEntity {
    /** Effect priority */
    priority: number;
}

/**
 * Fire effect implements single interface
 * Generated: <bean name="FireEffect" parent="IEntity">
 */
export class FireEffect implements IEntity {
    public damage: number;
}

/**
 * Heal effect implements intermediate interface
 * Generated: <bean name="HealEffect" parent="IEffect">
 */
export class HealEffect implements IEffect {
    @Range(0, 100)
    public healAmount: number;
}

/**
 * Poison effect implements single interface with validators
 * Generated: <bean name="PoisonEffect" parent="IEffect">
 */
@LubanTable({ mode: "map", index: "id" })
export class PoisonEffect implements IEffect {
    @Required()
    public name: string;

    @Range(0, 100)
    public duration: number;
}

/**
 * Buff effect extends a class (extends takes priority)
 * Generated: <bean name="BuffEffect" parent="PoisonEffect">
 */
@LubanTable({ mode: "map", index: "id" })
export class BuffEffect extends PoisonEffect {
    public stackable: boolean;
}

/**
 * Effect with multiple implements (ambiguous, no parent)
 * Generated: <bean name="MultiInterfaceEffect">
 */
export class MultiInterfaceEffect implements IEntity, IEffect {
    public id: number;
    public priority: number;
    public value: number;
}

/**
 * Standalone class without implements or extends (no parent)
 * Generated: <bean name="StandaloneClass">
 */
export class StandaloneClass {
    public data: string;
}
