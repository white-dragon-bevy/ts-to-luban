use crate::config::BaseClassMapping;
use crate::parser::ClassInfo;

pub struct BaseClassResolver<'a> {
    mappings: &'a [BaseClassMapping],
    default_base: &'a str,
}

impl<'a> BaseClassResolver<'a> {
    pub fn new(mappings: &'a [BaseClassMapping], default_base: &'a str) -> Self {
        Self { mappings, default_base }
    }

    pub fn resolve(&self, class_info: &ClassInfo) -> String {
        // Interfaces don't have a parent class
        if class_info.is_interface {
            return String::new();
        }

        // Check implements clause for matching interface
        for iface in &class_info.implements {
            for mapping in self.mappings {
                if &mapping.interface == iface {
                    return mapping.maps_to.clone();
                }
            }
        }

        // Use default base class
        self.default_base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ClassInfo;

    fn make_class(name: &str, implements: Vec<&str>) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            comment: None,
            fields: vec![],
            implements: implements.into_iter().map(|s| s.to_string()).collect(),
            extends: None,
            source_file: String::new(),
            file_hash: String::new(),
            is_interface: false,
        }
    }

    #[test]
    fn test_resolve_from_implements() {
        let mappings = vec![
            BaseClassMapping {
                interface: "EntityTrigger".to_string(),
                maps_to: "TsTriggerClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let class = make_class("MyTrigger", vec!["EntityTrigger"]);
        assert_eq!(resolver.resolve(&class), "TsTriggerClass");
    }

    #[test]
    fn test_resolve_default() {
        let mappings = vec![];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let class = make_class("MyClass", vec![]);
        assert_eq!(resolver.resolve(&class), "TsClass");
    }

    #[test]
    fn test_interface_no_parent() {
        let mappings = vec![];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let mut iface = make_class("MyInterface", vec![]);
        iface.is_interface = true;

        assert_eq!(resolver.resolve(&iface), "");
    }

    #[test]
    fn test_multiple_mappings() {
        let mappings = vec![
            BaseClassMapping {
                interface: "EntityTrigger".to_string(),
                maps_to: "TsTriggerClass".to_string(),
            },
            BaseClassMapping {
                interface: "Component".to_string(),
                maps_to: "TsComponentClass".to_string(),
            },
        ];
        let resolver = BaseClassResolver::new(&mappings, "TsClass");

        let trigger = make_class("MyTrigger", vec!["EntityTrigger"]);
        let component = make_class("MyComponent", vec!["Component"]);

        assert_eq!(resolver.resolve(&trigger), "TsTriggerClass");
        assert_eq!(resolver.resolve(&component), "TsComponentClass");
    }
}
