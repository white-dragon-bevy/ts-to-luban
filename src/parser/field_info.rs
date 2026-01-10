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
    /// Whether this field is ObjectFactory<T> type
    pub is_object_factory: bool,
    /// Inner type T for ObjectFactory<T>
    pub factory_inner_type: Option<String>,
    /// Original TypeScript type (before mapping)
    pub original_type: String,
}

impl Default for FieldInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            field_type: String::new(),
            comment: None,
            is_optional: false,
            validators: FieldValidators::default(),
            is_object_factory: false,
            factory_inner_type: None,
            original_type: String::new(),
        }
    }
}
