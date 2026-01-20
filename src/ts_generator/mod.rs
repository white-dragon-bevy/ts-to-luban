mod beans_gen;
mod import_resolver;
mod tables_simple_gen;

pub use beans_gen::BeansGenerator;
pub use import_resolver::ImportResolver;
pub use tables_simple_gen::TablesSimpleGenerator;

use crate::parser::ClassInfo;
use crate::table_registry::TableRegistry;
use crate::tsconfig::TsConfig;
use std::path::PathBuf;

/// Main TypeScript code generator
pub struct TsCodeGenerator<'a> {
    output_path: PathBuf,
    classes: Vec<ClassInfo>,
    import_resolver: ImportResolver,
    default_module_name: String,
    table_registry: &'a TableRegistry,
}

impl<'a> TsCodeGenerator<'a> {
    pub fn new(
        output_path: PathBuf,
        _project_root: PathBuf,
        classes: Vec<ClassInfo>,
        tsconfig: &TsConfig,
        default_module_name: String,
        table_registry: &'a TableRegistry,
    ) -> Self {
        Self {
            output_path,
            classes,
            import_resolver: ImportResolver::new(tsconfig),
            default_module_name,
            table_registry,
        }
    }

    fn get_default_module_name(&self) -> &str {
        &self.default_module_name
    }

    pub fn generate(&self) -> anyhow::Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.output_path)?;

        // Get table classes from [tables] config
        let table_classes: Vec<_> = self
            .classes
            .iter()
            .filter(|c| {
                // Build full name: module.ClassName
                // Use class's module_name, or fall back to default_module_name
                let module = c
                    .module_name
                    .as_deref()
                    .unwrap_or(&self.default_module_name);
                let full_name = if module.is_empty() {
                    c.name.clone()
                } else {
                    format!("{}.{}", module, c.name)
                };
                // Check if this class is in the [tables] config
                self.table_registry.has_table(&full_name)
            })
            .collect();

        // Generate tables.d.ts with simple type definitions
        let tables_gen =
            TablesSimpleGenerator::new(&self.import_resolver, self.table_registry, &self.default_module_name);
        let tables_path = self.output_path.join("tables.d.ts");
        let content = tables_gen.generate(&table_classes, &tables_path);
        std::fs::write(&tables_path, content)?;

        // Generate beans.ts with all classes
        let all_class_refs: Vec<_> = self.classes.iter().collect();
        let beans_gen = BeansGenerator::new(&self.import_resolver);
        let beans_path = self.output_path.join("beans.ts");
        let beans_content = beans_gen.generate(
            &all_class_refs,
            &beans_path,
            self.get_default_module_name(),
        );
        std::fs::write(&beans_path, beans_content)?;

        Ok(())
    }
}
