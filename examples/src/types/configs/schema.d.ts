type deserializer = (item: unknown) => unknown

export interface Methods{
    getClass:(beanName: string)=> unknown,
    readList:(cfg:Record<string, unknown>[],deserializer:deserializer)=>object,
    readSet:(cfg:object[],deserializer:deserializer)=>object,
    readMap:(cfg:Record<string, unknown>,deserializer:deserializer)=>object
}


export interface TableMeta {
	name: string;
	file: string;
	mode: "map" | "list" | "one" | "singleton" | "single" | "array";
	index?: string;
	value_type: string;
}


export interface DeserializerBean   {
    _deserialize: (data: unknown) => unknown
}

/** 初始化 */
export function InitTypes(methods:Methods):{
    /** class 类 */
    beans: Map<string, DeserializerBean>,

    /** 表元数据 */
    tables: TableMeta[]

}