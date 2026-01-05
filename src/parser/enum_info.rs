use std::path::PathBuf;

/// Represents an enum variant (member)
#[derive(Debug, Clone)]
pub struct EnumVariant {
    /// Variant name (e.g., "Role")
    pub name: String,
    /// Alias - only from @alias tag, None if not specified
    pub alias: Option<String>,
    /// Value as string (numeric value or original string value for string enums)
    pub value: String,
    /// Optional comment from JSDoc (excludes @alias line)
    pub comment: Option<String>,
}

/// Represents a TypeScript enum
#[derive(Debug, Clone)]
pub struct EnumInfo {
    /// Enum name
    pub name: String,
    /// Optional alias from @alias tag in JSDoc
    pub alias: Option<String>,
    /// Optional comment from JSDoc (excludes @flags and @alias lines)
    pub comment: Option<String>,
    /// Whether this is a string enum (uses tags="string")
    pub is_string_enum: bool,
    /// Whether this is a flags enum (@flags="true")
    pub is_flags: bool,
    /// Enum variants
    pub variants: Vec<EnumVariant>,
    /// Source file path
    pub source_file: String,
    /// File hash for caching
    pub file_hash: String,
    /// Custom output path for this enum
    pub output_path: Option<PathBuf>,
    /// Custom module name for this enum
    pub module_name: Option<String>,
}
