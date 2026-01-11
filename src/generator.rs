use crate::parser::field_info::SizeConstraint;
use crate::parser::{ClassInfo, EnumInfo, FieldInfo, FieldValidators};
use crate::table_mapping::TableMappingResolver;
use crate::table_registry::TableRegistry;
use crate::type_mapper::TypeMapper;

pub struct XmlGenerator<'a> {
    type_mapper: &'a TypeMapper,
    table_registry: &'a TableRegistry,
    table_mapping_resolver: &'a TableMappingResolver,
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
        }
    }

    pub fn generate(&self, classes: &[ClassInfo], module_name: &str) -> String {
        let mut lines = vec![
            r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
            format!(
                r#"<module name="{}" comment="自动生成的 ts class Bean 定义">"#,
                escape_xml(module_name)
            ),
            String::new(),
        ];

        // Generate beans
        for class in classes {
            self.generate_bean(&mut lines, class, classes);
            lines.push(String::new());
        }

        // Generate tables for @LubanTable classes
        let luban_tables: Vec<_> = classes.iter().filter(|c| c.luban_table.is_some()).collect();

        if !luban_tables.is_empty() {
            lines.push("    <!-- 数据表配置 -->".to_string());
            for class in luban_tables {
                self.generate_table_element(&mut lines, class);
            }
            lines.push(String::new());
        }

        lines.push("</module>".to_string());
        lines.join("\n")
    }

    fn generate_table_element(&self, lines: &mut Vec<String>, class: &ClassInfo) {
        let config = class.luban_table.as_ref().unwrap();

        // Resolve input/output from table_mappings config
        let (input, output) = self
            .table_mapping_resolver
            .resolve(&class.name)
            .unwrap_or_else(|| {
                panic!(
                    "Error: No table_mapping for class '{}'. Add [[table_mappings]] in config.",
                    class.name
                )
            });

        let table_name = format!("{}Table", class.name);

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
        let parent = class.extends.clone().unwrap_or_default();

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

        let parent_attr = if parent.is_empty() {
            String::new()
        } else {
            format!(r#" parent="{}""#, parent)
        };

        lines.push(format!(
            r#"    <bean name="{}"{}{}{}>"#,
            class.name, alias_attr, parent_attr, comment_attr
        ));

        // Collect parent field names to skip redeclared fields
        let mut parent_field_names = std::collections::HashSet::new();
        let mut current_parent = class.extends.as_ref();
        while let Some(parent_name) = current_parent {
            if let Some(parent) = all_classes.iter().find(|c| &c.name == parent_name) {
                for field in &parent.fields {
                    parent_field_names.insert(field.name.as_str());
                }
                current_parent = parent.extends.as_ref();
            } else {
                break;
            }
        }

        // Only generate fields that are not redeclared from parent classes
        for field in &class.fields {
            if !parent_field_names.contains(field.name.as_str()) {
                self.generate_field(lines, field);
            }
        }

        lines.push("    </bean>".to_string());
    }

    fn generate_field(&self, lines: &mut Vec<String>, field: &FieldInfo) {
        // Handle Constructor<T> fields
        if field.is_constructor {
            if let Some(constructor_type) = &field.constructor_inner_type {
                let final_type = format!("string#constructor={}", constructor_type);

                let comment_attr = field
                    .comment
                    .as_ref()
                    .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
                    .unwrap_or_default();

                let tags_attr = field
                    .relocate_tags
                    .as_ref()
                    .map(|t| format!(r#" tags="{}""#, escape_xml(t)))
                    .unwrap_or_default();

                lines.push(format!(
                    r#"        <var name="{}" type="{}"{}{}/>"#,
                    field.name, final_type, comment_attr, tags_attr
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

        // Check if this is a container type (list, map, array, set)
        let is_container = mapped_type.starts_with("list,")
            || mapped_type.starts_with("map,")
            || mapped_type.starts_with("array,")
            || mapped_type.starts_with("set,");

        let final_type = if is_container {
            // Handle container types with size/index validators
            self.apply_container_validators(&mapped_type, validators)
        } else {
            // Handle scalar types with validators
            self.apply_scalar_validators(&mapped_type, validators, field.is_optional)
        };

        let comment_attr = field
            .comment
            .as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let tags_attr = field
            .relocate_tags
            .as_ref()
            .map(|t| format!(r#" tags="{}""#, escape_xml(t)))
            .unwrap_or_default();

        lines.push(format!(
            r#"        <var name="{}" type="{}"{}{}/>"#,
            field.name, final_type, comment_attr, tags_attr
        ));
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

    let tags_attr = if enum_info.is_string_enum {
        r#" tags="string""#.to_string()
    } else {
        String::new()
    };

    lines.push(format!(
        r#"    <enum name="{}"{}{}{}{}>"#,
        enum_info.name, alias_attr, flags_attr, comment_attr, tags_attr
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
    fn test_no_extends_no_parent() {
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
        };

        let xml = generate_xml(&[class]);
        assert!(xml.contains(r#"<bean name="ChildClass" parent="ParentClass">"#));
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
        assert!(xml.contains(r#"<enum name="ItemType" comment="物品类型" tags="string">"#));
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
}
