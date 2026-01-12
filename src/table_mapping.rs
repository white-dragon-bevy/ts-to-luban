use crate::config::TableMapping;
use regex::Regex;

pub struct TableMappingResolver {
    mappings: Vec<(Regex, TableMapping)>,
}

impl TableMappingResolver {
    pub fn new(mappings: &[TableMapping]) -> Self {
        let compiled: Vec<_> = mappings
            .iter()
            .filter_map(|m| Regex::new(&m.pattern).ok().map(|r| (r, m.clone())))
            .collect();
        Self { mappings: compiled }
    }

    /// Resolve class name to (input, output, table_name) paths
    /// Replaces {name} placeholder with kebab-case class name (e.g. TbItem -> tb-item) for input/output
    /// Replaces {name} placeholder with original class name for table_name
    pub fn resolve(&self, class_name: &str) -> Option<(String, Option<String>, Option<String>)> {
        for (regex, mapping) in &self.mappings {
            if regex.is_match(class_name) {
                let name_kebab = Self::to_kebab_case(class_name);
                let input = mapping.input.replace("{name}", &name_kebab);
                let output = mapping
                    .output
                    .as_ref()
                    .map(|o| o.replace("{name}", &name_kebab));
                let table_name = mapping
                    .table_name
                    .as_ref()
                    .map(|t| t.replace("{name}", class_name));
                return Some((input, output, table_name));
            }
        }
        None
    }

    fn to_kebab_case(s: &str) -> String {
        let mut result = String::new();
        for (i, c) in s.char_indices() {
            if c.is_uppercase() {
                if i > 0 {
                    result.push('-');
                }
                result.extend(c.to_lowercase());
            } else {
                result.push(c);
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::TableMapping;

    #[test]
    fn test_to_kebab_case() {
        assert_eq!(TableMappingResolver::to_kebab_case("TbItem"), "tb-item");
        assert_eq!(TableMappingResolver::to_kebab_case("MyConfig"), "my-config");
        assert_eq!(
            TableMappingResolver::to_kebab_case("HTMLParser"),
            "h-t-m-l-parser"
        ); // Simple implementation behavior
        assert_eq!(TableMappingResolver::to_kebab_case("simple"), "simple");
    }

    #[test]
    fn test_resolve_with_kebab_case() {
        let mappings = vec![TableMapping {
            pattern: "Tb.*".to_string(),
            input: "datas/{name}.xlsx".to_string(),
            output: Some("tables/{name}".to_string()),
            table_name: None,
        }];
        let resolver = TableMappingResolver::new(&mappings);

        let (input, output, table_name) = resolver.resolve("TbItem").unwrap();
        assert_eq!(input, "datas/tb-item.xlsx");
        assert_eq!(output, Some("tables/tb-item".to_string()));
        assert_eq!(table_name, None);
    }

    #[test]
    fn test_resolve_with_table_name_override() {
        let mappings = vec![TableMapping {
            pattern: "Tb.*".to_string(),
            input: "datas/{name}.xlsx".to_string(),
            output: None,
            table_name: Some("New{name}Table".to_string()),
        }];
        let resolver = TableMappingResolver::new(&mappings);

        let (input, _, table_name) = resolver.resolve("TbItem").unwrap();
        assert_eq!(input, "datas/tb-item.xlsx");
        assert_eq!(table_name, Some("NewTbItemTable".to_string()));
    }
}
