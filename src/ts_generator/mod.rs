mod import_resolver;
mod tables_simple_gen;

pub use import_resolver::ImportResolver;
pub use tables_simple_gen::TablesSimpleGenerator;

use crate::parser::ClassInfo;
use crate::tsconfig::TsConfig;
use std::path::PathBuf;

/// Main TypeScript code generator
pub struct TsCodeGenerator {
    output_path: PathBuf,
    classes: Vec<ClassInfo>,
    import_resolver: ImportResolver,
}

impl TsCodeGenerator {
    pub fn new(output_path: PathBuf, classes: Vec<ClassInfo>, tsconfig: &TsConfig) -> Self {
        Self {
            output_path,
            classes,
            import_resolver: ImportResolver::new(tsconfig),
        }
    }

    pub fn generate(&self) -> anyhow::Result<()> {
        // Create output directory
        std::fs::create_dir_all(&self.output_path)?;

        // Get @LubanTable classes
        let table_classes: Vec<_> = self
            .classes
            .iter()
            .filter(|c| c.luban_table.is_some())
            .collect();

        // Generate tables.ts with simple type definitions
        let tables_gen = TablesSimpleGenerator::new(&self.import_resolver);
        let tables_path = self.output_path.join("tables.ts");
        let content = tables_gen.generate(&table_classes, &tables_path);
        std::fs::write(&tables_path, content)?;

        Ok(())
    }
}
