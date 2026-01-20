/**
 * Loader Methods for Luban Runtime
 *
 * Provides methods object for Lua's InitTypes function:
 * - getClass: Retrieves TypeScript class from Beans dictionary
 * - readList/readArray/readSet/readMap: Collection deserializers
 */

import { Beans } from "../../types/configs/beans";
import { Methods } from "../../types/configs/schema";

export function createLoaderMethods() : Methods {
	return {
		/**
		 * Get TypeScript class constructor from Beans dictionary
		 * @param beanName - Bean name in format "module.ClassName" (e.g., "examples.Item")
		 * @returns TypeScript class constructor or empty table for interfaces
		 */
		getClass: (beanName: string) => {
			const ClassConstructor = Beans[beanName as keyof typeof Beans];
			if (!ClassConstructor) {
				// Return empty table for interfaces (they don't have class constructors)
				return {};
			}
			// Return class constructor directly for Lua to use with setmetatable
			return ClassConstructor;
		},

		/**
		 * Deserialize list/array data
		 * @param data - Array of raw data
		 * @param deserializer - Deserializer function for each item
		 * @returns Deserialized array
		 */
		readList: (data: Record<string, unknown>[], deserializer: (item: unknown) => unknown) => {
			const result = [];
			for (const item of data) {
				result.push(deserializer(item));
			}
			return result;
		},

		/**
		 * Deserialize set data
		 * @param data - Array of raw data
		 * @param deserializer - Deserializer function for each item
		 * @returns Deserialized Set
		 */
		readSet: (data: object[], deserializer: (item: unknown) => unknown) => {
			const result = [];
			for (const item of data) {
				result.push(deserializer(item));
			}
			return new Set(result);
		},

		/**
		 * Deserialize map data
		 * @param data - Object with string keys
		 * @param deserializer - Deserializer function for each value
		 * @returns Deserialized Map
		 */
		readMap: (data: Record<string, unknown>, deserializer: (item: unknown) => unknown) => {
			const result = new Map();
			for (const [key, value] of pairs(data)) {
				result.set(key, deserializer(value));
			}
			return result;
		},
	};
}
