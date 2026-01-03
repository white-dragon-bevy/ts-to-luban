use super::field_info::FieldInfo;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub comment: Option<String>,
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
}
