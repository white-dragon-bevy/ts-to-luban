#[derive(Debug, Clone)]
pub enum SizeConstraint {
    Exact(usize),
    Range(usize, usize),
}

#[derive(Debug, Clone, Default)]
pub struct FieldValidators {
    pub ref_target: Option<String>,
    pub range: Option<(f64, f64)>,
    pub required: bool,
    pub size: Option<SizeConstraint>,
    pub set_values: Vec<String>,
    pub index_field: Option<String>,
    pub nominal: bool,
}

#[derive(Debug, Clone)]
pub struct FieldInfo {
    pub name: String,
    pub field_type: String,
    pub comment: Option<String>,
    pub is_optional: bool,
    pub validators: FieldValidators,
}
