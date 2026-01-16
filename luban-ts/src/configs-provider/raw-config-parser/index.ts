import { Methods, DeserializerBean, TableMeta, InitTypes } from "../../types/configs/schema";
import { AllTables } from "../../types/configs/tables";
import { createLoaderMethods } from "./loader-methods";
import { loadTableData } from "./table-loader";


/**
 * 解析原始配置表为最终数据结构
 */
export class RawConfigParser{

    private readonly methods:Methods
    private readonly beans: Map<string,DeserializerBean>
    private readonly tables: TableMeta[]
    private readonly tableMetaMap: Map<string,TableMeta> = new Map();

    constructor(){
        this.methods = createLoaderMethods();
        const { beans, tables } = InitTypes(this.methods);
        this.beans = beans;
        this.tables = tables;
        for (const tableMeta of tables) {
            this.tableMetaMap.set(tableMeta.file, tableMeta);
        }
    }

    getMetadata(fileName:string): TableMeta | undefined {
        return this.tableMetaMap.get(fileName);
    }


    /**
     * 解析单个原始配置数据
     * @param fileName - 配置文件名
     * @param rawConfig - 原始配置数据
     */
    parseRawConfig<T = object>(fileName:string, rawConfig:unknown): T {
        const tableMeta = this.tableMetaMap.get(fileName);
        if (!tableMeta) {
            error(`No table meta found for file: ${fileName}`);
        }

        if(rawConfig === undefined){
            error(`No raw config found for file: ${fileName}`);
        }

        const tableData = loadTableData(tableMeta, rawConfig, this.beans);
        return tableData as T;
    }


    /**
     * 解析原始配置数据
     * @param rawConfigs - 原始配置数据, key 为文件名, value 为文件内容 
     * @returns 
     */
    parseRawConfigs(rawConfigs:Map<string,unknown>): AllTables {
        const result: Record<string, any> = {};

        for (const tableMeta of this.tables) {
            const tableData = loadTableData(
                tableMeta,
                rawConfigs.get(tableMeta.file),
                this.beans
            );

            result[tableMeta.name] = tableData;
        }

        return result as AllTables;
    }

}



