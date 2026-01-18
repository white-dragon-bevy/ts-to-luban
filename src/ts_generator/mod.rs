mod beans_gen;
mod import_resolver;
mod tables_simple_gen;

pub use beans_gen::BeansGenerator;
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
    default_module_name: String,
}

impl TsCodeGenerator {
    pub fn new(
        output_path: PathBuf,
        _project_root: PathBuf,
        classes: Vec<ClassInfo>,
        tsconfig: &TsConfig,
        default_module_name: String,
    ) -> Self {
        Self {
            output_path,
            classes,
            import_resolver: ImportResolver::new(tsconfig),
            default_module_name,
        }
    }

    fn get_default_module_name(&self) -> &str {
        &self.default_module_name
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

        // Generate tables.d.ts with simple type definitions
        let tables_gen = TablesSimpleGenerator::new(&self.import_resolver);
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
