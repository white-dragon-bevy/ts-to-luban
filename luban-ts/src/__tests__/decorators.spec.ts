/**
 * Decorator Runtime Tests
 *
 * Note: In roblox-ts environment, decorators are runtime stubs that only exist
 * for XML generation by the Rust tool. These tests verify that decorators can
 * be applied without errors and compile correctly.
 */

import {
	LubanTable,
	Ref,
	Range,
	Required,
	Size,
	Set,
	Index,
	Nominal,
} from "../index";

// Test class to apply decorators to
class TestRefTarget {
	public testField = "";
}

export = () => {
	describe("@LubanTable Decorator", () => {
		it("should apply to class with map mode config", () => {
			@LubanTable({ mode: "map", index: "id" })
			class TestMapTable {
				public id: number;
				public name: string;
			}
			expect(TestMapTable).to.be.ok();
		});

		it("should apply to class with list mode config", () => {
			@LubanTable({ mode: "list", index: "rank" })
			class TestListTable {
				public rank: number;
				public score: number;
			}
			expect(TestListTable).to.be.ok();
		});

		it("should apply to class with one mode config", () => {
			@LubanTable({ mode: "one" })
			class TestOneTable {
				public id: number;
				public value: string;
			}
			expect(TestOneTable).to.be.ok();
		});

		it("should apply to class with singleton mode config", () => {
			@LubanTable({ mode: "singleton" })
			class TestSingletonTable {
				public id: number;
				public data: string;
			}
			expect(TestSingletonTable).to.be.ok();
		});

		it("should accept group option", () => {
			@LubanTable({ mode: "map", index: "id", group: "client" })
			class TestGroupTable {
				public id: number;
			}
			expect(TestGroupTable).to.be.ok();
		});

		it("should accept tags option", () => {
			@LubanTable({ mode: "map", index: "id", tags: "customTag" })
			class TestTagsTable {
				public id: number;
			}
			expect(TestTagsTable).to.be.ok();
		});
	});

	describe("@Ref Decorator", () => {
		it("should apply to field with class reference", () => {
			class RefTest {
				@Ref(TestRefTarget)
				public targetId: number;
			}
			const instance = new RefTest();
			expect(instance).to.be.ok();
		});

		it("should work with primitive field types", () => {
			class RefTest {
				@Ref(TestRefTarget)
				public itemId: number;
			}
			const instance = new RefTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Range Decorator", () => {
		it("should apply to number field with min and max", () => {
			class RangeTest {
				@Range(1, 100)
				public value: number;
			}
			const instance = new RangeTest();
			expect(instance).to.be.ok();
		});

		it("should accept negative ranges", () => {
			class RangeTest {
				@Range(-100, -1)
				public negativeValue: number;
			}
			const instance = new RangeTest();
			expect(instance).to.be.ok();
		});

		it("should accept zero in range", () => {
			class RangeTest {
				@Range(0, 100)
				public zeroToHundred: number;
			}
			const instance = new RangeTest();
			expect(instance).to.be.ok();
		});

		it("should accept decimal ranges", () => {
			class RangeTest {
				@Range(0.1, 0.9)
				public decimalValue: number;
			}
			const instance = new RangeTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Required Decorator", () => {
		it("should apply to string field", () => {
			class RequiredTest {
				@Required()
				public name: string;
			}
			const instance = new RequiredTest();
			expect(instance).to.be.ok();
		});

		it("should apply to number field", () => {
			class RequiredTest {
				@Required()
				public id: number;
			}
			const instance = new RequiredTest();
			expect(instance).to.be.ok();
		});

		it("should apply to boolean field", () => {
			class RequiredTest {
				@Required()
				public active: boolean;
			}
			const instance = new RequiredTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Size Decorator", () => {
		it("should apply with fixed size", () => {
			class SizeTest {
				@Size(3)
				public items: number[];
			}
			const instance = new SizeTest();
			expect(instance).to.be.ok();
		});

		it("should apply with min and max range", () => {
			class SizeTest {
				@Size(1, 5)
				public items: string[];
			}
			const instance = new SizeTest();
			expect(instance).to.be.ok();
		});

		it("should accept zero as min value", () => {
			class SizeTest {
				@Size(0, 10)
				public items: number[];
			}
			const instance = new SizeTest();
			expect(instance).to.be.ok();
		});

		it("should accept equal min and max", () => {
			class SizeTest {
				@Size(5, 5)
				public items: number[];
			}
			const instance = new SizeTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Set Decorator", () => {
		it("should apply with string values", () => {
			class SetTest {
				@Set("weapon", "armor", "consumable")
				public category: string;
			}
			const instance = new SetTest();
			expect(instance).to.be.ok();
		});

		it("should apply with number values", () => {
			class SetTest {
				@Set(1, 2, 3, 4, 5)
				public level: number;
			}
			const instance = new SetTest();
			expect(instance).to.be.ok();
		});

		it("should apply with mixed number and string values", () => {
			class SetTest {
				@Set("easy", "normal", "hard")
				public difficulty: string;
			}
			const instance = new SetTest();
			expect(instance).to.be.ok();
		});

		it("should accept single value", () => {
			class SetTest {
				@Set("constant")
				public constantValue: string;
			}
			const instance = new SetTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Index Decorator", () => {
		it("should apply with field name", () => {
			class IndexTest {
				@Index("itemId")
				public items: unknown[];
			}
			const instance = new IndexTest();
			expect(instance).to.be.ok();
		});

		it("should accept multi-word field names", () => {
			class IndexTest {
				@Index("itemId")
				public drops: unknown[];
			}
			const instance = new IndexTest();
			expect(instance).to.be.ok();
		});
	});

	describe("@Nominal Decorator", () => {
		it("should apply to any field type", () => {
			class NominalTest {
				@Nominal()
				public nominalField: string;
			}
			const instance = new NominalTest();
			expect(instance).to.be.ok();
		});

		it("should work with number fields", () => {
			class NominalTest {
				@Nominal()
				public nominalNumber: number;
			}
			const instance = new NominalTest();
			expect(instance).to.be.ok();
		});
	});

	describe("Stacked Decorators", () => {
		it("should apply multiple decorators to same field", () => {
			class StackedTest {
				@Ref(TestRefTarget)
				@Required()
				@Range(1, 9999)
				public itemId: number;
			}
			const instance = new StackedTest();
			expect(instance).to.be.ok();
		});

		it("should work with Ref and Required", () => {
			class StackedTest {
				@Ref(TestRefTarget)
				@Required()
				public targetId: number;
			}
			const instance = new StackedTest();
			expect(instance).to.be.ok();
		});

		it("should work with Size and Index", () => {
			class StackedTest {
				@Size(1, 10)
				@Index("itemId")
				public items: unknown[];
			}
			const instance = new StackedTest();
			expect(instance).to.be.ok();
		});

		it("should work with Ref, Size, and Index together", () => {
			class StackedTest {
				@Ref(TestRefTarget)
				@Size(1, 5)
				@Index("targetId")
				public targets: { targetId: number }[];
			}
			const instance = new StackedTest();
			expect(instance).to.be.ok();
		});

		it("should work with Required and Range", () => {
			class StackedTest {
				@Required()
				@Range(1, 100)
				public level: number;
			}
			const instance = new StackedTest();
			expect(instance).to.be.ok();
		});
	});

	describe("Complete Class Example", () => {
		it("should compile class with all decorator types", () => {
			@LubanTable({ mode: "map", index: "id" })
			class CompleteExample {
				public id: number;

				@Required()
				public name: string;

				@Range(1, 100)
				public level: number;

				@Set("warrior", "mage", "archer")
				public classType: string;

				@Size(3, 5)
				@Ref(TestRefTarget)
				public skills: number[];

				@Index("itemId")
				public inventory: { itemId: number }[];

				@Nominal()
				public nominalId: number;
			}
			const instance = new CompleteExample();
			expect(instance).to.be.ok();
		});
	});
};
