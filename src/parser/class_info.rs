use super::field_info::FieldInfo;
use std::collections::HashMap;
use std::path::PathBuf;

/// Import information: maps imported type name to the resolved source file path
pub type ImportMap = HashMap<String, PathBuf>;

/// Configuration for Luban table generation from @luban-table decorator
#[derive(Debug, Clone, Default)]
pub struct LubanTableConfig {
    pub mode: String,
    pub index: String,
    pub group: Option<String>,
    pub tags: Option<String>,
    /// Custom table name from table_mappings config (overrides default {ClassName}Table)
    pub table_name: Option<String>,
}

/// Configuration for Luban table from @table JSDoc tag
#[derive(Debug, Clone)]
pub struct JsDocTableConfig {
    pub mode: String,
    pub index: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub comment: Option<String>,
    /// Optional alias from @alias tag in JSDoc
    pub alias: Option<String>,
    pub fields: Vec<FieldInfo>,
    #[allow(dead_code)]
    pub implements: Vec<String>,
    pub extends: Option<String>,
    pub source_file: String,
    pub file_hash: String,
    pub is_interface: bool,
    /// Custom output path for this class (overrides default output)
    pub output_path: Option<PathBuf>,
    /// Custom module name for this class (overrides default module_name)
    pub module_name: Option<String>,
    /// Generic type parameters mapping: T -> ConstraintType
    /// e.g., {"T": "SkillMetadata", "K": "string"}
    #[allow(dead_code)]
    pub type_params: HashMap<String, String>,
    /// Luban table configuration from @luban-table decorator
    pub luban_table: Option<LubanTableConfig>,
    /// Table configuration from @table JSDoc tag (e.g., @table="map,id")
    pub table_config: Option<JsDocTableConfig>,
    /// Input path from @input JSDoc tag (e.g., @input="../datas/skill")
    pub input_path: Option<String>,
    /// Import mappings: type_name -> resolved source file path
    /// Used for cross-module type resolution when same-named types exist in different modules
    pub imports: ImportMap,
}
