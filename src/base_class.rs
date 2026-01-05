use crate::parser::ClassInfo;
use crate::config::ParentMapping;
use regex::Regex;

pub struct BaseClassResolver<'a> {
    default_base: &'a str,
    parent_mappings: Vec<(Regex, String)>,
}

impl<'a> BaseClassResolver<'a> {
    pub fn new(default_base: &'a str, mappings: &[ParentMapping]) -> Self {
        let parent_mappings = mappings
            .iter()
            .filter_map(|m| {
                Regex::new(&m.pattern)
                    .ok()
                    .map(|re| (re, m.parent.clone()))
            })
            .collect();

        Self {
            default_base,
            parent_mappings,
        }
    }

    pub fn resolve(&self, class_info: &ClassInfo) -> String {
        // Interfaces: only have parent if they extend another interface
        if class_info.is_interface {
            return class_info.extends.clone().unwrap_or_default();
        }

        // Check parent_mappings regex patterns (config takes priority)
        for (pattern, parent) in &self.parent_mappings {
            if pattern.is_match(&class_info.name) {
                return parent.clone();
            }
        }

        // Otherwise use default base class
        self.default_base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ClassInfo;

    fn make_class(name: &str) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: String::new(),
            file_hash: String::new(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_class_uses_default_parent() {
        let resolver = BaseClassResolver::new("TsClass", &[]);
        let class = make_class("MyClass");
        assert_eq!(resolver.resolve(&class), "TsClass");
    }

    #[test]
    fn test_class_extends_ignored() {
        // extends is ignored, config takes priority
        let resolver = BaseClassResolver::new("TsClass", &[]);
        let mut class = make_class("ChildClass");
        class.extends = Some("ParentClass".to_string());
        // Should use default, not extends
        assert_eq!(resolver.resolve(&class), "TsClass");
    }

    #[test]
    fn test_interface_no_extends_no_parent() {
        let resolver = BaseClassResolver::new("TsClass", &[]);
        let mut iface = make_class("MyInterface");
        iface.is_interface = true;
        assert_eq!(resolver.resolve(&iface), "");
    }

    #[test]
    fn test_interface_with_extends_has_parent() {
        let resolver = BaseClassResolver::new("TsClass", &[]);
        let mut iface = make_class("ChildInterface");
        iface.is_interface = true;
        iface.extends = Some("ParentInterface".to_string());
        assert_eq!(resolver.resolve(&iface), "ParentInterface");
    }

    #[test]
    fn test_parent_mapping_regex() {
        let mappings = vec![
            ParentMapping {
                pattern: ".*Trigger$".to_string(),
                parent: "TsTriggerClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new("TsClass", &mappings);

        let trigger = make_class("DamageTrigger");
        assert_eq!(resolver.resolve(&trigger), "TsTriggerClass");

        let other = make_class("SomeComponent");
        assert_eq!(resolver.resolve(&other), "TsClass");
    }

    #[test]
    fn test_config_takes_priority_over_extends() {
        let mappings = vec![
            ParentMapping {
                pattern: ".*Trigger.*".to_string(),
                parent: "TsTriggerClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new("TsClass", &mappings);

        let mut trigger = make_class("HealTrigger2");
        trigger.extends = Some("HealTrigger".to_string());
        // config mapping should take priority over extends
        assert_eq!(resolver.resolve(&trigger), "TsTriggerClass");
    }
}
