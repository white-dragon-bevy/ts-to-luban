/**
 * 技能数据提供者类型
 *
 * 可以是直接的数据对象，或者返回数据的函数
 */
export type SkillDataProvider<T extends object = object> = T | (() => T);

/**
 * 技能状态机接口
 *
 * 定义技能状态机的行为规范
 */
export interface IAmSkillStateMachine {
	/** 状态机标识 */
	readonly id?: string;
}

/**
 * 技能状态机类
 *
 * 实现技能状态机的具体类
 */
export class SkillStateMachine implements IAmSkillStateMachine {
	readonly id?: string;
	
	constructor(skillDataProvider: SkillDataProvider<any>) {
		// 构造函数实现
	}
}

/**
 * 标签鉴别器类型
 *
 * 用于标识和区分不同类型的标签组件
 */
export type TagDiscriminator = string | number | symbol;

/**
 * 组件工厂类型
 *
 * 用于创建组件的工厂函数或构造函数
 */
export type ComponentFactory<T = any> = new (...args: any[]) => T;

/**
 * 可触发的组件工厂定义
 *
 * 定义可以在特定时机触发的组件工厂
 */
export interface TriggerableComponentFactoryDefine {
	/** 组件工厂 */
	factory: ComponentFactory<any>;
	/** 触发条件 */
	condition?: (...args: any[]) => boolean;
	/** 优先级 */
	priority?: number;
}

export interface SkillMetadata{

}

/**
 * 技能数据配置接口
 *
 * 定义技能的静态配置数据，创建时确定且运行时不变。
 */
export interface SkillDataConfig<T extends SkillMetadata> {
	/** 激活持续时间（单位：秒） */
	activeDuration?: number;

	/** 施法持续时间（单位：秒） */
	castDuration: number;

	/** 冷却持续时间（单位：秒） */
	cooldownDuration: number;

	data: T;

	/** 技能唯一标识符 */
	id: string;

	/**
	 * 状态机类型（构造函数）
	 *
	 * 必须是实现 IAmSkillStateMachine 接口的组件类
	 * 构造函数接收 skillDataProvider（自定义数据或数据提供者函数）
	 */
	stateMachine?: new (skillDataProvider: SkillDataProvider<T>) => SkillStateMachine;

	/** 附加的标签组件列表 */
	tags?: ComponentFactory<TagDiscriminator>[];

	/** 激活钩子 - 技能激活时触发 */
	triggerOnActive?: TriggerableComponentFactoryDefine[];

	/** 施法钩子 - 开始施法时触发 */
	triggerOnCast?: TriggerableComponentFactoryDefine[];

	/** 结束钩子 - 技能结束时触发 */
	triggerOnEnd?: TriggerableComponentFactoryDefine[];

	/** 中断钩子 - 技能被中断时触发 */
	triggerOnInterrupt?: TriggerableComponentFactoryDefine[];
}
