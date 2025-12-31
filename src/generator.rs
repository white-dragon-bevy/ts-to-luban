use crate::parser::{ClassInfo, FieldInfo};
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

    pub fn generate(&self, classes: &[ClassInfo]) -> String {
        let mut lines = vec![
            r#"<?xml version="1.0" encoding="utf-8"?>"#.to_string(),
            r#"<module name="" comment="自动生成的 ts class Bean 定义">"#.to_string(),
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

// Convenience function for simple cases
pub fn generate_xml(classes: &[ClassInfo], default_base: &str) -> String {
    let mappings = vec![];
    let base_resolver = BaseClassResolver::new(&mappings, default_base);
    let type_mapper = TypeMapper::new(&std::collections::HashMap::new());
    let generator = XmlGenerator::new(&base_resolver, &type_mapper);
    generator.generate(classes)
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
        };

        let xml = generate_xml(&[class], "TsClass");
        // List types should NOT have ? suffix even when optional
        assert!(xml.contains(r#"type="list,string""#));
        assert!(!xml.contains(r#"type="list,string?""#));
    }

    #[test]
    fn test_interface_no_parent() {
        let class = ClassInfo {
            name: "MyInterface".to_string(),
            comment: None,
            fields: vec![make_field("value", "int", false)],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "abc123".to_string(),
            is_interface: true,
        };

        let xml = generate_xml(&[class], "TsClass");
        assert!(xml.contains(r#"<bean name="MyInterface">"#));
        assert!(!xml.contains("parent="));
    }

    #[test]
    fn test_xml_escape() {
        assert_eq!(escape_xml("a < b & c > d"), "a &lt; b &amp; c &gt; d");
        assert_eq!(escape_xml(r#"say "hello""#), r#"say &quot;hello&quot;"#);
    }
}
