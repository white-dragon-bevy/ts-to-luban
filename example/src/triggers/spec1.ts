

type Vector3 = string

/**
 * 爆炸系统类型名称常量
 *
 */
export const EXPLOSION_TYPE_NAME = "explosion";

/**
 * 爆炸形状类型枚举
 */
export enum ExplosionShapeType {
	/** 圆形爆炸（基于半径） */
	Circle = "Circle",
	/** 盒子爆炸（基于边界） */
	Box = "Box",
	/** 扇形爆炸（基于方向和角度） */
	Sector = "Sector",
	/** 三角形爆炸（基于三个顶点） */
	Triangle = "Triangle",
}

/**
 * 圆形爆炸形状配置
 */
export interface CircleExplosionShape {
	readonly type: ExplosionShapeType.Circle;
	/** 爆炸半径（米） */
	readonly radius: number;
}

/**
 * 盒子爆炸形状配置
 */
export interface BoxExplosionShape {
	readonly type: ExplosionShapeType.Box;
	/** 盒子尺寸（长宽高） */
	readonly size: Vector3;
}

/**
 * 扇形爆炸形状配置
 */
export interface SectorExplosionShape {
	readonly type: ExplosionShapeType.Sector;
	/** 扇形半径（米） */
	readonly radius: number;
	/** 扇形角度（弧度） */
	readonly angle: number;
	/** 是否使用爆炸实体的朝向作为方向（默认 true） */
	readonly useEntityDirection?: boolean;
}

/**
 * 三角形爆炸形状配置
 */
export interface TriangleExplosionShape {
	readonly type: ExplosionShapeType.Triangle;
	/** 三角形的三个顶点（相对于爆炸中心的偏移） */
	readonly vertex1: Vector3;
	readonly vertex2: Vector3;
	readonly vertex3: Vector3;
}

/**
 * 爆炸形状配置（联合类型）
 */
export type ExplosionShape =
	| CircleExplosionShape
	| BoxExplosionShape
	| SectorExplosionShape
	| TriangleExplosionShape;

/**
 * 爆炸数据
 */
export interface ExplosionData {

	id:string;

	/**
	 * 爆炸形状配置
	 * 如果未指定，默认使用圆形爆炸（半径从 radius 字段读取）
	 */
	readonly shape?: ExplosionShape;

	/**
	 * 爆炸影响半径（米）
	 * @deprecated 使用 shape 字段替代。为了向后兼容保留此字段
	 */
	readonly radius?: number;

	/**
	 * 爆炸触发器
	 */
	triggerOnExplode: TriggerableComponentFactoryDefine[]
}
