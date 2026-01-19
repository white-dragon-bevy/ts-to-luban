use crate::parser::field_info::SizeConstraint;
use crate::parser::{ClassInfo, EnumInfo, FieldInfo, FieldValidators};
use crate::table_mapping::TableMappingResolver;
use crate::table_registry::TableRegistry;
use crate::type_mapper::TypeMapper;
use std::collections::HashMap;

pub struct XmlGenerator<'a> {
    type_mapper: &'a TypeMapper,
    table_registry: &'a TableRegistry,
    table_mapping_resolver: &'a TableMappingResolver,
    /// Mapping from type name to module name (for cross-module type resolution)
    type_to_module: HashMap<String, String>,
}

impl<'a> XmlGenerator<'a> {
    pub fn new(
        type_mapper: &'a TypeMapper,
        table_registry: &'a TableRegistry,
        table_mapping_resolver: &'a TableMappingResolver,
    ) -> Self {
        Self {
            type_mapper,
            table_registry,
            table_mapping_resolver,
            type_to_module: HashMap::new(),
        }
    }

    /// Create a new XmlGenerator with a pre-built type-to-module mapping
    pub fn with_type_mapping(
        type_mapper: &'a TypeMapper,
        table_registry: &'a TableRegistry,
        table_mapping_resolver: &'a TableMappingResolver,
        type_to_module: HashMap<String, String>,
    ) -> Self {
        Self {
            type_mapper,
            table_registry,
            table_mapping_resolver,
            type_to_module,
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
        let mut lines = vec![
            r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
            format!(
                r#"<module name="{}" comment="自动生成的 ts class Bean 定义">"#,
                escape_xml(module_name)
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

        // Deduplicate classes by name, prioritizing @LubanTable classes
        let mut seen: std::collections::HashMap<String, &ClassInfo> = std::collections::HashMap::new();
        for class in classes {
            let name = &class.name;
            if let Some(existing) = seen.get(name) {
                // If existing class has no @LubanTable but current has, replace it
                if existing.luban_table.is_none() && class.luban_table.is_some() {
                    seen.insert(name.clone(), class);
                }
            } else {
                seen.insert(name.clone(), class);
            }
        }
        let mut unique_classes: Vec<_> = seen.values().collect();
        // Sort beans by name for consistent output
        unique_classes.sort_by(|a, b| a.name.cmp(&b.name));

        // Generate beans
        for class in &unique_classes {
            self.generate_bean_with_module_map(&mut lines, class, all_classes, module_name, &class_to_module);
            lines.push(String::new());
        }

        // Generate tables for @LubanTable classes
        // First collect tables with their resolved names for sorting
        let mut table_entries: Vec<_> = classes
            .iter()
            .filter(|c| c.luban_table.is_some())
            .map(|class| {
                let (input, output, table_name_override) = self
                    .table_mapping_resolver
                    .resolve(&class.name)
                    .unwrap_or_else(|| {
                        panic!(
                            "Error: No table_mapping for class '{}'. Add [[table_mappings]] in config.",
                            class.name
                        )
                    });
                let table_name = table_name_override.unwrap_or_else(|| format!("{}Table", class.name));
                (class, table_name, input, output)
            })
            .collect();
        // Sort by table name for consistent output
        table_entries.sort_by(|a, b| a.1.cmp(&b.1));

        if !table_entries.is_empty() {
            lines.push("    <!-- 数据表配置 -->".to_string());
            for (class, table_name, input, output) in table_entries {
                self.generate_table_element_with_resolved(&mut lines, class, &table_name, &input, output.as_deref());
            }
            lines.push(String::new());
        }

        lines.push("</module>".to_string());
        lines.join("\n")
    }

    /// Generate table element with pre-resolved table name, input, and output
    fn generate_table_element_with_resolved(
        &self,
        lines: &mut Vec<String>,
        class: &ClassInfo,
        table_name: &str,
        input: &str,
        output: Option<&str>,
    ) {
        let config = class.luban_table.as_ref().unwrap();

        let mut attrs = vec![
            format!(r#"name="{}""#, table_name),
            format!(r#"value="{}""#, class.name),
            format!(r#"mode="{}""#, config.mode),
            format!(r#"index="{}""#, config.index),
            format!(r#"input="{}""#, input),
        ];

        if let Some(out) = output {
            attrs.push(format!(r#"output="{}""#, out));
        }

        if let Some(group) = &config.group {
            attrs.push(format!(r#"group="{}""#, group));
        }

        if let Some(tags) = &config.tags {
            attrs.push(format!(r#"tags="{}""#, tags));
        }

        lines.push(format!(r#"    <table {}/>"#, attrs.join(" ")));
    }

    fn generate_table_element(&self, lines: &mut Vec<String>, class: &ClassInfo) {
        let config = class.luban_table.as_ref().unwrap();

        // Resolve input/output from table_mappings config
        let (input, output, table_name_override) = self
            .table_mapping_resolver
            .resolve(&class.name)
            .unwrap_or_else(|| {
                panic!(
                    "Error: No table_mapping for class '{}'. Add [[table_mappings]] in config.",
                    class.name
                )
            });

        let table_name = table_name_override.unwrap_or_else(|| format!("{}Table", class.name));

        let mut attrs = vec![
            format!(r#"name="{}""#, table_name),
            format!(r#"value="{}""#, class.name),
            format!(r#"mode="{}""#, config.mode),
            format!(r#"index="{}""#, config.index),
            format!(r#"input="{}""#, input),
        ];

        if let Some(out) = output {
            attrs.push(format!(r#"output="{}""#, out));
        }

        if let Some(group) = &config.group {
            attrs.push(format!(r#"group="{}""#, group));
        }

        if let Some(tags) = &config.tags {
            attrs.push(format!(r#"tags="{}""#, tags));
        }

        lines.push(format!(r#"    <table {}/>"#, attrs.join(" ")));
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

        // Resolve parent with module prefix if needed
        let resolved_parent = self.resolve_type_with_module(&parent, current_module, class_to_module);

        let alias_attr = class
            .alias
            .as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        let comment_attr = class
            .comment
            .as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let parent_attr = if resolved_parent.is_empty() {
            String::new()
        } else {
            format!(r#" parent="{}""#, resolved_parent)
        };

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
                self.generate_field_with_module_map(lines, field, current_module, class_to_module);
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
        if type_name.is_empty() {
            return String::new();
        }

        // Check if this type is in the class_to_module mapping
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
    /// 3. Default "TsClass" (only when no extends and no/multiple implements)
    fn resolve_class_parent(&self, class: &ClassInfo, _all_classes: &[ClassInfo]) -> String {
        // Priority 1: Use extends if present
        if let Some(extends) = &class.extends {
            return extends.clone();
        }

        // Priority 2: Use single implements if present
        if class.implements.len() == 1 {
            return class.implements[0].clone();
        }

        // Priority 3: Default to TsClass
        "TsClass".to_string()
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
        // Handle Constructor<T> fields
        if field.is_constructor {
            if let Some(constructor_type) = &field.constructor_inner_type {
                let resolved_constructor_type = self.resolve_type_with_module(constructor_type, current_module, class_to_module);
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

                let tags_attr = field
                    .relocate_tags
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

        let mut mapped_type = self.type_mapper.map_full_type(&field.field_type);
        let validators = &field.validators;

        // @Set only supports int/long/string/enum, not double
        // When @Set is present and type is double, convert to int
        if !validators.set_values.is_empty() && mapped_type == "double" {
            mapped_type = "int".to_string();
        }

        // Resolve type references with module prefix
        mapped_type = self.resolve_full_type_with_module(&mapped_type, current_module, class_to_module);

        // Check if this is a container type (list, map, array, set)
        let is_container = mapped_type.starts_with("list,")
            || mapped_type.starts_with("map,")
            || mapped_type.starts_with("array,")
            || mapped_type.starts_with("set,");

        let final_type = if is_container {
            // Handle container types with size/index validators
            self.apply_container_validators_with_module(&mapped_type, validators, current_module, class_to_module)
        } else {
            // Handle scalar types with validators
            self.apply_scalar_validators(&mapped_type, validators, field.is_optional)
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

        // Build tags: relocate_tags + injectData for ObjectFactory
        let tags_attr = {
            let mut tags = Vec::new();

            if let Some(relocate) = &field.relocate_tags {
                tags.push(relocate.as_str());
            }

            if field.is_object_factory {
                tags.push("ObjectFactory=true");
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

    /// Apply validators to scalar types
    /// e.g., "int" -> "int!#ref=examples.TbItem#range=[1,100]"
    fn apply_scalar_validators(
        &self,
        base_type: &str,
        validators: &FieldValidators,
        is_optional: bool,
    ) -> String {
        let mut result = base_type.to_string();

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

        // Handle ref - resolve full namespace path using TableRegistry
        if let Some(ref_target) = &validators.ref_target {
            // Must resolve via registry, error if not found
            let resolved = self.table_registry.resolve_ref(ref_target)
                .unwrap_or_else(|| panic!("Error: @Ref target '{}' not found. Make sure '{}' has @LubanTable decorator.", ref_target, ref_target));
            validator_parts.push(format!("ref={}", resolved));
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
            ref_target: validators.ref_target.clone(),
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
            ref_target: validators.ref_target.clone(),
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
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
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
    lines.join("\n")
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

    let comment_attr = enum_info
        .comment
        .as_ref()
        .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
        .unwrap_or_default();

    lines.push(format!(
        r#"    <enum name="{}"{}{}{}>"#,
        enum_info.name, alias_attr, flags_attr, comment_attr
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

        lines.push(format!(
            r#"        <var name="{}" value="{}"{}{}/>"#,
            variant.name,
            escape_xml(&variant.value),
            var_alias_attr,
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

/// Generate a single <table> element for a class with @LubanTable decorator
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

    format!(r#"    <table {}/>"#, attrs.join(" "))
}

#[cfg(test)]
fn generate_xml(classes: &[ClassInfo]) -> String {
    let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
    let table_registry = TableRegistry::new();
    let table_mapping_resolver = TableMappingResolver::new(&[]);
    let generator = XmlGenerator::new(&type_mapper, &table_registry, &table_mapping_resolver);
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
            relocate_tags: None,
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
                relocate_tags: None,
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
        };

        let xml = generate_xml(&[class]);
        // List types should NOT have ? suffix even when optional
        assert!(xml.contains(r#"type="list,string""#));
        assert!(!xml.contains(r#"type="list,string?""#));
    }

    #[test]
    fn test_class_no_extends_has_tsclass_parent() {
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
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="MyClass" parent="TsClass">"#));
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
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="DamageTrigger" parent="EntityTrigger">"#));
    }

    #[test]
    fn test_class_multiple_implements_no_extends_has_tsclass_parent() {
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
        };

        let xml = generate_xml(&[class]);
        // Multiple implements is ambiguous, should default to TsClass
        assert!(xml.contains(r#"<bean name="MultiImplClass" parent="TsClass">"#));
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
        };

        let xml = generate_xml(&[base_interface, child_interface, class]);
        assert!(xml.contains(r#"<bean name="DamageTrigger" parent="BaseTrigger">"#));
        assert!(xml.contains(r#"<bean name="BaseTrigger" parent="EntityTrigger">"#));
    }

    #[test]
    fn test_class_no_implements_no_extends_has_tsclass_parent() {
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
        };

        let xml = generate_xml(&[class]);
        // Backward compatibility: no implements, no extends → default to TsClass
        assert!(xml.contains(r#"<bean name="SimpleClass" parent="TsClass">"#));
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
        // Should have alias attribute (from @alias tag)
        assert!(xml.contains(r#"<var name="CAN_MOVE" value="1" alias="移动" comment="可以移动"/>"#));
        assert!(
            xml.contains(r#"<var name="CAN_ATTACK" value="2" alias="攻击" comment="可以攻击"/>"#)
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
    fn test_object_factory_with_relocate_tags() {
        let class = ClassInfo {
            name: "WeaponConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![FieldInfo {
                name: "mainStat".to_string(),
                field_type: "ScalingStat".to_string(),
                comment: None,
                alias: None,
                is_optional: false,
                validators: FieldValidators::default(),
                is_object_factory: true,
                factory_inner_type: Some("ScalingStat".to_string()),
                is_constructor: false,
                constructor_inner_type: None,
                original_type: "ObjectFactory<ScalingStat>".to_string(),
                relocate_tags: Some("relocateTo=TScalingStat,prefix=_main".to_string()),
            }],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
            luban_table: None,
        };

        let xml = generate_xml(&[class]);
        // Both tags should be present, separated by comma
        assert!(
            xml.contains(
                r#"tags="relocateTo=TScalingStat,prefix=_main,ObjectFactory=true""#
            ) || xml.contains(
                r#"tags="ObjectFactory=true,relocateTo=TScalingStat,prefix=_main""#
            ),
            "XML should combine relocate_tags and ObjectFactory tag"
        );
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
        };

        let xml = generate_xml(&[class]);
        // $type field should NOT be in the output
        assert!(!xml.contains(r#"<var name="$type""#), "XML should not contain $type field");
        // Normal fields should still be present
        assert!(xml.contains(r#"<var name="width" type="double""#), "XML should contain normal fields");
    }

    #[test]
    fn test_beans_sorted_by_name() {
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
        };

        // Pass classes in Z, A, M order
        let xml = generate_xml(&[class_z, class_a, class_m]);

        // Find positions of each bean in the output
        let pos_a = xml.find(r#"<bean name="AClass""#).expect("AClass not found");
        let pos_m = xml.find(r#"<bean name="MClass""#).expect("MClass not found");
        let pos_z = xml.find(r#"<bean name="ZClass""#).expect("ZClass not found");

        // Beans should be sorted alphabetically: A < M < Z
        assert!(pos_a < pos_m, "AClass should come before MClass");
        assert!(pos_m < pos_z, "MClass should come before ZClass");
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
                    relocate_tags: None,
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
        };

        // Generate XML for weapon module (which references resource module)
        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let table_mapping_resolver = TableMappingResolver::new(&[]);
        let generator = XmlGenerator::new(&type_mapper, &table_registry, &table_mapping_resolver);

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
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let table_mapping_resolver = TableMappingResolver::new(&[]);
        let generator = XmlGenerator::new(&type_mapper, &table_registry, &table_mapping_resolver);

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
        };

        let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
        let table_registry = TableRegistry::new();
        let table_mapping_resolver = TableMappingResolver::new(&[]);
        let generator = XmlGenerator::new(&type_mapper, &table_registry, &table_mapping_resolver);

        let all_classes = vec![weapon_level_config, weapon_config.clone()];
        let xml = generator.generate_with_all_classes(&[weapon_config], "weapon", &all_classes);

        // Same module, no prefix needed
        assert!(
            xml.contains(r#"type="list,WeaponLevelConfig""#),
            "Same module types should not have prefix. Got:\n{}",
            xml
        );
    }
}
