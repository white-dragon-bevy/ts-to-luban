use super::field_info::FieldInfo;

#[derive(Debug, Clone)]
pub struct ClassInfo {
    pub name: String,
    pub comment: Option<String>,
    pub fields: Vec<FieldInfo>,
    pub implements: Vec<String>,
    pub extends: Option<String>,
    pub source_file: String,
    pub file_hash: String,
    pub is_interface: bool,
}
