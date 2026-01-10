mod creator_gen;
mod table_gen;
mod registry_gen;
mod index_gen;
mod import_resolver;

pub use creator_gen::CreatorGenerator;
pub use table_gen::TableGenerator;
pub use registry_gen::RegistryGenerator;
pub use index_gen::IndexGenerator;
pub use import_resolver::ImportResolver;

use std::path::PathBuf;
use crate::parser::ClassInfo;
use crate::tsconfig::TsConfig;

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
        // Create output directories
        std::fs::create_dir_all(self.output_path.join("creators"))?;
        std::fs::create_dir_all(self.output_path.join("tables"))?;

        // Generate registry.ts
        let registry_content = RegistryGenerator::generate();
        std::fs::write(self.output_path.join("registry.ts"), registry_content)?;

        // Generate creators
        let creator_gen = CreatorGenerator::new(&self.import_resolver, &self.classes);
        for class in &self.classes {
            let file_name = format!("{}.ts", to_kebab_case(&class.name));
            let file_path = self.output_path.join("creators").join(&file_name);
            let content = creator_gen.generate(class, &file_path);
            std::fs::write(&file_path, content)?;
        }

        // Generate tables for @LubanTable classes
        let table_classes: Vec<_> = self.classes.iter()
            .filter(|c| c.luban_table.is_some())
            .collect();

        for class in &table_classes {
            let file_name = format!("{}.ts", to_kebab_case(&class.name));
            let file_path = self.output_path.join("tables").join(&file_name);
            let content = TableGenerator::generate(class, &file_path);
            std::fs::write(&file_path, content)?;
        }

        // Generate index.ts
        let index_content = IndexGenerator::generate(&self.classes, &table_classes);
        std::fs::write(self.output_path.join("index.ts"), index_content)?;

        Ok(())
    }
}

/// Convert PascalCase to kebab-case
pub fn to_kebab_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('-');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}
