use crate::parser::{ClassInfo, EnumInfo, FieldInfo};
use crate::base_class::BaseClassResolver;
use crate::type_mapper::TypeMapper;

pub struct XmlGenerator<'a> {
    base_resolver: &'a BaseClassResolver<'a>,
    type_mapper: &'a TypeMapper,
}

impl<'a> XmlGenerator<'a> {
    pub fn new(base_resolver: &'a BaseClassResolver<'a>, type_mapper: &'a TypeMapper) -> Self {
        Self { base_resolver, type_mapper }
    }

    pub fn generate(&self, classes: &[ClassInfo], module_name: &str) -> String {
        let mut lines = vec![
            r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
            format!(r#"<module name="{}" comment="自动生成的 ts class Bean 定义">"#, escape_xml(module_name)),
            String::new(),
        ];

        for class in classes {
            self.generate_bean(&mut lines, class);
            lines.push(String::new());
        }

        lines.push("</module>".to_string());
        lines.join("\n")
    }

    fn generate_bean(&self, lines: &mut Vec<String>, class: &ClassInfo) {
        let parent = self.base_resolver.resolve(class);

        let alias_attr = class.alias.as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        let comment_attr = class.comment.as_ref()
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

        for field in &class.fields {
            self.generate_field(lines, field);
        }

        lines.push("    </bean>".to_string());
    }

    fn generate_field(&self, lines: &mut Vec<String>, field: &FieldInfo) {
        let mapped_type = self.type_mapper.map_full_type(&field.field_type);

        let final_type = if field.is_optional && !mapped_type.starts_with("list,") {
            format!("{}?", mapped_type)
        } else {
            mapped_type
        };

        let comment_attr = field.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        lines.push(format!(
            r#"        <var name="{}" type="{}"{}/>"#,
            field.name, final_type, comment_attr
        ));
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Generate XML for enums only
pub fn generate_enum_xml(enums: &[EnumInfo], module_name: &str) -> String {
    let mut lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
        format!(r#"<module name="{}" comment="自动生成的 ts enum 定义">"#, escape_xml(module_name)),
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
    let alias_attr = enum_info.alias.as_ref()
        .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
        .unwrap_or_default();

    let flags_attr = if enum_info.is_flags {
        r#" flags="true""#.to_string()
    } else {
        String::new()
    };

    let comment_attr = enum_info.comment.as_ref()
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
        let var_alias_attr = variant.alias.as_ref()
            .map(|a| format!(r#" alias="{}""#, escape_xml(a)))
            .unwrap_or_default();

        let var_comment_attr = variant.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        lines.push(format!(
            r#"        <var name="{}" value="{}"{}{}/>"#,
            variant.name, escape_xml(&variant.value), var_alias_attr, var_comment_attr
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
pub fn generate_bean_type_enums_xml(beans_with_parents: &[(&str, &str, Option<&str>, Option<&str>)], module_name: &str) -> String {
    use std::collections::HashMap;

    // Group beans by parent: parent -> [(bean_name, alias, comment)]
    let mut parent_to_beans: HashMap<&str, Vec<(&str, Option<&str>, Option<&str>)>> = HashMap::new();
    for (bean_name, parent, alias, comment) in beans_with_parents {
        if !parent.is_empty() {
            parent_to_beans.entry(parent).or_default().push((bean_name, *alias, *comment));
        }
    }

    let mut lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
        format!(r#"<module name="{}" comment="自动生成的 bean 类型枚举">"#, escape_xml(module_name)),
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
                if alias_attr.is_empty() { " ".to_string() } else { format!("{} ", alias_attr) },
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

#[cfg(test)]
fn generate_xml(classes: &[ClassInfo], default_base: &str) -> String {
    let base_resolver = BaseClassResolver::new(default_base, &[]);
    let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
    let generator = XmlGenerator::new(&base_resolver, &type_mapper);
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
        }
    }

    #[test]
    fn test_generate_simple_bean() {
        let class = ClassInfo {
            name: "MyClass".to_string(),
            comment: Some("Test class".to_string()),
            alias: None,
            fields: vec![
                FieldInfo {
                    name: "name".to_string(),
                    field_type: "string".to_string(),
                    comment: Some("Name field".to_string()),
                    is_optional: false,
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
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyClass" parent="TsClass" comment="Test class">"#));
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
        };

        let xml = generate_xml(&[class], "TsClass");
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
        };

        let xml = generate_xml(&[class], "TsClass");
        // List types should NOT have ? suffix even when optional
        assert!(xml.contains(r#"type="list,string""#));
        assert!(!xml.contains(r#"type="list,string?""#));
    }

    #[test]
    fn test_interface_no_extends_no_parent() {
        let class = ClassInfo {
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
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyInterface">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_interface_with_extends_has_parent() {
        let class = ClassInfo {
            name: "ChildInterface".to_string(),
            comment: None,
            alias: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: Some("ParentInterface".to_string()),
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="ChildInterface" parent="ParentInterface">"#));
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
                    alias: None,  // No @alias tag
                    value: "role".to_string(),  // Original string value
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
                    alias: None,  // No @alias tag
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
                    alias: Some("移动".to_string()),  // Has @alias tag
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
        assert!(xml.contains(r#"<var name="CAN_ATTACK" value="2" alias="攻击" comment="可以攻击"/>"#));
    }
}
