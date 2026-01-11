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

    /// Resolve class name to (input, output) paths
    /// Replaces {name} placeholder with lowercase class name
    pub fn resolve(&self, class_name: &str) -> Option<(String, Option<String>)> {
        for (regex, mapping) in &self.mappings {
            if regex.is_match(class_name) {
                let name_lower = class_name.to_lowercase();
                let input = mapping.input.replace("{name}", &name_lower);
                let output = mapping
                    .output
                    .as_ref()
                    .map(|o| o.replace("{name}", &name_lower));
                return Some((input, output));
            }
        }
        None
    }
}
