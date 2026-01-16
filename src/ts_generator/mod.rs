mod beans_gen;
mod import_resolver;
mod tables_simple_gen;

pub use beans_gen::BeansGenerator;
pub use import_resolver::ImportResolver;
pub use tables_simple_gen::TablesSimpleGenerator;

use crate::parser::ClassInfo;
use crate::tsconfig::TsConfig;
use anyhow::Context;
use std::path::PathBuf;

/// Main TypeScript code generator
pub struct TsCodeGenerator {
    output_path: PathBuf,
    project_root: PathBuf,
    classes: Vec<ClassInfo>,
    import_resolver: ImportResolver,
    default_module_name: String,
}

impl TsCodeGenerator {
    pub fn new(
        output_path: PathBuf,
        project_root: PathBuf,
        classes: Vec<ClassInfo>,
        tsconfig: &TsConfig,
        default_module_name: String,
    ) -> Self {
        Self {
            output_path,
            project_root,
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

        // Copy schema.d.ts to output directory
        self.copy_schema_file()?;

        Ok(())
    }

    /// Copy schema.d.ts from assets folder to output directory
    fn copy_schema_file(&self) -> anyhow::Result<()> {
        let schema_source = self.project_root.join("assets").join("schema.d.ts");
        let schema_dest = self.output_path.join("schema.d.ts");

        // 检查源文件是否存在
        if !schema_source.exists() {
            eprintln!("  Warning: schema.d.ts source not found at {:?}", schema_source);
            eprintln!("  Skipping schema.d.ts copy (this is OK if you don't use the raw config parser)");
            return Ok(());
        }

        // 复制文件
        std::fs::copy(&schema_source, &schema_dest)
            .context(format!(
                "Failed to copy schema.d.ts from {:?} to {:?}",
                schema_source, schema_dest
            ))?;

        eprintln!("  Copied schema.d.ts to {:?}", schema_dest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_copy_schema_file_success() {
        // 创建临时目录结构
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // 创建 assets 文件夹和源 schema.d.ts
        let assets_dir = project_root.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        let source_file = assets_dir.join("schema.d.ts");
        fs::write(&source_file, "export interface Schema {}").unwrap();

        // 创建输出目录
        let output_path = project_root.join("src").join("types").join("configs");
        fs::create_dir_all(&output_path).unwrap();

        // 创建 generator（最小设置）
        let generator = TsCodeGenerator::new(
            output_path.clone(),
            project_root,
            vec![],
            &TsConfig::default(),
            String::new(),
        );

        // 执行复制
        let result = generator.copy_schema_file();

        // 验证成功
        assert!(result.is_ok());

        // 验证文件已复制
        let dest_file = output_path.join("schema.d.ts");
        assert!(dest_file.exists());
        let content = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "export interface Schema {}");
    }

    #[test]
    fn test_copy_schema_file_missing_source() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // 创建输出目录但不创建源文件
        let output_path = project_root.join("src").join("types").join("configs");
        fs::create_dir_all(&output_path).unwrap();

        let generator = TsCodeGenerator::new(
            output_path.clone(),
            project_root,
            vec![],
            &TsConfig::default(),
            String::new(),
        );

        // 不应失败，只记录警告
        let result = generator.copy_schema_file();
        assert!(result.is_ok());

        // 验证目标文件未创建
        let dest_file = output_path.join("schema.d.ts");
        assert!(!dest_file.exists());
    }

    #[test]
    fn test_copy_schema_file_overwrites_existing() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let project_root = temp_dir.path().to_path_buf();

        // 创建 assets 文件夹和源文件
        let assets_dir = project_root.join("assets");
        fs::create_dir_all(&assets_dir).unwrap();
        let source_file = assets_dir.join("schema.d.ts");
        fs::write(&source_file, "new content").unwrap();

        // 创建输出目录和已有的 schema.d.ts
        let output_path = project_root.join("src").join("types").join("configs");
        fs::create_dir_all(&output_path).unwrap();
        let dest_file = output_path.join("schema.d.ts");
        fs::write(&dest_file, "old content").unwrap();

        let generator = TsCodeGenerator::new(
            output_path.clone(),
            project_root,
            vec![],
            &TsConfig::default(),
            String::new(),
        );

        // 执行复制
        generator.copy_schema_file().unwrap();

        // 验证文件已覆盖
        let content = fs::read_to_string(&dest_file).unwrap();
        assert_eq!(content, "new content");
    }
}
