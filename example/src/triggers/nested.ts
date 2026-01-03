/**
 * 嵌套引用类型示例
 * A -> B -> C -> D (interface)
 */

/**
 * 最底层接口
 */
export interface DConfig {
    /** 配置ID */
    id: number;
    /** 配置名称 */
    name: string;
    /** 是否启用 */
    enabled: boolean;
}

/**
 * C 类型，持有 D 接口
 */
export class CComponent {
    /** D 配置 */
    public config: DConfig;
    /** 权重 */
    public weight: number;
    /** 标签列表 */
    public tags: string[];
}

/**
 * B 类型，持有 C 组件
 */
export class BModule {
    /** 主组件 */
    public mainComponent: CComponent;
    /** 备用组件（可选） */
    public backupComponent?: CComponent;
    /** 模块优先级 */
    public priority: number;
}

/**
 * A 类型，持有 B 模块
 * @param module 主模块
 * @param modules 模块列表
 */
export class ASystem {
    /** 主模块 */
    public module: BModule;
    /** 所有模块 */
    public modules: BModule[];
    /** 系统名称 */
    public systemName: string;
    /** 模块映射 */
    public moduleMap: Map<string, BModule>;
}
