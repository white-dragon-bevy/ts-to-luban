import { LubanTable } from "../index";

/**
 * 武器配置
 * 注意：mainStat 和 subStat 字段不会在 TypeScript 中定义
 * 它们通过 luban.config.toml 中的 virtual_fields 配置添加
 */
@LubanTable({ mode: "map", index: "id" })
export class WeaponConfig {
    public id: number;
    public name: string;
    // 虚拟字段（通过配置添加）:
    // - mainStat: ScalingStat (relocateTo=TScalingStat,prefix=_main)
    // - subStat: ScalingStat (relocateTo=TScalingStat,prefix=_sub)
}

/**
 * 装甲配置
 */
@LubanTable({ mode: "map", index: "id" })
export class ArmorConfig {
    public id: number;
    public name: string;
    // 虚拟字段（通过配置添加）:
    // - defenseStat: ScalingStat (relocateTo=TScalingStat,prefix=_defense)
}
