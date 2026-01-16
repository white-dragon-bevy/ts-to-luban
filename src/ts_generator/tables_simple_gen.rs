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
            let table_name = format!("{}Table", class.name);
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
