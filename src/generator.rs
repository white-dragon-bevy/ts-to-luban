use crate::parser::field_info::SizeConstraint;
use crate::parser::{ClassInfo, EnumInfo, FieldInfo, FieldValidators, ImportMap};
use crate::table_registry::{ResolvedTableConfig, TableRegistry};
use crate::type_mapper::TypeMapper;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct XmlGenerator<'a> {
    type_mapper: &'a TypeMapper,
    table_registry: &'a TableRegistry,
    /// Mapping from type name to module name (for cross-module type resolution)
    type_to_module: HashMap<String, String>,
    /// Mapping from source file path to module name (for import-based type resolution)
    file_to_module: HashMap<PathBuf, String>,
}

impl<'a> XmlGenerator<'a> {
    pub fn new(type_mapper: &'a TypeMapper, table_registry: &'a TableRegistry) -> Self {
        Self {
            type_mapper,
            table_registry,
            type_to_module: HashMap::new(),
            file_to_module: HashMap::new(),
        }
    }

    /// Create a new XmlGenerator with a pre-built type-to-module mapping
    pub fn with_type_mapping(
        type_mapper: &'a TypeMapper,
        table_registry: &'a TableRegistry,
        type_to_module: HashMap<String, String>,
    ) -> Self {
        Self {
            type_mapper,
            table_registry,
            type_to_module,
            file_to_module: HashMap::new(),
        }
    }

    /// Create a new XmlGenerator with both type-to-module and file-to-module mappings
    pub fn with_type_and_file_mapping(
        type_mapper: &'a TypeMapper,
        table_registry: &'a TableRegistry,
        type_to_module: HashMap<String, String>,
        file_to_module: HashMap<PathBuf, String>,
    ) -> Self {
        Self {
            type_mapper,
            table_registry,
            type_to_module,
            file_to_module,
        }
    }

    pub fn generate(&self, classes: &[ClassInfo], module_name: &str) -> String {
        // For backward compatibility, use classes as all_classes
        self.generate_with_all_classes(classes, module_name, classes)
    }

    pub fn generate_with_all_classes(
        &self,
        classes: &[ClassInfo],
        module_name: &str,
        all_classes: &[ClassInfo],
    ) -> String {
        // For backward compatibility, call with empty enums
        self.generate_with_all_classes_and_enums(classes, &[], module_name, all_classes)
    }

    pub fn generate_with_all_classes_and_enums(
        &self,
        classes: &[ClassInfo],
        enums: &[EnumInfo],
        module_name: &str,
        all_classes: &[ClassInfo],
    ) -> String {
        // Determine comment based on content
        let has_beans = !classes.is_empty();
        let has_enums = !enums.is_empty();
        let comment = if has_beans {
            "自动生成的 ts class Bean 定义"
        } else if has_enums {
            "自动生成的 ts enum 定义"
        } else {
            "自动生成的定义"
        };

        let mut lines = vec![
            format!(
                r#"<module name="{}" comment="{}">"#,
                escape_xml(module_name),
                comment
            ),
            String::new(),
        ];

        // Build class name -> module name mapping from all_classes
        // and merge with self.type_to_module (which includes enums)
        let mut class_to_module: HashMap<String, String> = self.type_to_module.clone();
        for c in all_classes {
            if let Some(m) = &c.module_name {
                class_to_module.insert(c.name.clone(), m.clone());
            }
        }

        // Generate enums first (before beans)
        if !enums.is_empty() {
            // Keep original source file order (no sorting)
            let sorted_enums: Vec<_> = enums.iter().collect();

            for enum_info in sorted_enums {
                generate_enum(&mut lines, enum_info);
                lines.push(String::new());
            }
        }

        // Deduplicate classes by name, prioritizing @LubanTable classes
        // Use a Vec to preserve insertion order
        let mut seen_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        let mut unique_classes: Vec<&ClassInfo> = Vec::new();

        // First pass: collect classes with @LubanTable
        for class in classes {
            if class.luban_table.is_some() && !seen_names.contains(&class.name) {
                seen_names.insert(class.name.clone());
                unique_classes.push(class);
            }
        }

        // Second pass: collect remaining classes (without @LubanTable)
        for class in classes {
            if !seen_names.contains(&class.name) {
                seen_names.insert(class.name.clone());
                unique_classes.push(class);
            }
        }
        // Keep original source file order (no sorting)

        // Generate beans
        for class in &unique_classes {
            self.generate_bean_with_module_map(&mut lines, class, all_classes, module_name, &class_to_module);
            lines.push(String::new());
        }

        // Generate tables from [tables] config in registry
        // Look up each class by its full name (module.ClassName) in the registry
        let table_entries: Vec<&ResolvedTableConfig> = classes
            .iter()
            .filter_map(|class| {
                // Build full name: module.ClassName
                let full_name = if module_name.is_empty() {
                    class.name.clone()
                } else {
                    format!("{}.{}", module_name, class.name)
                };
                self.table_registry.get_table(&full_name)
            })
            .collect();

        if !table_entries.is_empty() {
            for table_config in &table_entries {
                self.generate_table_from_config(&mut lines, table_config);
            }
            lines.push(String::new());
        }

        lines.push("</module>".to_string());
        lines.join("\n") + "\n"
    }

    /// Generate table element from ResolvedTableConfig
    fn generate_table_from_config(&self, lines: &mut Vec<String>, config: &ResolvedTableConfig) {
        let mut attrs = vec![
            format!(r#"name="{}""#, config.name),
            format!(r#"value="{}""#, config.class_name),
        ];

        // Only add mode and index if mode is not "map" (map is the default)
        if config.mode != "map" {
            attrs.push(format!(r#"mode="{}""#, config.mode));
        }

        // Always add index for map mode
        if config.mode == "map" {
            attrs.push(format!(r#"index="{}""#, config.index));
        }

        attrs.push(format!(r#"input="{}""#, config.input));

        lines.push(format!(r#"    <table {} />"#, attrs.join(" ")));
    }

    fn generate_bean(&self, lines: &mut Vec<String>, class: &ClassInfo, all_classes: &[ClassInfo]) {
        // For backward compatibility, use empty module map
        let class_to_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        self.generate_bean_with_module_map(lines, class, all_classes, "", &class_to_module);
    }

    fn generate_bean_with_module_map(
        &self,
        lines: &mut Vec<String>,
        class: &ClassInfo,
        all_classes: &[ClassInfo],
        current_module: &str,
        class_to_module: &std::collections::HashMap<String, String>,
    ) {
        let parent = if class.is_interface {
            // Interface: no parent if no extends (not affected by the change)
            class.extends.clone().unwrap_or_default()
        } else {
            // Class: resolve parent based on extends, implements, or default to TsClass
            self.resolve_class_parent(class, all_classes)
        };

        // Resolve parent with module prefix if needed, using imports for accurate resolution
        let resolved_parent = self.resolve_type_with_imports(&parent, current_module, class_to_module, &class.imports);

        let alias_attr = class
            .alias
            .as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        let comment_attr = class
            .comment
            .as_ref()
            .filter(|c| !c.is_empty())
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let parent_attr = if resolved_parent.is_empty() {
            String::new()
        } else {
            format!(r#" parent="{}""#, resolved_parent)
        };

        // Add XML comment before bean if comment exists
        if let Some(comment) = &class.comment {
            if !comment.is_empty() {
                lines.push(format!("    <!-- {} -->", escape_xml(comment)));
            }
        }

        lines.push(format!(
            r#"    <bean name="{}"{}{}{}>"#,
            class.name, alias_attr, parent_attr, comment_attr
        ));

        // Collect parent field names to skip redeclared fields
        // Note: we need to look up parent by simple name, not qualified name
        let mut parent_field_names = std::collections::HashSet::new();
        let mut current_parent = if parent.is_empty() {
            None
        } else {
            Some(parent.as_str())
        };
        while let Some(parent_name) = current_parent {
            if let Some(parent) = all_classes.iter().find(|c| &c.name == parent_name) {
                for field in &parent.fields {
                    parent_field_names.insert(field.name.as_str());
                }
                current_parent = parent.extends.as_ref().map(|s| s.as_str());
            } else {
                break;
            }
        }

        // Only generate fields that are not redeclared from parent classes
        // Skip $type field (used for TypeScript discriminated unions, not needed in Luban)
        for field in &class.fields {
            if !parent_field_names.contains(field.name.as_str()) && field.name != "$type" {
                self.generate_field_with_imports(lines, field, current_module, class_to_module, &class.imports);
            }
        }

        lines.push("    </bean>".to_string());
    }

    /// Resolve a type name with module prefix if it's from a different module
    fn resolve_type_with_module(
        &self,
        type_name: &str,
        current_module: &str,
        class_to_module: &HashMap<String, String>,
    ) -> String {
        // Use the new import-aware resolution with empty imports (for backward compatibility)
        self.resolve_type_with_imports(type_name, current_module, class_to_module, &ImportMap::new())
    }

    /// Resolve a type name with module prefix, using imports to determine the correct module
    /// This handles the case where same-named types exist in different modules
    fn resolve_type_with_imports(
        &self,
        type_name: &str,
        current_module: &str,
        class_to_module: &HashMap<String, String>,
        imports: &ImportMap,
    ) -> String {
        if type_name.is_empty() {
            return String::new();
        }

        // First, check if this type was imported from a specific file
        if let Some(import_source_path) = imports.get(type_name) {
            // Look up the module for this source file
            if let Some(target_module) = self.file_to_module.get(import_source_path) {
                // If the imported type is from a different module, add the module prefix
                if target_module != current_module && !target_module.is_empty() {
                    return format!("{}.{}", target_module, type_name);
                }
                // Same module - no prefix needed
                return type_name.to_string();
            }
        }

        // Fall back to the global class_to_module mapping
        if let Some(target_module) = class_to_module.get(type_name) {
            // If the type is from a different module, add the module prefix
            if target_module != current_module && !target_module.is_empty() {
                return format!("{}.{}", target_module, type_name);
            }
        }

        // Return the type name as-is (same module or not in mapping)
        type_name.to_string()
    }

    /// Resolves the parent for a class based on:
    /// 1. Extends keyword (highest priority)
    /// 2. Single implements interface (only when no extends)
    /// 3. No parent (empty string) when no extends and no/multiple implements
    fn resolve_class_parent(&self, class: &ClassInfo, _all_classes: &[ClassInfo]) -> String {
        // Priority 1: Use extends if present
        if let Some(extends) = &class.extends {
            return extends.clone();
        }

        // Priority 2: Use single implements if present
        if class.implements.len() == 1 {
            return class.implements[0].clone();
        }

        // Priority 3: No parent
        String::new()
    }

    fn generate_field(&self, lines: &mut Vec<String>, field: &FieldInfo) {
        // For backward compatibility, use empty module map
        let class_to_module: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        self.generate_field_with_module_map(lines, field, "", &class_to_module);
    }

    fn generate_field_with_module_map(
        &self,
        lines: &mut Vec<String>,
        field: &FieldInfo,
        current_module: &str,
        class_to_module: &std::collections::HashMap<String, String>,
    ) {
        // For backward compatibility, use empty imports
        self.generate_field_with_imports(lines, field, current_module, class_to_module, &ImportMap::new());
    }

    fn generate_field_with_imports(
        &self,
        lines: &mut Vec<String>,
        field: &FieldInfo,
        current_module: &str,
        class_to_module: &std::collections::HashMap<String, String>,
        imports: &ImportMap,
    ) {
        // Handle @RefReplace<T, "field">() decorator
        // Generates type from T's index_type and tags="RefOverride=field"
        if let Some((ref_type, ref_field)) = &field.ref_replace {
            // Look up the table for ref_type to get index_type and table name
            if let Some(table_config) = self.table_registry.get_table_by_class(ref_type) {
                let index_type = table_config.index_type.clone().unwrap_or_else(|| "int".to_string());
                let table_ref = if table_config.module.is_empty() {
                    table_config.name.clone()
                } else {
                    format!("{}.{}", table_config.module, table_config.name)
                };

                let mut final_type = index_type;
                if field.is_optional {
                    final_type.push('?');
                }
                final_type.push_str(&format!("#ref={}", table_ref));

                let comment_attr = field
                    .comment
                    .as_ref()
                    .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
                    .unwrap_or_default();

                let alias_attr = field
                    .alias
                    .as_ref()
                    .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
                    .unwrap_or_default();

                // Build tags: RefOverride=field_name + custom_tags
                let tags_attr = {
                    let mut tags = vec![format!("RefOverride={}", ref_field)];
                    if let Some(custom) = &field.custom_tags {
                        tags.push(custom.clone());
                    }
                    format!(r#" tags="{}""#, escape_xml(&tags.join(",")))
                };

                lines.push(format!(
                    r#"        <var name="{}" type="{}"{}{}{}/>"#,
                    field.name, final_type, alias_attr, comment_attr, tags_attr
                ));
                return;
            }
        }

        // Handle Constructor<T> fields
        if field.is_constructor {
            if let Some(constructor_type) = &field.constructor_inner_type {
                let resolved_constructor_type = self.resolve_type_with_imports(constructor_type, current_module, class_to_module, imports);
                let mut final_type = String::from("string");
                if field.is_optional {
                    final_type.push('?');
                }
                final_type.push_str(&format!("#constructor={}", resolved_constructor_type));

                let comment_attr = field
                    .comment
                    .as_ref()
                    .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
                    .unwrap_or_default();

                let alias_attr = field
                    .alias
                    .as_ref()
                    .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
                    .unwrap_or_default();

                // Build tags: custom_tags only
                let tags_attr = field
                    .custom_tags
                    .as_ref()
                    .map(|t| format!(r#" tags="{}""#, escape_xml(t)))
                    .unwrap_or_default();

                lines.push(format!(
                    r#"        <var name="{}" type="{}"{}{}{}/>"#,
                    field.name, final_type, alias_attr, comment_attr, tags_attr
                ));
                return;
            }
        }

        // Apply @type override if present, otherwise use mapped type
        let mut mapped_type = if let Some(type_override) = &field.type_override {
            type_override.clone()
        } else {
            self.type_mapper.map_full_type(&field.field_type)
        };
        let validators = &field.validators;

        // @Set only supports int/long/string/enum, not double
        // When @Set is present and type is double, convert to int
        if !validators.set_values.is_empty() && mapped_type == "double" {
            mapped_type = "int".to_string();
        }

        // Resolve type references with module prefix, using imports for accurate resolution
        mapped_type = self.resolve_full_type_with_imports(&mapped_type, current_module, class_to_module, imports);

        // Check if this is a container type (list, map, array, set)
        let is_container = mapped_type.starts_with("list,")
            || mapped_type.starts_with("map,")
            || mapped_type.starts_with("array,")
            || mapped_type.starts_with("set,");

        let final_type = if is_container {
            // Handle container types with size/index validators and separators
            // Note: mapped_type is already resolved with module prefixes
            self.apply_container_validators_with_module_and_separators(
                &mapped_type,
                validators,
                current_module,
                class_to_module,
                field.separator.as_deref(),
                field.map_separator.as_deref(),
                field.default_value.as_deref(),
            )
        } else {
            // Handle scalar types with validators and default value
            self.apply_scalar_validators_with_default(
                &mapped_type,
                validators,
                field.is_optional,
                field.default_value.as_deref(),
            )
        };

        let comment_attr = field
            .comment
            .as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let alias_attr = field
            .alias
            .as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        // Build tags: RefOverride (auto for @ref JSDoc tag only) + ObjectFactory + custom_tags
        let tags_attr = {
            let mut tags = Vec::new();

            // Auto-add RefOverride=true when @ref JSDoc tag is present
            // NOT when RefKey<T> generic type is used
            if field.validators.has_ref {
                tags.push("RefOverride=true");
            }

            if field.is_object_factory {
                tags.push("ObjectFactory=true");
            }

            if let Some(custom) = &field.custom_tags {
                tags.push(custom.as_str());
            }

            if tags.is_empty() {
                String::new()
            } else {
                format!(r#" tags="{}""#, escape_xml(&tags.join(",")))
            }
        };

        lines.push(format!(
            r#"        <var name="{}" type="{}"{}{}{}/>"#,
            field.name, final_type, alias_attr, comment_attr, tags_attr
        ));
    }

    /// Resolve a full type string (including list,T and map,K,V) with module prefixes
    fn resolve_full_type_with_module(
        &self,
        type_str: &str,
        current_module: &str,
        class_to_module: &std::collections::HashMap<String, String>,
    ) -> String {
        // Handle list,T
        if type_str.starts_with("list,") {
            let element = &type_str[5..];
            let resolved_element = self.resolve_type_with_module(element, current_module, class_to_module);
            return format!("list,{}", resolved_element);
        }

        // Handle map,K,V
        if type_str.starts_with("map,") {
            let parts: Vec<&str> = type_str[4..].splitn(2, ',').collect();
            if parts.len() == 2 {
                let resolved_key = self.resolve_type_with_module(parts[0], current_module, class_to_module);
                let resolved_value = self.resolve_type_with_module(parts[1], current_module, class_to_module);
                return format!("map,{},{}", resolved_key, resolved_value);
            }
        }

        // Handle array,T
        if type_str.starts_with("array,") {
            let element = &type_str[6..];
            let resolved_element = self.resolve_type_with_module(element, current_module, class_to_module);
            return format!("array,{}", resolved_element);
        }

        // Handle set,T
        if type_str.starts_with("set,") {
            let element = &type_str[4..];
            let resolved_element = self.resolve_type_with_module(element, current_module, class_to_module);
            return format!("set,{}", resolved_element);
        }

        // Simple type
        self.resolve_type_with_module(type_str, current_module, class_to_module)
    }

    /// Resolve a full type string (including list,T and map,K,V) with module prefixes, using imports
    fn resolve_full_type_with_imports(
        &self,
        type_str: &str,
        current_module: &str,
        class_to_module: &std::collections::HashMap<String, String>,
        imports: &ImportMap,
    ) -> String {
        // Handle list,T
        if type_str.starts_with("list,") {
            let element = &type_str[5..];
            let resolved_element = self.resolve_type_with_imports(element, current_module, class_to_module, imports);
            return format!("list,{}", resolved_element);
        }

        // Handle map,K,V
        if type_str.starts_with("map,") {
            let parts: Vec<&str> = type_str[4..].splitn(2, ',').collect();
            if parts.len() == 2 {
                let resolved_key = self.resolve_type_with_imports(parts[0], current_module, class_to_module, imports);
                let resolved_value = self.resolve_type_with_imports(parts[1], current_module, class_to_module, imports);
                return format!("map,{},{}", resolved_key, resolved_value);
            }
        }

        // Handle array,T
        if type_str.starts_with("array,") {
            let element = &type_str[6..];
            let resolved_element = self.resolve_type_with_imports(element, current_module, class_to_module, imports);
            return format!("array,{}", resolved_element);
        }

        // Handle set,T
        if type_str.starts_with("set,") {
            let element = &type_str[4..];
            let resolved_element = self.resolve_type_with_imports(element, current_module, class_to_module, imports);
            return format!("set,{}", resolved_element);
        }

        // Simple type
        self.resolve_type_with_imports(type_str, current_module, class_to_module, imports)
    }

    /// Apply validators to scalar types
    /// e.g., "int" -> "int!#ref=examples.TbItem#range=[1,100]"
    /// For @ref fields, the type is replaced with the target table's index type
    fn apply_scalar_validators(
        &self,
        base_type: &str,
        validators: &FieldValidators,
        is_optional: bool,
    ) -> String {
        // Handle @ref - replace type with target table's index type
        let effective_type = if validators.has_ref {
            let type_name = base_type.split('.').last().unwrap_or(base_type);
            if let Some(table_config) = self.table_registry.get_table_by_class(type_name) {
                table_config.index_type.clone().unwrap_or_else(|| base_type.to_string())
            } else {
                base_type.to_string()
            }
        } else {
            base_type.to_string()
        };

        let mut result = effective_type;

        // Add optional marker
        if is_optional {
            result.push('?');
        }

        // Add required marker (!) - notDefaultValue validator
        if validators.required {
            result.push('!');
        }

        // Collect validator suffixes
        let mut validator_parts = Vec::new();

        // Handle @ref - auto-discover target table from base_type
        if validators.has_ref {
            let type_name = base_type.split('.').last().unwrap_or(base_type);
            if let Some(table_config) = self.table_registry.get_table_by_class(type_name) {
                let table_ref = if table_config.module.is_empty() {
                    table_config.name.clone()
                } else {
                    format!("{}.{}", table_config.module, table_config.name)
                };
                validator_parts.push(format!("ref={}", table_ref));
            }
        }

        // Handle range
        if let Some((min, max)) = &validators.range {
            // Format numbers nicely (remove unnecessary decimal points)
            let min_str = format_number(*min);
            let max_str = format_number(*max);
            validator_parts.push(format!("range=[{},{}]", min_str, max_str));
        }

        // Handle set
        if !validators.set_values.is_empty() {
            let set_str = validators.set_values.join(",");
            validator_parts.push(format!("set={}", set_str));
        }

        // Append validators with # prefix
        if !validator_parts.is_empty() {
            result.push_str(&format!("#{}", validator_parts.join("#")));
        }

        result
    }

    /// Apply validators to scalar types with default value support
    /// e.g., "int" -> "int!#ref=examples.TbItem#range=[1,100]#default=0"
    fn apply_scalar_validators_with_default(
        &self,
        base_type: &str,
        validators: &FieldValidators,
        is_optional: bool,
        default_value: Option<&str>,
    ) -> String {
        // Handle @ref or RefKey<T> - replace type with target table's index type
        let has_any_ref = validators.has_ref || validators.has_ref_key;
        let effective_type = if has_any_ref {
            let type_name = base_type.split('.').last().unwrap_or(base_type);
            if let Some(table_config) = self.table_registry.get_table_by_class(type_name) {
                table_config.index_type.clone().unwrap_or_else(|| base_type.to_string())
            } else {
                base_type.to_string()
            }
        } else {
            base_type.to_string()
        };

        let mut result = effective_type;

        // Add optional marker
        if is_optional {
            result.push('?');
        }

        // Add required marker (!) - notDefaultValue validator
        if validators.required {
            result.push('!');
        }

        // Collect validator suffixes
        let mut validator_parts = Vec::new();

        // Handle @ref or RefKey<T> - auto-discover target table from base_type
        if has_any_ref {
            let type_name = base_type.split('.').last().unwrap_or(base_type);
            if let Some(table_config) = self.table_registry.get_table_by_class(type_name) {
                let table_ref = if table_config.module.is_empty() {
                    table_config.name.clone()
                } else {
                    format!("{}.{}", table_config.module, table_config.name)
                };
                validator_parts.push(format!("ref={}", table_ref));
            }
        }

        // Handle range
        if let Some((min, max)) = &validators.range {
            // Format numbers nicely (remove unnecessary decimal points)
            let min_str = format_number(*min);
            let max_str = format_number(*max);
            validator_parts.push(format!("range=[{},{}]", min_str, max_str));
        }

        // Handle set
        if !validators.set_values.is_empty() {
            let set_str = validators.set_values.join(",");
            validator_parts.push(format!("set={}", set_str));
        }

        // Handle default value
        if let Some(default) = default_value {
            validator_parts.push(format!("default={}", default));
        }

        // Append validators with # prefix
        if !validator_parts.is_empty() {
            result.push_str(&format!("#{}", validator_parts.join("#")));
        }

        result
    }

    /// Apply validators to container types with module resolution and separator support
    /// e.g., "list,string" -> "(list#sep=|),string" or "map,string,int" -> "(map#sep=,|),string,int"
    fn apply_container_validators_with_module_and_separators(
        &self,
        container_type: &str,
        validators: &FieldValidators,
        _current_module: &str,
        _class_to_module: &std::collections::HashMap<String, String>,
        separator: Option<&str>,
        map_separator: Option<&str>,
        default_value: Option<&str>,
    ) -> String {
        // Parse container type: "list,ElementType" or "map,KeyType,ValueType"
        // Note: ElementType may already have module prefix like "enums.QualityType"
        let parts: Vec<&str> = container_type.splitn(2, ',').collect();
        if parts.len() < 2 {
            return container_type.to_string();
        }

        let container = parts[0];
        let rest = parts[1]; // For list: "ElementType", for map: "KeyType,ValueType"

        // Build container validators
        let mut container_mods = Vec::new();

        // Handle separator for list types
        if container == "list" || container == "array" || container == "set" {
            if let Some(sep) = separator {
                container_mods.push(format!("sep={}", sep));
            }
        }

        // Handle map separator for map types
        if container == "map" {
            if let Some(sep) = map_separator {
                container_mods.push(format!("sep={}", sep));
            }
        }

        if let Some(size) = &validators.size {
            match size {
                SizeConstraint::Exact(n) => container_mods.push(format!("size={}", n)),
                SizeConstraint::Range(min, max) => {
                    container_mods.push(format!("size=[{},{}]", min, max))
                }
            }
        }

        if let Some(index) = &validators.index_field {
            container_mods.push(format!("index={}", index));
        }

        // Handle map type separately - need to process key and value types
        if container == "map" {
            // Split rest into key and value: "KeyType,ValueType"
            let kv_parts: Vec<&str> = rest.splitn(2, ',').collect();
            if kv_parts.len() == 2 {
                let key_type = kv_parts[0];
                let value_type = kv_parts[1];

                // Apply @refKey to key type
                let key_validators = FieldValidators {
                    has_ref: validators.has_ref_key, // @refKey applies to key
                    has_ref_key: false,
                    range: None,
                    required: false,
                    set_values: vec![],
                    size: None,
                    index_field: None,
                    nominal: false,
                };
                let key_with_validators =
                    self.apply_scalar_validators_with_default(key_type, &key_validators, false, None);

                // Apply @ref to value type
                let value_validators = FieldValidators {
                    has_ref: validators.has_ref, // @ref applies to value
                    has_ref_key: false,
                    range: validators.range,
                    required: validators.required,
                    set_values: validators.set_values.clone(),
                    size: None,
                    index_field: None,
                    nominal: validators.nominal,
                };
                let value_with_validators =
                    self.apply_scalar_validators_with_default(value_type, &value_validators, false, None);

                // Build the final type string for map
                let mut result = if container_mods.is_empty() {
                    format!("map,{},{}", key_with_validators, value_with_validators)
                } else {
                    format!(
                        "(map#{}),{},{}",
                        container_mods.join("#"),
                        key_with_validators,
                        value_with_validators
                    )
                };

                // Append default value at the end if present
                if let Some(default) = default_value {
                    result.push_str(&format!("#default={}", default));
                }

                return result;
            }
        }

        // For list/array/set types
        // Build element type with its validators (ref, range, set, required)
        // Note: has_ref_key for list means RefKey<T>[] or Array<RefKey<T>>
        let element_validators = FieldValidators {
            has_ref: validators.has_ref || validators.has_ref_key, // Both @ref and RefKey<T> apply to element
            has_ref_key: false,
            range: validators.range,
            required: validators.required,
            set_values: validators.set_values.clone(),
            // These are container-level, not element-level
            size: None,
            index_field: None,
            nominal: validators.nominal,
        };

        let element_with_validators =
            self.apply_scalar_validators_with_default(rest, &element_validators, false, None);

        // Build the final type string
        let mut result = if container_mods.is_empty() {
            format!("{},{}", container, element_with_validators)
        } else {
            format!(
                "({}#{}),{}",
                container,
                container_mods.join("#"),
                element_with_validators
            )
        };

        // Append default value at the end if present
        if let Some(default) = default_value {
            result.push_str(&format!("#default={}", default));
        }

        result
    }

    /// Apply validators to container types
    /// e.g., "list,int" -> "(list#size=4#index=id),int#ref=TbItem"
    fn apply_container_validators(
        &self,
        container_type: &str,
        validators: &FieldValidators,
    ) -> String {
        // Parse container type: "list,ElementType" or "map,KeyType,ValueType"
        let parts: Vec<&str> = container_type.splitn(2, ',').collect();
        if parts.len() < 2 {
            return container_type.to_string();
        }

        let container = parts[0];
        let element_type = parts[1];

        // Build container validators
        let mut container_mods = Vec::new();

        if let Some(size) = &validators.size {
            match size {
                SizeConstraint::Exact(n) => container_mods.push(format!("size={}", n)),
                SizeConstraint::Range(min, max) => {
                    container_mods.push(format!("size=[{},{}]", min, max))
                }
            }
        }

        if let Some(index) = &validators.index_field {
            container_mods.push(format!("index={}", index));
        }

        // Build element type with its validators (ref, range, set, required)
        let element_validators = FieldValidators {
            has_ref: validators.has_ref,
            has_ref_key: false,
            range: validators.range,
            required: validators.required,
            set_values: validators.set_values.clone(),
            // These are container-level, not element-level
            size: None,
            index_field: None,
            nominal: validators.nominal,
        };

        let element_with_validators =
            self.apply_scalar_validators(element_type, &element_validators, false);

        if container_mods.is_empty() {
            format!("{},{}", container, element_with_validators)
        } else {
            format!(
                "({}#{}),{}",
                container,
                container_mods.join("#"),
                element_with_validators
            )
        }
    }

    /// Apply validators to container types with module resolution
    /// The container_type is already resolved with module prefixes
    fn apply_container_validators_with_module(
        &self,
        container_type: &str,
        validators: &FieldValidators,
        _current_module: &str,
        _class_to_module: &std::collections::HashMap<String, String>,
    ) -> String {
        // Parse container type: "list,ElementType" or "map,KeyType,ValueType"
        // Note: ElementType may already have module prefix like "enums.QualityType"
        let parts: Vec<&str> = container_type.splitn(2, ',').collect();
        if parts.len() < 2 {
            return container_type.to_string();
        }

        let container = parts[0];
        let element_type = parts[1];

        // Build container validators
        let mut container_mods = Vec::new();

        if let Some(size) = &validators.size {
            match size {
                SizeConstraint::Exact(n) => container_mods.push(format!("size={}", n)),
                SizeConstraint::Range(min, max) => {
                    container_mods.push(format!("size=[{},{}]", min, max))
                }
            }
        }

        if let Some(index) = &validators.index_field {
            container_mods.push(format!("index={}", index));
        }

        // Build element type with its validators (ref, range, set, required)
        // Note: element_type is already resolved with module prefix
        let element_validators = FieldValidators {
            has_ref: validators.has_ref,
            has_ref_key: false,
            range: validators.range,
            required: validators.required,
            set_values: validators.set_values.clone(),
            // These are container-level, not element-level
            size: None,
            index_field: None,
            nominal: validators.nominal,
        };

        let element_with_validators =
            self.apply_scalar_validators(element_type, &element_validators, false);

        if container_mods.is_empty() {
            format!("{},{}", container, element_with_validators)
        } else {
            format!(
                "({}#{}),{}",
                container,
                container_mods.join("#"),
                element_with_validators
            )
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Format a number, removing unnecessary decimal points
/// e.g., 1.0 -> "1", 1.5 -> "1.5"
fn format_number(n: f64) -> String {
    if n.fract() == 0.0 {
        format!("{}", n as i64)
    } else {
        format!("{}", n)
    }
}

/// Generate XML for enums only
pub fn generate_enum_xml(enums: &[EnumInfo], module_name: &str) -> String {
    let mut lines = vec![
        format!(
            r#"<module name="{}" comment="自动生成的 ts enum 定义">"#,
            escape_xml(module_name)
        ),
        String::new(),
    ];

    for enum_info in enums {
        generate_enum(&mut lines, enum_info);
        lines.push(String::new());
    }

    lines.push("</module>".to_string());
    lines.join("\n") + "\n"
}

fn generate_enum(lines: &mut Vec<String>, enum_info: &EnumInfo) {
    let alias_attr = enum_info
        .alias
        .as_ref()
        .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
        .unwrap_or_default();

    let flags_attr = if enum_info.is_flags {
        r#" flags="true""#.to_string()
    } else {
        String::new()
    };

    let tags_attr = enum_info
        .tags
        .as_ref()
        .map(|t| format!(r#" tags="{}""#, escape_xml(t)))
        .unwrap_or_default();

    let comment_attr = enum_info
        .comment
        .as_ref()
        .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
        .unwrap_or_default();

    // Add XML comment before enum if comment exists
    if let Some(comment) = &enum_info.comment {
        if !comment.is_empty() {
            lines.push(format!("    <!-- {} -->", escape_xml(comment)));
        }
    }

    lines.push(format!(
        r#"    <enum name="{}"{}{}{}{}>"#,
        enum_info.name, alias_attr, flags_attr, tags_attr, comment_attr
    ));

    for variant in &enum_info.variants {
        let var_alias_attr = variant
            .alias
            .as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        let var_comment_attr = variant
            .comment
            .as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        // Format: name, alias (if present), value, comment (if present)
        // alias needs a leading space, value needs a leading space after alias or name
        let alias_part = if var_alias_attr.is_empty() {
            " ".to_string()
        } else {
            format!("{} ", var_alias_attr)
        };
        lines.push(format!(
            r#"        <var name="{}"{}value="{}"{}/>"#,
            variant.name,
            alias_part,
            escape_xml(&variant.value),
            var_comment_attr
        ));
    }

    lines.push("    </enum>".to_string());
}

/// Generate XML for bean type enums grouped by parent
/// Each parent becomes an enum with all beans that have that parent as variants
/// Rules:
/// - value = bean name (string)
/// - alias only generated when @alias tag exists
/// - comment from bean is included if present
/// - Beans without parent are excluded
/// Input: (bean_name, parent, alias, comment)
pub fn generate_bean_type_enums_xml(
    beans_with_parents: &[(&str, &str, Option<&str>, Option<&str>)],
    module_name: &str,
) -> String {
    use std::collections::HashMap;

    // Group beans by parent: parent -> [(bean_name, alias, comment)]
    let mut parent_to_beans: HashMap<&str, Vec<(&str, Option<&str>, Option<&str>)>> =
        HashMap::new();
    for (bean_name, parent, alias, comment) in beans_with_parents {
        if !parent.is_empty() {
            parent_to_beans
                .entry(parent)
                .or_default()
                .push((bean_name, *alias, *comment));
        }
    }

    let mut lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
        format!(
            r#"<module name="{}" comment="自动生成的 bean 类型枚举">"#,
            escape_xml(module_name)
        ),
        String::new(),
    ];

    // Sort parents for consistent output
    let mut parents: Vec<_> = parent_to_beans.keys().collect();
    parents.sort();

    for parent in parents {
        let beans = parent_to_beans.get(parent).unwrap();

        // Generate enum for this parent
        lines.push(format!(
            r#"    <enum name="{}Enum" comment="{} 的子类型">"#,
            parent, parent
        ));

        for (bean_name, alias, comment) in beans.iter() {
            // Only include alias attribute if @alias tag exists
            let alias_attr = alias
                .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
                .unwrap_or_default();
            // Include comment attribute if present
            let comment_attr = comment
                .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
                .unwrap_or_default();
            lines.push(format!(
                r#"        <var name="{}"{}value="{}"{}/>"#,
                bean_name,
                if alias_attr.is_empty() {
                    " ".to_string()
                } else {
                    format!("{} ", alias_attr)
                },
                bean_name,
                comment_attr
            ));
        }

        lines.push("    </enum>".to_string());
        lines.push(String::new());
    }

    lines.push("</module>".to_string());
    lines.join("\n")
}

/// Generate a single <table> element for a class configured in [tables] section
pub fn generate_table(class: &ClassInfo, input: &str, output: &str) -> String {
    let config = class
        .luban_table
        .as_ref()
        .expect("Class must have @LubanTable");

    let mut attrs = vec![
        format!(r#"name="{}""#, class.name),
        format!(r#"value="{}""#, class.name),
        format!(r#"mode="{}""#, config.mode),
        format!(r#"index="{}""#, config.index),
        format!(r#"input="{}""#, input),
        format!(r#"output="{}""#, output),
    ];

    if let Some(group) = &config.group {
        attrs.push(format!(r#"group="{}""#, group));
    }

    if let Some(tags) = &config.tags {
        attrs.push(format!(r#"tags="{}""#, tags));
    }

    format!(r#"    <table {} />"#, attrs.join(" "))
}

#[cfg(test)]
fn generate_xml(classes: &[ClassInfo]) -> String {
    let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
    let table_registry = TableRegistry::new();
    let generator = XmlGenerator::new(&type_mapper, &table_registry);
    generator.generate(classes, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_field(name: &str, field_type: &str, optional: bool) -> FieldInfo {
        FieldInfo {
            name: name.to_string(),
            field_type: field_type.to_string(),
            comment: None,
            alias: None,
            is_optional: optional,
            validators: FieldValidators::default(),
            is_object_factory: false,
            factory_inner_type: None,
            is_constructor: false,
            constructor_inner_type: None,
            original_type: field_type.to_string(),
            default_value: None,
            type_override: None,
            separator: None,
            map_separator: None,
            custom_tags: None,
            ref_key_inner_type: None,
            ref_replace: None,
        }
    }

    #[test]
    fn test_generate_simple_bean() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: Some("Test class".to_string()),
            alias: None,
            fields: vec![FieldInfo {
                name: "name".to_string(),
                field_type: "string".to_string(),
                comment: Some("Name field".to_string()),
                alias: None,
                is_optional: false,
                validators: FieldValidators::default(),
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "string".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
            ref_key_inner_type: None,
            ref_replace: None,
}],
            implements: vec![],
            extends: Some("BaseClass".to_string()),
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="MyClass" parent="BaseClass" comment="Test class">"#));
        assert!(xml.contains(r#"<var name="name" type="string" comment="Name field"/>"#));
    }

    #[test]
    fn test_optional_field() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "string", true)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"type="string?""#));
    }

    #[test]
    fn test_list_not_optional() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("items", "list,string", true)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // List types should NOT have ? suffix even when optional
        assert!(xml.contains(r#"type="list,string""#));
        assert!(!xml.contains(r#"type="list,string?""#));
    }

    #[test]
    fn test_class_no_extends_no_parent() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="MyClass">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_with_extends_has_parent() {
        let class = ClassInfo {
            name: "ChildClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: Some("ParentClass".to_string()),
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="ChildClass" parent="ParentClass">"#));
    }

    #[test]
    fn test_interface_no_extends_no_parent() {
        let interface = ClassInfo {
            name: "MyInterface".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[interface]);
        assert!(xml.contains(r#"<bean name="MyInterface">"#));
        assert!(!xml.contains("parent="));
    }

    // Tests for implements → parent feature

    #[test]
    fn test_class_single_implements_no_extends_has_parent() {
        let class = ClassInfo {
            name: "DamageTrigger".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("damage", "double", false)],
            implements: vec!["EntityTrigger".to_string()],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="DamageTrigger" parent="EntityTrigger">"#));
    }

    #[test]
    fn test_class_multiple_implements_no_extends_no_parent() {
        let class = ClassInfo {
            name: "MultiImplClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec!["Interface1".to_string(), "Interface2".to_string()],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Multiple implements is ambiguous, should have no parent
        assert!(xml.contains(r#"<bean name="MultiImplClass">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_class_extends_overrides_implements() {
        let class = ClassInfo {
            name: "ChildClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec!["SomeInterface".to_string()],
            extends: Some("BaseClass".to_string()),
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Extends should take priority over implements
        assert!(xml.contains(r#"<bean name="ChildClass" parent="BaseClass">"#));
    }

    #[test]
    fn test_class_implements_recursive_interface_chain() {
        let base_interface = ClassInfo {
            name: "EntityTrigger".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "double", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let child_interface = ClassInfo {
            name: "BaseTrigger".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("name", "string", false)],
            implements: vec![],
            extends: Some("EntityTrigger".to_string()),
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let class = ClassInfo {
            name: "DamageTrigger".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("damage", "double", false)],
            implements: vec!["BaseTrigger".to_string()],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[base_interface, child_interface, class]);
        assert!(xml.contains(r#"<bean name="DamageTrigger" parent="BaseTrigger">"#));
        assert!(xml.contains(r#"<bean name="BaseTrigger" parent="EntityTrigger">"#));
    }

    #[test]
    fn test_class_no_implements_no_extends_no_parent() {
        let class = ClassInfo {
            name: "SimpleClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // No implements, no extends -> no parent
        assert!(xml.contains(r#"<bean name="SimpleClass">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(escape_xml("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(escape_xml(r#"say "hello""#), r#"say &quot;hello&quot;"#);
    }

    #[test]
    fn test_generate_string_enum() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "ItemType".to_string(),
            alias: None,
            comment: Some("物品类型".to_string()),
            is_string_enum: true,
            is_flags: false,
            tags: None,
            variants: vec![
                EnumVariant {
                    name: "Role".to_string(),
                    alias: None,               // No @alias tag
                    value: "role".to_string(), // Original string value
                    comment: Some("角色".to_string()),
                },
                EnumVariant {
                    name: "Consumable".to_string(),
                    alias: None,
                    value: "consumable".to_string(),
                    comment: Some("消耗品".to_string()),
                },
            ],
            source_file: "test.ts".to_string(),
            file_hash: "abc".to_string(),
            output_path: None,
            module_name: None,
        };

        let xml = generate_enum_xml(&[enum_info], "test");
        assert!(xml.contains(r#"<enum name="ItemType" comment="物品类型">"#));
        // No alias attribute, value is the original string
        assert!(xml.contains(r#"<var name="Role" value="role" comment="角色"/>"#));
        assert!(xml.contains(r#"<var name="Consumable" value="consumable" comment="消耗品"/>"#));
    }

    #[test]
    fn test_generate_number_enum() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "SkillStyle".to_string(),
            alias: None,
            comment: Some("技能类型".to_string()),
            is_string_enum: false,
            is_flags: false,
            tags: None,
            variants: vec![
                EnumVariant {
                    name: "Attack".to_string(),
                    alias: None, // No @alias tag
                    value: "1".to_string(),
                    comment: Some("攻击技能".to_string()),
                },
                EnumVariant {
                    name: "Defense".to_string(),
                    alias: None,
                    value: "2".to_string(),
                    comment: None,
                },
            ],
            source_file: "test.ts".to_string(),
            file_hash: "abc".to_string(),
            output_path: None,
            module_name: None,
        };

        let xml = generate_enum_xml(&[enum_info], "test");
        // Number enum should NOT have tags="string"
        assert!(xml.contains(r#"<enum name="SkillStyle" comment="技能类型">"#));
        assert!(!xml.contains(r#"tags="string""#));
        // No alias attribute
        assert!(xml.contains(r#"<var name="Attack" value="1" comment="攻击技能"/>"#));
        assert!(xml.contains(r#"<var name="Defense" value="2"/>"#));
    }

    #[test]
    fn test_generate_flags_enum() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "UnitFlag".to_string(),
            alias: None,
            comment: Some("权限控制".to_string()),
            is_string_enum: false,
            is_flags: true,
            tags: None,
            variants: vec![
                EnumVariant {
                    name: "CAN_MOVE".to_string(),
                    alias: Some("移动".to_string()), // Has @alias tag
                    value: "1".to_string(),
                    comment: Some("可以移动".to_string()),
                },
                EnumVariant {
                    name: "CAN_ATTACK".to_string(),
                    alias: Some("攻击".to_string()),
                    value: "2".to_string(),
                    comment: Some("可以攻击".to_string()),
                },
            ],
            source_file: "test.ts".to_string(),
            file_hash: "abc".to_string(),
            output_path: None,
            module_name: None,
        };

        let xml = generate_enum_xml(&[enum_info], "test");
        // Should have flags="true" attribute
        assert!(xml.contains(r#"<enum name="UnitFlag" flags="true" comment="权限控制">"#));
        // Should have alias attribute before value (from @alias tag)
        assert!(xml.contains(r#"<var name="CAN_MOVE" alias="移动" value="1" comment="可以移动"/>"#));
        assert!(
            xml.contains(r#"<var name="CAN_ATTACK" alias="攻击" value="2" comment="可以攻击"/>"#)
        );
    }

    #[test]
    fn test_object_factory_field_inject_data_tag() {
        let class = ClassInfo {
            name: "CharacterConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "triggers".to_string(),
                    field_type: "list,BaseTrigger".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: true,
                    factory_inner_type: Some("BaseTrigger".to_string()),
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "ObjectFactory<BaseTrigger>[]".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
                FieldInfo {
                    name: "normalField".to_string(),
                    field_type: "string".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "string".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // ObjectFactory field should have ObjectFactory=true tag
        assert!(
            xml.contains(r#"tags="ObjectFactory=true""#),
            "XML should contain ObjectFactory=true tag for ObjectFactory fields"
        );
        // Normal field should NOT have ObjectFactory tag
        assert!(!xml.contains(
            r#"<var name="normalField" type="string" tags="ObjectFactory=true""#
        ));
    }

    #[test]
    fn test_skip_dollar_type_field() {
        let class = ClassInfo {
            name: "ShapeInfo".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "$type".to_string(),
                    field_type: "ShapeType".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "ShapeType".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
                FieldInfo {
                    name: "width".to_string(),
                    field_type: "number".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // $type field should NOT be in the output
        assert!(!xml.contains(r#"<var name="$type""#), "XML should not contain $type field");
        // Normal fields should still be present
        assert!(xml.contains(r#"<var name="width" type="double""#), "XML should contain normal fields");
    }

    #[test]
    fn test_beans_keep_source_order() {
        // Create classes in non-alphabetical order
        let class_z = ClassInfo {
            name: "ZClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let class_a = ClassInfo {
            name: "AClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let class_m = ClassInfo {
            name: "MClass".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Pass classes in Z, A, M order - should preserve this order
        let xml = generate_xml(&[class_z, class_a, class_m]);

        // Find positions of each bean in the output
        let pos_a = xml.find(r#"<bean name="AClass""#).expect("AClass not found");
        let pos_m = xml.find(r#"<bean name="MClass""#).expect("MClass not found");
        let pos_z = xml.find(r#"<bean name="ZClass""#).expect("ZClass not found");

        // Beans should keep source order: Z < A < M (as passed in)
        assert!(pos_z < pos_a, "ZClass should come before AClass (source order)");
        assert!(pos_a < pos_m, "AClass should come before MClass (source order)");
    }

    #[test]
    fn test_optional_constructor_field() {
        let class = ClassInfo {
            name: "StatData".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "id".to_string(),
                    field_type: "string".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "string".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
                FieldInfo {
                    name: "component".to_string(),
                    field_type: "string".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: true,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: true,
                    constructor_inner_type: Some("ComponentCls".to_string()),
                    original_type: "Constructor<ComponentCls>".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Optional Constructor field should have ? before #constructor
        assert!(xml.contains(r#"type="string?#constructor=ComponentCls""#),
            "Optional Constructor field should generate string?#constructor=ComponentCls");
    }

    #[test]
    fn test_field_alias() {
        let class = ClassInfo {
            name: "ItemConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "id".to_string(),
                    field_type: "int".to_string(),
                    comment: Some("Item ID".to_string()),
                    alias: Some("物品ID".to_string()),
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "int".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
                FieldInfo {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    comment: None,
                    alias: Some("名称".to_string()),
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "string".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
                FieldInfo {
                    name: "value".to_string(),
                    field_type: "double".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Field with alias and comment should have both attributes
        assert!(
            xml.contains(r#"<var name="id" type="int" alias="物品ID" comment="Item ID"/>"#),
            "Field with alias and comment should generate both attributes"
        );
        // Field with alias only should have alias attribute
        assert!(
            xml.contains(r#"<var name="name" type="string" alias="名称"/>"#),
            "Field with alias only should generate alias attribute"
        );
        // Field without alias should not have alias attribute
        assert!(
            xml.contains(r#"<var name="value" type="double"/>"#),
            "Field without alias should not have alias attribute"
        );
    }

    #[test]
    fn test_cross_module_parent_reference() {
        // ResourceConfig is in module "resource"
        let resource_config = ClassInfo {
            name: "ResourceConfig".to_string(),
            comment: Some("资源基础配置".to_string()),
            alias: None,
            fields: vec![make_field("id", "string", false)],
            implements: vec![],
            extends: None,
            source_file: "resource/resource-config.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("resource".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // WeaponConfig is in module "weapon", extends ResourceConfig
        let weapon_config = ClassInfo {
            name: "WeaponConfig".to_string(),
            comment: Some("武器配置".to_string()),
            alias: None,
            fields: vec![make_field("damage", "double", false)],
            implements: vec![],
            extends: Some("ResourceConfig".to_string()),
            source_file: "weapon/weapon-config.ts".to_string(),
            file_hash: "def456".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("weapon".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Generate XML for weapon module (which references resource module)
        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let generator = XmlGenerator::new(&type_mapper, &table_registry);

        // Pass all classes so the generator can build the class-to-module mapping
        let all_classes = vec![resource_config, weapon_config.clone()];
        let xml = generator.generate_with_all_classes(&[weapon_config], "weapon", &all_classes);

        // Parent should include module prefix: resource.ResourceConfig
        assert!(
            xml.contains(r#"parent="resource.ResourceConfig""#),
            "Parent should include module prefix. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_cross_module_type_reference() {
        // QualityType is an enum in module "enums"
        // ResourceConfig is in module "resource", has a field of type QualityType
        let resource_config = ClassInfo {
            name: "ResourceConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                make_field("id", "string", false),
                make_field("quality", "QualityType", true),
            ],
            implements: vec![],
            extends: None,
            source_file: "resource/resource-config.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("resource".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // QualityType enum (simulated as a class for the mapping)
        let quality_type = ClassInfo {
            name: "QualityType".to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: "enums/quality-type.ts".to_string(),
            file_hash: "ghi789".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("enums".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let generator = XmlGenerator::new(&type_mapper, &table_registry);

        let all_classes = vec![resource_config.clone(), quality_type];
        let xml = generator.generate_with_all_classes(&[resource_config], "resource", &all_classes);

        // Type should include module prefix: enums.QualityType
        assert!(
            xml.contains(r#"type="enums.QualityType?""#),
            "Type should include module prefix. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_same_module_no_prefix() {
        // Both classes are in the same module "weapon"
        let weapon_level_config = ClassInfo {
            name: "WeaponLevelConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("level", "double", false)],
            implements: vec![],
            extends: None,
            source_file: "weapon/weapon-level-config.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("weapon".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let weapon_config = ClassInfo {
            name: "WeaponConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("levels", "list,WeaponLevelConfig", false)],
            implements: vec![],
            extends: None,
            source_file: "weapon/weapon-config.ts".to_string(),
            file_hash: "def456".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("weapon".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let generator = XmlGenerator::new(&type_mapper, &table_registry);

        let all_classes = vec![weapon_level_config, weapon_config.clone()];
        let xml = generator.generate_with_all_classes(&[weapon_config], "weapon", &all_classes);

        // Same module, no prefix needed
        assert!(
            xml.contains(r#"type="list,WeaponLevelConfig""#),
            "Same module types should not have prefix. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_type_override() {
        let class = ClassInfo {
            name: "ConfigWithTypeOverride".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "count".to_string(),
                    field_type: "number".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: Some("int".to_string()),
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Type should be overridden to int instead of double
        assert!(
            xml.contains(r#"type="int""#),
            "@type override should change number to int. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_default_value() {
        let class = ClassInfo {
            name: "ConfigWithDefault".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "value".to_string(),
                    field_type: "number".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number".to_string(),
                    custom_tags: None,
                    default_value: Some("0".to_string()),
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Type should include default value
        assert!(
            xml.contains(r#"type="double#default=0""#),
            "@default should add #default=0 to type. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_type_override_with_default() {
        let class = ClassInfo {
            name: "ConfigWithTypeAndDefault".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "level".to_string(),
                    field_type: "number".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number".to_string(),
                    custom_tags: None,
                    default_value: Some("1".to_string()),
                    type_override: Some("int".to_string()),
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Type should be overridden and include default value
        assert!(
            xml.contains(r#"type="int#default=1""#),
            "@type and @default should combine. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_list_separator() {
        let class = ClassInfo {
            name: "ConfigWithListSep".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "tags".to_string(),
                    field_type: "list,string".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "string[]".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: Some("|".to_string()),
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // List should have separator
        assert!(
            xml.contains(r#"type="(list#sep=|),string""#),
            "@sep should add separator to list. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_map_separator() {
        let class = ClassInfo {
            name: "ConfigWithMapSep".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "data".to_string(),
                    field_type: "map,string,int".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "Map<string, int>".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: None,
                    map_separator: Some(",|".to_string()),
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Map should have separator
        assert!(
            xml.contains(r#"type="(map#sep=,|),string,int""#),
            "@mapsep should add separator to map. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_list_with_separator_and_size() {
        let class = ClassInfo {
            name: "ConfigWithListSepAndSize".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "coords".to_string(),
                    field_type: "list,double".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators {
                        size: Some(SizeConstraint::Exact(3)),
                        ..Default::default()
                    },
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "number[]".to_string(),
                    custom_tags: None,
                    default_value: None,
                    type_override: None,
                    separator: Some("|".to_string()),
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // List should have both separator and size
        assert!(
            xml.contains(r#"type="(list#sep=|#size=3),double""#),
            "@sep and @Size should combine. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_enum_with_tags() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "PieceAttributeType".to_string(),
            alias: None,
            comment: Some("属性类型".to_string()),
            is_string_enum: true,
            is_flags: false,
            tags: Some("string".to_string()),
            variants: vec![
                EnumVariant {
                    name: "Attack".to_string(),
                    alias: None,
                    value: "attack".to_string(),
                    comment: None,
                },
                EnumVariant {
                    name: "Defense".to_string(),
                    alias: None,
                    value: "defense".to_string(),
                    comment: None,
                },
            ],
            source_file: "test.ts".to_string(),
            file_hash: "abc".to_string(),
            output_path: None,
            module_name: None,
        };

        let xml = generate_enum_xml(&[enum_info], "test");
        // Enum should have tags attribute
        assert!(
            xml.contains(r#"<enum name="PieceAttributeType" tags="string" comment="属性类型">"#),
            "@tags should add tags attribute to enum. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_container_with_default() {
        let class = ClassInfo {
            name: "ConfigWithContainerDefault".to_string(),
            comment: None,
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "items".to_string(),
                    field_type: "list,string".to_string(),
                    comment: None,
                    alias: None,
                    is_optional: false,
                    validators: FieldValidators::default(),
                    is_object_factory: false,
                    factory_inner_type: None,
                    is_constructor: false,
                    constructor_inner_type: None,
                    original_type: "string[]".to_string(),
                    custom_tags: None,
                    default_value: Some("[]".to_string()),
                    type_override: None,
                    separator: None,
                    map_separator: None,
                    ref_key_inner_type: None,
                    ref_replace: None,
                },
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Container should have default value at the end
        assert!(
            xml.contains(r#"type="list,string#default=[]""#),
            "@default should add default value to container type. Got:\n{}",
            xml
        );
    }

    // Tests for [tables] config-based table generation

    #[test]
    fn test_table_from_config_map_mode() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "SkillConfig".to_string(),
            comment: Some("技能配置".to_string()),
            alias: None,
            fields: vec![
                make_field("id", "int", false),
                make_field("name", "string", false),
            ],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("skill".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry from config
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "skill.SkillConfig".to_string(),
            TableConfig::Simple("../datas/skill".to_string()),
        );
        let table_registry = TableRegistry::from_config(&tables_config);

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "skill");

        // Should generate table element from [tables] config
        assert!(
            xml.contains(r#"<table name="SkillConfigTable" value="SkillConfig" index="id" input="../datas/skill" />"#),
            "Should generate table element from [tables] config. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_table_from_config_one_mode() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "RollSkillConfig".to_string(),
            comment: Some("抽技能配置".to_string()),
            alias: None,
            fields: vec![make_field("selectionCount", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("rollSkill".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry from config with mode="one"
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "rollSkill.RollSkillConfig".to_string(),
            TableConfig::Full {
                input: "../datas/roll-skill".to_string(),
                name: None,
                mode: Some("one".to_string()),
                index: None,
            },
        );
        let table_registry = TableRegistry::from_config(&tables_config);

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "rollSkill");

        // Should generate table element with one mode (no index)
        assert!(
            xml.contains(r#"<table name="RollSkillConfigTable" value="RollSkillConfig" mode="one" input="../datas/roll-skill" />"#),
            "Should generate table element with mode=one. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_table_from_config_with_chinese_path() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "AllianceAttackInfo".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("Id", "string", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("battle".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Chinese path
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "battle.AllianceAttackInfo".to_string(),
            TableConfig::Full {
                input: "../datas/battle/我方普通攻击配置表.xlsx".to_string(),
                name: None,
                mode: None,
                index: Some("Id".to_string()),
            },
        );
        let table_registry = TableRegistry::from_config(&tables_config);

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "battle");

        // Should handle Chinese characters in path
        assert!(
            xml.contains(r#"input="../datas/battle/我方普通攻击配置表.xlsx""#),
            "Should handle Chinese characters in input path. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_no_table_without_config() {
        let class = ClassInfo {
            name: "NoTableConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let xml = generate_xml(&[class]);
        // Should NOT generate table element without [tables] config
        assert!(
            !xml.contains("<table"),
            "Should not generate table element without [tables] config. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_table_with_custom_name() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "BattleData".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("battleId", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("battle".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with custom table name
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "battle.BattleData".to_string(),
            TableConfig::Full {
                input: "../datas/battle".to_string(),
                name: Some("TbBattle".to_string()),
                mode: None,
                index: Some("battleId".to_string()),
            },
        );
        let table_registry = TableRegistry::from_config(&tables_config);

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "battle");

        // Should use custom table name
        assert!(
            xml.contains(r#"<table name="TbBattle" value="BattleData" index="battleId" input="../datas/battle" />"#),
            "Should use custom table name from config. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_ref_on_list_generates_ref_validator() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "DropConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "items".to_string(),
                field_type: "list,Item".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators {
                    has_ref: true,
                    ..Default::default()
                },
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "Item[]".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
            ref_key_inner_type: None,
            ref_replace: None,
}],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Item table
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "items.Item".to_string(),
            TableConfig::Full {
                input: "../datas/items".to_string(),
                name: Some("TbItem".to_string()),
                mode: None,
                index: Some("id".to_string()),
            },
        );
        let mut table_registry = TableRegistry::from_config(&tables_config);

        // Set index type for Item table
        let item_class = ClassInfo {
            name: "Item".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "string", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        table_registry.set_index_types(&[item_class], &type_mapper);

        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "items");

        // Should generate list,string#ref=items.TbItem
        assert!(
            xml.contains(r#"type="list,string#ref=items.TbItem""#),
            "Should generate ref validator for list element. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_ref_on_map_generates_ref_validator() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "SkillConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "itemToSkill".to_string(),
                field_type: "map,Item,Skill".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators {
                    has_ref: true,      // @ref for value
                    has_ref_key: true,  // RefKey<T> for key
                    ..Default::default()
                },
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "Map<RefKey<Item>, Skill>".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
                ref_key_inner_type: Some("Item".to_string()),
                ref_replace: None,
            }],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("skills".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Item and Skill tables
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "items.Item".to_string(),
            TableConfig::Full {
                input: "../datas/items".to_string(),
                name: Some("TbItem".to_string()),
                mode: None,
                index: Some("id".to_string()),
            },
        );
        tables_config.insert(
            "skills.Skill".to_string(),
            TableConfig::Full {
                input: "../datas/skills".to_string(),
                name: Some("TbSkill".to_string()),
                mode: None,
                index: Some("skillId".to_string()),
            },
        );
        let mut table_registry = TableRegistry::from_config(&tables_config);

        // Set index types
        let item_class = ClassInfo {
            name: "Item".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };
        let skill_class = ClassInfo {
            name: "Skill".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("skillId", "string", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("skills".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        table_registry.set_index_types(&[item_class, skill_class], &type_mapper);

        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "skills");

        // Should generate map,int#ref=items.TbItem,string#ref=skills.TbSkill
        assert!(
            xml.contains(r#"type="map,int#ref=items.TbItem,string#ref=skills.TbSkill""#),
            "Should generate ref validators for both map key and value. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_ref_on_set_generates_ref_validator() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "CollectionConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "uniqueItems".to_string(),
                field_type: "set,Item".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators {
                    has_ref: true,
                    ..Default::default()
                },
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "Set<Item>".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
            ref_key_inner_type: None,
            ref_replace: None,
}],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Item table
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "items.Item".to_string(),
            TableConfig::Full {
                input: "../datas/items".to_string(),
                name: Some("TbItem".to_string()),
                mode: None,
                index: Some("id".to_string()),
            },
        );
        let mut table_registry = TableRegistry::from_config(&tables_config);

        // Set index type for Item table
        let item_class = ClassInfo {
            name: "Item".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        table_registry.set_index_types(&[item_class], &type_mapper);

        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "items");

        // Should generate set,int#ref=items.TbItem (int is the index type)
        assert!(
            xml.contains(r#"type="set,int#ref=items.TbItem""#),
            "Should generate ref validator for set element. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_ref_key_scalar_generates_ref_validator() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "RefKeyScalarConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "item".to_string(),
                field_type: "Item".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators {
                    has_ref_key: true, // RefKey<Item>
                    ..Default::default()
                },
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "RefKey<Item>".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
                ref_key_inner_type: Some("Item".to_string()),
                ref_replace: None,
            }],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Item table
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "items.Item".to_string(),
            TableConfig::Full {
                input: "../datas/items".to_string(),
                name: Some("TbItem".to_string()),
                mode: None,
                index: Some("id".to_string()),
            },
        );
        let mut table_registry = TableRegistry::from_config(&tables_config);

        // Set index type for Item table
        let item_class = ClassInfo {
            name: "Item".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        table_registry.set_index_types(&[item_class], &type_mapper);

        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "items");

        // Should generate int#ref=items.TbItem (int is the index type)
        // Note: RefKey<T> does NOT add RefOverride=true tag (unlike @ref)
        assert!(
            xml.contains(r#"type="int#ref=items.TbItem""#),
            "RefKey<T> scalar should generate ref validator. Got:\n{}",
            xml
        );
        // Should NOT have RefOverride tag
        assert!(
            !xml.contains(r#"tags="RefOverride=true""#),
            "RefKey<T> should NOT add RefOverride tag. Got:\n{}",
            xml
        );
    }

    #[test]
    fn test_ref_key_array_generates_ref_validator() {
        use crate::config::TableConfig;

        let class = ClassInfo {
            name: "RefKeyArrayConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "items".to_string(),
                field_type: "list,Item".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators {
                    has_ref_key: true, // RefKey<Item>[]
                    ..Default::default()
                },
                is_object_factory: false,
                factory_inner_type: None,
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "RefKey<Item>[]".to_string(),
                default_value: None,
                type_override: None,
                separator: None,
                map_separator: None,
                custom_tags: None,
                ref_key_inner_type: Some("Item".to_string()),
                ref_replace: None,
            }],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        // Build table registry with Item table
        let mut tables_config = std::collections::HashMap::new();
        tables_config.insert(
            "items.Item".to_string(),
            TableConfig::Full {
                input: "../datas/items".to_string(),
                name: Some("TbItem".to_string()),
                mode: None,
                index: Some("id".to_string()),
            },
        );
        let mut table_registry = TableRegistry::from_config(&tables_config);

        // Set index type for Item table
        let item_class = ClassInfo {
            name: "Item".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("id", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: Some("items".to_string()),
            type_params: std::collections::HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: ImportMap::new(),
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        table_registry.set_index_types(&[item_class], &type_mapper);

        let generator = XmlGenerator::new(&type_mapper, &table_registry);
        let xml = generator.generate(&[class], "items");

        // Should generate list,int#ref=items.TbItem
        assert!(
            xml.contains(r#"type="list,int#ref=items.TbItem""#),
            "RefKey<T>[] should generate ref validator for list element. Got:\n{}",
            xml
        );
        // Should NOT have RefOverride tag
        assert!(
            !xml.contains(r#"tags="RefOverride=true""#),
            "RefKey<T>[] should NOT add RefOverride tag. Got:\n{}",
            xml
        );
    }
}
