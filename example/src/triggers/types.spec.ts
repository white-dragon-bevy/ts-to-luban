/**
 * 爆炸系统类型测试
 *
 * 测试类型定义和接口
 */

/// &lt;reference types="@rbxts/testez/globals" />

import {
	EXPLOSION_TYPE_NAME,
	ExplosionShapeType,
	type CircleExplosionShape,
	type BoxExplosionShape,
	type SectorExplosionShape,
	type TriangleExplosionShape,
	type ExplosionShape,
	type ExplosionData,
} from "./spec1";

export = (): void => {
	describe("EXPLOSION_TYPE_NAME", () => {
		it("应该是字符串 'explosion'", () => {
			expect(EXPLOSION_TYPE_NAME).to.equal("explosion");
		});
	});

	describe("ExplosionShapeType", () => {
		it("应该包含 Circle 类型", () => {
			expect(ExplosionShapeType.Circle).to.equal("Circle");
		});

		it("应该包含 Box 类型", () => {
			expect(ExplosionShapeType.Box).to.equal("Box");
		});

		it("应该包含 Sector 类型", () => {
			expect(ExplosionShapeType.Sector).to.equal("Sector");
		});

		it("应该包含 Triangle 类型", () => {
			expect(ExplosionShapeType.Triangle).to.equal("Triangle");
		});
	});

	describe("CircleExplosionShape", () => {
		it("应该正确创建圆形爆炸形状", () => {
			const shape: CircleExplosionShape = {
				type: ExplosionShapeType.Circle,
				radius: 10,
			};

			expect(shape.type).to.equal(ExplosionShapeType.Circle);
			expect(shape.radius).to.equal(10);
		});

		it("应该支持不同的半径值", () => {
			const smallShape: CircleExplosionShape = {
				type: ExplosionShapeType.Circle,
				radius: 5,
			};

			const largeShape: CircleExplosionShape = {
				type: ExplosionShapeType.Circle,
				radius: 50,
			};

			expect(smallShape.radius).to.equal(5);
			expect(largeShape.radius).to.equal(50);
		});
	});

	describe("BoxExplosionShape", () => {
		it("应该正确创建盒子爆炸形状", () => {
			const shape: BoxExplosionShape = {
				type: ExplosionShapeType.Box,
				size: new Vector3(10, 5, 10),
			};

			expect(shape.type).to.equal(ExplosionShapeType.Box);
			expect(shape.size).to.equal(new Vector3(10, 5, 10));
		});

		it("应该支持不同的尺寸", () => {
			const shape: BoxExplosionShape = {
				type: ExplosionShapeType.Box,
				size: new Vector3(20, 10, 15),
			};

			expect(shape.size.X).to.equal(20);
			expect(shape.size.Y).to.equal(10);
			expect(shape.size.Z).to.equal(15);
		});
	});

	describe("SectorExplosionShape", () => {
		it("应该正确创建扇形爆炸形状", () => {
			const shape: SectorExplosionShape = {
				type: ExplosionShapeType.Sector,
				radius: 15,
				angle: math.pi / 2,
			};

			expect(shape.type).to.equal(ExplosionShapeType.Sector);
			expect(shape.radius).to.equal(15);
			expect(shape.angle).to.equal(math.pi / 2);
		});

		it("应该支持使用实体朝向", () => {
			const shape: SectorExplosionShape = {
				type: ExplosionShapeType.Sector,
				radius: 10,
				angle: math.pi / 3,
				useEntityDirection: true,
			};

			expect(shape.useEntityDirection).to.equal(true);
		});

		it("应该支持不使用实体朝向", () => {
			const shape: SectorExplosionShape = {
				type: ExplosionShapeType.Sector,
				radius: 10,
				angle: math.pi / 4,
				useEntityDirection: false,
			};

			expect(shape.useEntityDirection).to.equal(false);
		});
	});

	describe("TriangleExplosionShape", () => {
		it("应该正确创建三角形爆炸形状", () => {
			const shape: TriangleExplosionShape = {
				type: ExplosionShapeType.Triangle,
				vertex1: new Vector3(0, 0, 0),
				vertex2: new Vector3(10, 0, 0),
				vertex3: new Vector3(5, 0, 10),
			};

			expect(shape.type).to.equal(ExplosionShapeType.Triangle);
			expect(shape.vertex1).to.equal(new Vector3(0, 0, 0));
			expect(shape.vertex2).to.equal(new Vector3(10, 0, 0));
			expect(shape.vertex3).to.equal(new Vector3(5, 0, 10));
		});

		it("应该支持任意顶点位置", () => {
			const shape: TriangleExplosionShape = {
				type: ExplosionShapeType.Triangle,
				vertex1: new Vector3(-5, 2, -5),
				vertex2: new Vector3(5, 2, -5),
				vertex3: new Vector3(0, 2, 5),
			};

			expect(shape.vertex1.X).to.equal(-5);
			expect(shape.vertex2.X).to.equal(5);
			expect(shape.vertex3.Z).to.equal(5);
		});
	});

	describe("ExplosionData", () => {
		it("应该正确创建爆炸数据", () => {
			const data: ExplosionData = {
				id: "explosion_001",
				shape: {
					type: ExplosionShapeType.Circle,
					radius: 10,
				},
				triggerOnExplode: [],
			};

			expect(data.id).to.equal("explosion_001");
			expect(data.shape).to.be.ok();
			expect((data.shape as CircleExplosionShape).radius).to.equal(10);
		});

		it("应该支持向后兼容的 radius 字段", () => {
			const data: ExplosionData = {
				id: "legacy_explosion",
				radius: 15,
				triggerOnExplode: [],
			};

			expect(data.radius).to.equal(15);
			expect(data.shape).never.to.be.ok();
		});

		it("应该支持空的触发器列表", () => {
			const data: ExplosionData = {
				id: "no_trigger",
				shape: {
					type: ExplosionShapeType.Box,
					size: new Vector3(5, 5, 5),
				},
				triggerOnExplode: [],
			};

			expect(data.triggerOnExplode.size()).to.equal(0);
		});

		it("应该支持不同的爆炸形状", () => {
			const circleData: ExplosionData = {
				id: "circle_explosion",
				shape: {
					type: ExplosionShapeType.Circle,
					radius: 10,
				},
				triggerOnExplode: [],
			};

			const boxData: ExplosionData = {
				id: "box_explosion",
				shape: {
					type: ExplosionShapeType.Box,
					size: new Vector3(10, 5, 10),
				},
				triggerOnExplode: [],
			};

			expect(circleData.shape?.type).to.equal(ExplosionShapeType.Circle);
			expect(boxData.shape?.type).to.equal(ExplosionShapeType.Box);
		});
	});

	describe("ExplosionShape 联合类型", () => {
		it("应该能够使用类型守卫", () => {
			const shapes: ExplosionShape[] = [
				{ type: ExplosionShapeType.Circle, radius: 10 },
				{ type: ExplosionShapeType.Box, size: new Vector3(5, 5, 5) },
				{ type: ExplosionShapeType.Sector, radius: 15, angle: math.pi / 2 },
			];

			shapes.forEach((shape) => {
				expect(shape.type).to.be.a("string");

				if (shape.type === ExplosionShapeType.Circle) {
					expect((shape as CircleExplosionShape).radius).to.be.a("number");
				} else if (shape.type === ExplosionShapeType.Box) {
					expect((shape as BoxExplosionShape).size).to.be.ok();
				} else if (shape.type === ExplosionShapeType.Sector) {
					expect((shape as SectorExplosionShape).angle).to.be.a("number");
				}
			});
		});
	});
};
