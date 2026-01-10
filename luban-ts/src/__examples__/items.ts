/**
 * 物品模块 - 独立数据源测试
 */

import { LubanTable, Range, Required, Ref } from "../index";

/**
 * 武器配置
 * @alias:武器
 */
@LubanTable({ mode: "map", index: "id" })
export class Weapon {
    public id: number;

    @Required()
    public name: string;

    @Range(1, 100)
    public damage: number;

    @Range(0.5, 2.0)
    public attackSpeed: number;
}

/**
 * 防具配置
 * @alias:防具
 */
@LubanTable({ mode: "map", index: "id" })
export class Armor {
    public id: number;

    @Required()
    public name: string;

    @Range(1, 500)
    public defense: number;
}

/**
 * 装备套装
 */
export class EquipmentSet {
    @Ref(Weapon)
    public weaponId: number;

    @Ref(Armor)
    public armorId: number;
}
