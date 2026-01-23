#[derive(Debug, Clone)]
pub enum SizeConstraint {
    Exact(usize),
    Range(usize, usize),
}

#[derive(Debug, Clone, Default)]
pub struct FieldValidators {
    /// @ref JSDoc tag - applies to scalar, list element, or map value
    /// Auto-discovers target table from field type
    pub has_ref: bool,
    /// RefKey<T> generic type - applies to map key only
    /// Set when Map<RefKey<T>, V> pattern is detected
    pub has_ref_key: bool,
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
    /// Optional alias from @alias tag in JSDoc
    pub alias: Option<String>,
    pub is_optional: bool,
    pub validators: FieldValidators,
    /// Whether this field is ObjectFactory<T> type
    pub is_object_factory: bool,
    /// Inner type T for ObjectFactory<T>
    pub factory_inner_type: Option<String>,
    /// Whether this field is Constructor<T> type
    pub is_constructor: bool,
    /// Inner type T for Constructor<T>
    pub constructor_inner_type: Option<String>,
    /// Original TypeScript type (before mapping)
    pub original_type: String,
    /// Default value from @default JSDoc tag (e.g., @default="0")
    pub default_value: Option<String>,
    /// Type override from @type JSDoc tag (e.g., @type="int" to override number -> int)
    pub type_override: Option<String>,
    /// List separator from @sep JSDoc tag (e.g., @sep="|")
    pub separator: Option<String>,
    /// Map separator from @mapsep JSDoc tag (e.g., @mapsep=",|")
    pub map_separator: Option<String>,
    /// Custom tags from @tags JSDoc tag (e.g., @tags="RefOverride=true,foo=bar")
    pub custom_tags: Option<String>,
    /// Inner type T for RefKey<T> in Map<RefKey<T>, V> - used for map key ref resolution
    pub ref_key_inner_type: Option<String>,
}

impl Default for FieldInfo {
    fn default() -> Self {
        Self {
            name: String::new(),
            field_type: String::new(),
            comment: None,
            alias: None,
            is_optional: false,
            validators: FieldValidators::default(),
            is_object_factory: false,
            factory_inner_type: None,
            is_constructor: false,
            constructor_inner_type: None,
            original_type: String::new(),
            default_value: None,
            type_override: None,
            separator: None,
            map_separator: None,
            custom_tags: None,
            ref_key_inner_type: None,
        }
    }
}
