import type { QualityType } from "../enums/quality-type";

/**
 * 资源基础配置
 */
export interface ResourceConfig {
    /**
     * 资源ID
     */
    readonly id: string;
    /**
     * 资源名称
     */
    readonly name: string;
    /**
     * 资源图标
     */
    readonly icon?: string;
    /**
     * 资源描述
     */
    readonly description?: string;
    /**
     * 资源品质
     */
    readonly quality?: QualityType;
}
