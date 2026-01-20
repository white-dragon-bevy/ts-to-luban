import { TableMeta, DeserializerBean } from "../../types/configs/schema";




/**
 * Load and deserialize table data
 * @param tableMeta - Table metadata from Lua InitTypes
 * @param rawData - Raw configuration data
 * @param beans - Beans dictionary with _deserialize functions
 * @returns Deserialized table data (Map/Array/Object based on mode), or undefined if rawData is undefined
 */
export function loadTableData(
	tableMeta: TableMeta,
	rawData: unknown,
	beans: Map<string, DeserializerBean>,
) {
	const beanClass = beans.get(tableMeta.value_type);

	if (!beanClass || !beanClass._deserialize) {
		error(`Bean deserializer not found: ${tableMeta.value_type}`);
	}

	if (rawData === undefined) {
		return undefined;
	}

	if (typeOf(rawData) === "nil") {
		error(`Config data is nil for table: ${tableMeta.name}`);
	}

	switch (tableMeta.mode) {
		case "map": {
			const result = new Map<string,unknown>();
			// Ensure rawData is an object
			if (typeOf(rawData) !== "table") {
				error(`Expected table for map mode, got ${typeOf(rawData)} for ${tableMeta.name}`);
			}

			for (const [key, value] of pairs(rawData as Record<string, unknown>)) {
				const instance = beanClass._deserialize(value);
				result.set(key, instance);
			}
			return result;
		}

		case "array" :
		case "list" : {
			// Ensure rawData is an array
			if (typeOf(rawData) !== "table") {
				error(`Expected table for list mode, got ${typeOf(rawData)} for ${tableMeta.name}`);
			}

			const dataArray = rawData as unknown[];
			const result = [];
			for (const item of dataArray) {
				result.push(beanClass._deserialize(item));
			}
			return result;
		}

		case "one":
		case "single" :
		case "singleton": {
			// Ensure rawData is an object
			if (typeOf(rawData) !== "table") {
				error(`Expected table for singleton mode, got ${typeOf(rawData)} for ${tableMeta.name}`);
			}

			return beanClass._deserialize(rawData);
		}

		default:
			error(`Unknown table mode: ${tableMeta.mode}`);
	}
}
