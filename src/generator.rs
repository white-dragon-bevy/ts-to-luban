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
        let comment_attr = class.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        let parent_attr = if parent.is_empty() {
            String::new()
        } else {
            format!(r#" parent="{}""#, parent)
        };

        lines.push(format!(
            r#"    <bean name="{}"{}{}>"#,
            class.name, parent_attr, comment_attr
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
        r#"    <enum name="{}"{}{}{}>"#,
        enum_info.name, flags_attr, comment_attr, tags_attr
    ));

    for variant in &enum_info.variants {
        let var_comment_attr = variant.comment.as_ref()
            .map(|c| format!(r#" comment="{}""#, escape_xml(c)))
            .unwrap_or_default();

        lines.push(format!(
            r#"        <var name="{}" alias="{}" value="{}"{}/>"#,
            variant.name, variant.alias, variant.value, var_comment_attr
        ));
    }

    lines.push("    </enum>".to_string());
}

/// Generate XML for bean names collection
/// Creates beans:
/// - TsClassName: single bean name (string#set)
/// - TsClassNames: multiple bean names (list,string#set)
pub fn generate_bean_names_xml(bean_names: &[&str], module_name: &str) -> String {
    let set_value = bean_names.join(",");

    let lines = vec![
        r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
        format!(r#"<module name="{}" comment="bean name set">"#, escape_xml(module_name)),
        String::new(),
        r#"    <bean name="TsClassName">"#.to_string(),
        format!(r#"        <var name="name" type="string#(set={})"/>"#, set_value),
        r#"    </bean>"#.to_string(),
        String::new(),
        r#"    <bean name="TsClassNames">"#.to_string(),
        format!(r#"        <var name="names" type="list,string#(set={})"/>"#, set_value),
        r#"    </bean>"#.to_string(),
        String::new(),
        "</module>".to_string(),
    ];

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
            comment: Some("物品类型".to_string()),
            is_string_enum: true,
            is_flags: false,
            variants: vec![
                EnumVariant {
                    name: "Role".to_string(),
                    alias: "role".to_string(),
                    value: 1,
                    comment: Some("角色".to_string()),
                },
                EnumVariant {
                    name: "Consumable".to_string(),
                    alias: "consumable".to_string(),
                    value: 2,
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
        assert!(xml.contains(r#"<var name="Role" alias="role" value="1" comment="角色"/>"#));
        assert!(xml.contains(r#"<var name="Consumable" alias="consumable" value="2" comment="消耗品"/>"#));
    }

    #[test]
    fn test_generate_number_enum() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "SkillStyle".to_string(),
            comment: Some("技能类型".to_string()),
            is_string_enum: false,
            is_flags: false,
            variants: vec![
                EnumVariant {
                    name: "Attack".to_string(),
                    alias: "attack".to_string(),
                    value: 1,
                    comment: Some("攻击技能".to_string()),
                },
                EnumVariant {
                    name: "Defense".to_string(),
                    alias: "defense".to_string(),
                    value: 2,
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
        assert!(xml.contains(r#"<var name="Attack" alias="attack" value="1" comment="攻击技能"/>"#));
        assert!(xml.contains(r#"<var name="Defense" alias="defense" value="2"/>"#));
    }

    #[test]
    fn test_generate_bean_names_xml() {
        let bean_names = vec!["DamageTrigger", "HealTrigger", "SpawnTrigger"];
        let xml = generate_bean_names_xml(&bean_names, "meta");

        assert!(xml.contains(r#"<module name="meta" comment="bean name set">"#));
        assert!(xml.contains(r#"<bean name="TsClassName">"#));
        assert!(xml.contains(r#"<var name="name" type="string#(set=DamageTrigger,HealTrigger,SpawnTrigger)"/>"#));
        // TsClassNames with list type
        assert!(xml.contains(r#"<bean name="TsClassNames">"#));
        assert!(xml.contains(r#"<var name="names" type="list,string#(set=DamageTrigger,HealTrigger,SpawnTrigger)"/>"#));
    }

    #[test]
    fn test_generate_flags_enum() {
        use crate::parser::{EnumInfo, EnumVariant};

        let enum_info = EnumInfo {
            name: "UnitFlag".to_string(),
            comment: Some("权限控制".to_string()),
            is_string_enum: false,
            is_flags: true,
            variants: vec![
                EnumVariant {
                    name: "CAN_MOVE".to_string(),
                    alias: "移动".to_string(),
                    value: 1,
                    comment: Some("可以移动".to_string()),
                },
                EnumVariant {
                    name: "CAN_ATTACK".to_string(),
                    alias: "攻击".to_string(),
                    value: 2,
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
        // Should use custom alias
        assert!(xml.contains(r#"<var name="CAN_MOVE" alias="移动" value="1" comment="可以移动"/>"#));
        assert!(xml.contains(r#"<var name="CAN_ATTACK" alias="攻击" value="2" comment="可以攻击"/>"#));
    }
}
