use crate::parser::ClassInfo;
use crate::ts_generator::import_resolver::ImportResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Simple tables generator - only generates type definitions
pub struct TablesSimpleGenerator<'a> {
    import_resolver: &'a ImportResolver,
}

impl<'a> TablesSimpleGenerator<'a> {
    pub fn new(import_resolver: &'a ImportResolver) -> Self {
        Self { import_resolver }
    }

    /// Generate tables.ts with type definitions only
    pub fn generate(
        &self,
        table_classes: &[&ClassInfo],
        output_path: &Path,
    ) -> String {
        let mut lines = Vec::new();

        // Collect imports
        let mut imports = HashMap::new();
        for class in table_classes {
            let source_path = PathBuf::from(&class.source_file);
            let import_path = self
                .import_resolver
                .resolve(output_path, &source_path);

            imports
                .entry(import_path)
                .or_insert_with(Vec::new)
                .push(class.name.as_str());
        }

        // Generate import statements
        for (import_path, class_names) in imports {
            lines.push(format!(
                "import {{ {} }} from \"{}\";",
                class_names.join(", "),
                import_path
            ));
        }

        if !lines.is_empty() {
            lines.push(String::new());
        }

        // Generate AllTables interface
        lines.push("export interface AllTables {".to_string());
        for class in table_classes {
            let config = class.luban_table.as_ref().unwrap();
            let table_name = config
                .table_name
                .as_ref()
                .cloned()
                .unwrap_or_else(|| format!("{}Table", class.name));
            let type_def = self.generate_table_type(&class.name, &config.mode, &config.index);
            lines.push(format!("    {}: {};", table_name, type_def));
        }
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate table type based on mode
    fn generate_table_type(&self, class_name: &str, mode: &str, _index: &str) -> String {
        match mode {
            "map" => {
                // Determine key type from index field
                // For now, assume number (could be enhanced to parse actual field type)
                format!("Map<number, {}>", class_name)
            }
            "list" => {
                format!("{}[]", class_name)
            }
            "one" | "singleton" => {
                class_name.to_string()
            }
            _ => {
                // Default to map
                format!("Map<number, {}>", class_name)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{ClassInfo, LubanTableConfig};
    use crate::ts_generator::ImportResolver;
    use crate::tsconfig::TsConfig;
    use std::path::PathBuf;

    #[test]
    fn test_custom_table_name_override() {
        let resolver = ImportResolver::new(&TsConfig::default());
        let gen = TablesSimpleGenerator::new(&resolver);

        let mut config = LubanTableConfig::default();
        config.mode = "map".to_string();
        config.index = "id".to_string();
        config.table_name = Some("CustomTableName".to_string());

        let class = ClassInfo {
            name: "IBuffData".to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "hash".to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params: Default::default(),
            luban_table: Some(config),
        };

        let output_path = PathBuf::from("out/tables.d.ts");
        let content = gen.generate(&[&class], &output_path);

        assert!(content.contains("CustomTableName: Map<number, IBuffData>"));
        assert!(!content.contains("IBuffDataTable"));
    }

    #[test]
    fn test_default_table_name_when_no_override() {
        let resolver = ImportResolver::new(&TsConfig::default());
        let gen = TablesSimpleGenerator::new(&resolver);

        let mut config = LubanTableConfig::default();
        config.mode = "map".to_string();
        config.index = "id".to_string();
        config.table_name = None;

        let class = ClassInfo {
            name: "MyConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
            file_hash: "hash".to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
            type_params: Default::default(),
            luban_table: Some(config),
        };

        let output_path = PathBuf::from("out/tables.d.ts");
        let content = gen.generate(&[&class], &output_path);

        assert!(content.contains("MyConfigTable: Map<number, MyConfig>"));
    }
}
