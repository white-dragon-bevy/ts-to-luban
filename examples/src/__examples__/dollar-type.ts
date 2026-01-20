import { $type } from "../index";

/**
 * $type<T> 类型包装器示例
 * $type<T> 会在生成时提取内部类型 T，忽略 $type 包装器
 */

export class BaseEntity {
    public id: number;
    public name: string;
}

export class Hero extends BaseEntity {
    public level: number;
}

export class Enemy extends BaseEntity {
    public hp: number;
}

/**
 * 使用 $type<T> 包装器的配置类
 */
export class EntityConfig {
    /** 实体类型 - 使用 $type 包装 */
    public entityType: $type<BaseEntity>;

    /** 实体列表 */
    public entities: $type<BaseEntity>[];

    /** 实体映射 */
    public entityMap: Map<string, $type<BaseEntity>>;

    /** 普通字段 */
    public description: string;
}

/**
 * 嵌套使用示例
 */
export class ComplexConfig {
    /** 英雄类型 */
    public heroType: $type<Hero>;

    /** 敌人类型 */
    public enemyType: $type<Enemy>;

    /** 混合列表 */
    public mixedList: Array<$type<BaseEntity>>;
}
