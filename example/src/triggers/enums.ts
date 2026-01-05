/**
 * 物品类型枚举
 * 使用字符串值
 */
export enum ItemType {
    /** 角色 */
    Role = "role",
    /** 消耗品 */
    Consumable = "consumable",
    /** 货币 */
    Currency = "currency",
}

/**
 * 技能类型
 * 使用数值
 */
export enum SkillStyle {
    /** 攻击技能 */
    Attack = 1,
    /** 防御技能 */
    Defense = 2,
    /** 辅助技能 */
    Support = 3,
    /** 控制技能 */
    Control = 4,
    /** 治疗技能 */
    Heal = 5,
}

/**
 * 触发器状态
 */
export enum TriggerState {
    /** 待机 */
    Idle = 0,
    /** 激活 */
    Active = 1,
    /** 冷却中 */
    Cooldown = 2,
    /** 已禁用 */
    Disabled = 3,
}


/**
 * 单位标志位枚举 - 纯权限控制系统
 *
 * 32 位标志位仅用于权限控制，不包含具体业务状态
 * 业务状态（如眩晕、冰冻）应由 Buff 系统通过 mode='revoke' 剥夺权限实现
 *
 * 分组:
 * - 0-7 位:   基础行为权限
 * - 8-15 位:  移动相关权限
 * - 16-23 位: 战斗相关权限
 * - 24-31 位: 特殊能力权限（含预留）
 *
 * 使用方式:
 * - Buff 通过 mode='grant' 授予权限
 * - Buff 通过 mode='revoke' 剥夺权限
 * 
 * @flags="true"
 */
export enum UnitFlag {
	/** 无权限 */
	/** @alias="无" */
	NONE = 0,

	// ========== 基础行为权限 (0-7位) ==========
	/**
	 * 可以移动
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现眩晕、定身等效果
	 * @alias="移动"
	 */
	CAN_MOVE = 1 << 0,

	/**
	 * 可以普通攻击
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现缴械、眩晕等效果
	 * @alias="攻击"
	 */
	CAN_ATTACK = 1 << 1,

	/**
	 * 可以使用技能
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现沉默、眩晕等效果
	 * @alias="技能"
	 */
	CAN_USE_SKILL = 1 << 2,

	/**
	 * 可以交互（拾取道具、开门等）
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现眩晕等效果
	 * @alias="交互"
	 */
	CAN_INTERACT = 1 << 3,

	/**
	 * 可以被选中为目标
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现隐身、无敌等效果
	 * @alias="选中"
	 */
	CAN_BE_TARGETED = 1 << 4,

	/**
	 * 可以接受治疗
	 * Buff 系统通过 mode='revoke' 剥夺此权限（如"重伤" Debuff）
	 * @alias="受治疗"
	 */
	CAN_RECEIVE_HEAL = 1 << 5,

	/**
	 * 可以接受增益效果
	 * Buff 系统通过 mode='revoke' 剥夺此权限（如"诅咒" Debuff）
	 * @alias="受增益"
	 */
	CAN_RECEIVE_BUFF = 1 << 6,

	/**
	 * 可以转向
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现眩晕、恐惧等效果
	 * @alias="转向"
	 */
	CAN_TURN = 1 << 7,

	// ========== 移动相关权限 (8-15位) ==========
	/**
	 * 可以跳跃
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现定身、冰冻等效果
	 * @alias="跳跃"
	 */
	CAN_JUMP = 1 << 8,

	/**
	 * 可以冲刺/闪避
	 * Buff 系统通过 mode='revoke' 剥夺此权限实现定身、虚弱等效果
	 * @alias="冲刺"
	 */
	CAN_DASH = 1 << 9,

	/**
	 * 可以飞行
	 * 飞行单位固有，或由特殊 Buff 通过 mode='grant' 授予
	 * @alias="飞行"
	 */
	CAN_FLY = 1 << 10,

	/**
	 * 忽略碰撞
	 * 幽灵形态、穿透 Buff 通过 mode='grant' 授予
	 * @alias="忽略碰撞"
	 */
	IGNORE_COLLISION = 1 << 11,

	/**
	 * 忽略重力
	 * 飞行状态、悬浮 Buff 通过 mode='grant' 授予
	 * @alias="忽略重力"
	 */
	IGNORE_GRAVITY = 1 << 12,

	/**
	 * 预留位，勿用
	 * @alias="预留13"
	 */
	RESERVED_13 = 1 << 13,

	/**
	 * 预留位，勿用
	 * @alias="预留14"
	 */
	RESERVED_14 = 1 << 14,

	/**
	 * 预留位，勿用
	 * @alias="预留15"
	 */
	RESERVED_15 = 1 << 15,

	// ========== 战斗相关权限 (16-23位) ==========
	/**
	 * 可以暴击
	 * Buff 系统通过 mode='revoke' 剥夺此权限（如"虚弱" Debuff）
	 * @alias="暴击"
	 */
	CAN_CRIT = 1 << 16,

	/**
	 * 可以格挡伤害
	 * Buff 系统通过 mode='revoke' 剥夺此权限（如"破防" Debuff）
	 * @alias="格挡"
	 */
	CAN_BLOCK_DAMAGE = 1 << 17,

	/**
	 * 预留位，勿用
	 * @alias="预留18"
	 */
	RESERVED_18 = 1 << 18,

	/**
	 * 物理免疫
	 * 特殊 Buff 或技能通过 mode='grant' 授予（如"金身"）
	 * @alias="物免"
	 */
	IMMUNE_PHYSICAL = 1 << 19,

	/**
	 * 魔法免疫
	 * 特殊 Buff 或技能通过 mode='grant' 授予（如"魔法护盾"）
	 * @alias="魔免"
	 */
	IMMUNE_MAGICAL = 1 << 20,

	/**
	 * 控制免疫
	 * BOSS 固有能力，或特殊 Buff 通过 mode='grant' 授予（如"霸体"）
	 * @alias="控免"
	 */
	IMMUNE_CONTROL = 1 << 21,

	/**
	 * 预留位，勿用
	 * @alias="预留22"
	 */
	RESERVED_22 = 1 << 22,

	/**
	 * 预留位，勿用
	 * @alias="预留23"
	 */
	RESERVED_23 = 1 << 23,

	// ========== 特殊能力权限 (24-31位) ==========
	/**
	 * 可以看见隐身单位
	 * 特殊技能或 Buff 通过 mode='grant' 授予（如"真视"）
	 * @alias="真视"
	 */
	CAN_SEE_INVISIBLE = 1 << 24,

	/**
	 * 预留位，勿用
	 * @alias="预留25"
	 */
	RESERVED_25 = 1 << 25,

	/**
	 * 预留位，勿用
	 * @alias="预留26"
	 */
	RESERVED_26 = 1 << 26,

	/**
	 * 在地图上显示
	 * 正常单位默认拥有，Buff 系统通过 mode='revoke' 剥夺此权限实现隐身
	 * @alias="显形"
	 */
	REVEAL_ON_MAP = 1 << 27,

	/**
	 * 预留位，勿用
	 * @alias="预留28"
	 */
	RESERVED_28 = 1 << 28,

	/**
	 * 预留位，勿用
	 * @alias="预留29"
	 */
	RESERVED_29 = 1 << 29,

	/**
	 * 预留位，勿用
	 * @alias="预留30"
	 */
	RESERVED_30 = 1 << 30,

	// ========== 快捷组合 ==========
	/**
	 * bit0-7全开：移动/攻击/技能/交互/被选中/受治疗/受增益/转向
	 * @alias="基础"
	 */
	BASICS = CAN_MOVE | CAN_ATTACK | CAN_USE_SKILL | CAN_INTERACT | CAN_BE_TARGETED | CAN_RECEIVE_HEAL | CAN_RECEIVE_BUFF | CAN_TURN,

	/**
	 * 硬控：剥夺所有机动与输出
	 * =CAN_MOVE|CAN_ATTACK|CAN_USE_SKILL|CAN_TURN|CAN_JUMP|CAN_DASH|CAN_FLY —— 封死所有机动与输出
	 * @alias="硬控"
	 */
	HARD_CC = CAN_MOVE | CAN_ATTACK | CAN_USE_SKILL | CAN_TURN | CAN_JUMP | CAN_DASH | CAN_FLY,

	/**
	 * 无敌：物理+魔法双免疫，仍吃控
	 * =IMMUNE_PHYSICAL|IMMUNE_MAGICAL —— 不掉血，可被控
	 * @alias="无敌"
	 */
	INVINCIBLE = IMMUNE_PHYSICAL | IMMUNE_MAGICAL,

	/**
	 * 定身：锁位移但可打
	 * =CAN_MOVE|CAN_TURN|CAN_JUMP|CAN_DASH —— 原地锁死，可攻击放技能
	 * @alias="定身"
	 */
	IMMOBILIZED = CAN_MOVE | CAN_TURN | CAN_JUMP | CAN_DASH,

	/**
	 * 空中定身：连飞行一起锁，飞行单位直接落地
	 * =CAN_MOVE|CAN_TURN|CAN_JUMP|CAN_DASH|CAN_FLY —— 飞行单位被锁后原地掉落
	 * @alias="空中定身"
	 */
	IMMOBILIZED_AIR = CAN_MOVE | CAN_TURN | CAN_JUMP | CAN_DASH | CAN_FLY,
}

/**
 * 权限修改组件
 * @alias="权限修正"
 */
export interface UnitFlagsModifier {
	/** 授予的权限位 */
	grant?: UnitFlag;
	/** 剥夺的权限位 */
	revoke?: UnitFlag;
}

/**
 * 单位权限预设
 */
export interface UnitFlagsPreset {
	/** 组合唯一标识（中文名称） */
	id: string;
	/** 单位基础权限 */
	baseFlags: UnitFlag;
	/** 单位免疫权限,该掩码不会被其他掩码覆盖 */
	immunityMask: UnitFlag;
}

