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
            let type_def = self.generate_table_type(class, &config.mode, &config.index);
            lines.push(format!("    {}: {};", table_name, type_def));
        }
        lines.push("}".to_string());

        lines.join("\n")
    }

    /// Generate table type based on mode
    fn generate_table_type(&self, class: &ClassInfo, mode: &str, index: &str) -> String {
        let class_name = &class.name;
        match mode {
            "map" => {
                // Determine key type from index field
                let key_type = self.get_index_field_ts_type(class, index);
                format!("Map<{}, {}>", key_type, class_name)
            }
            "list" => {
                format!("{}[]", class_name)
            }
            "one" | "singleton" => {
                class_name.to_string()
            }
            _ => {
                // Default to map with number key
                format!("Map<number, {}>", class_name)
            }
        }
    }

    /// Get TypeScript type for the index field
    fn get_index_field_ts_type(&self, class: &ClassInfo, index: &str) -> &'static str {
        // Find the index field in the class fields
        if let Some(field) = class.fields.iter().find(|f| f.name == index) {
            // Use original_type which is the TypeScript type before mapping
            match field.original_type.as_str() {
                "string" => "string",
                "number" | "int" | "float" | "double" | "long" => "number",
                _ => "number", // Default to number for unknown types
            }
        } else {
            // Default to number if index field not found
            "number"
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

    #[test]
    fn test_map_key_type_from_index_field() {
        use crate::parser::FieldInfo;

        let resolver = ImportResolver::new(&TsConfig::default());
        let gen = TablesSimpleGenerator::new(&resolver);

        let mut config = LubanTableConfig::default();
        config.mode = "map".to_string();
        config.index = "id".to_string();
        config.table_name = None;

        // Create a class with string id field
        let id_field = FieldInfo {
            name: "id".to_string(),
            field_type: "string".to_string(),
            original_type: "string".to_string(),
            ..Default::default()
        };

        let class = ClassInfo {
            name: "AnimationItem".to_string(),
            comment: None,
            alias: None,
            fields: vec![id_field],
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

        // Should use string as key type since id field is string
        assert!(
            content.contains("AnimationItemTable: Map<string, AnimationItem>"),
            "Expected Map<string, AnimationItem>, got: {}",
            content
        );
    }

    #[test]
    fn test_map_key_type_number_for_numeric_index() {
        use crate::parser::FieldInfo;

        let resolver = ImportResolver::new(&TsConfig::default());
        let gen = TablesSimpleGenerator::new(&resolver);

        let mut config = LubanTableConfig::default();
        config.mode = "map".to_string();
        config.index = "id".to_string();
        config.table_name = None;

        // Create a class with number id field
        let id_field = FieldInfo {
            name: "id".to_string(),
            field_type: "double".to_string(),
            original_type: "number".to_string(),
            ..Default::default()
        };

        let class = ClassInfo {
            name: "ItemConfig".to_string(),
            comment: None,
            alias: None,
            fields: vec![id_field],
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

        // Should use number as key type since id field is number
        assert!(
            content.contains("ItemConfigTable: Map<number, ItemConfig>"),
            "Expected Map<number, ItemConfig>, got: {}",
            content
        );
    }
}
