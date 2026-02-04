use crate::parser::ClassInfo;
use crate::ts_generator::import_resolver::ImportResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Maximum number of imports per file to stay under Luau's 200 register limit
const MAX_IMPORTS_PER_FILE: usize = 100;

/// Represents a generated beans file
pub struct BeansFile {
    /// File name (e.g., "beans.ts", "beans_1.ts")
    pub filename: String,
    /// File content
    pub content: String,
}

/// Beans generator - generates a dictionary of all classes with module.name keys
/// Splits into multiple files to avoid Luau's 200 register limit
pub struct BeansGenerator<'a> {
    import_resolver: &'a ImportResolver,
}

impl<'a> BeansGenerator<'a> {
    pub fn new(import_resolver: &'a ImportResolver) -> Self {
        Self { import_resolver }
    }

    /// Generate beans files - returns multiple files if needed to avoid register limit
    /// Returns: Vec of (filename, content) pairs
    pub fn generate(
        &self,
        all_classes: &[&ClassInfo],
        output_path: &Path,
        default_module: &str,
    ) -> Vec<BeansFile> {
        // Only include classes (not interfaces), deduplicate by name
        let mut seen = std::collections::HashSet::new();
        let classes: Vec<_> = all_classes
            .iter()
            .filter(|c| !c.is_interface && seen.insert(c.name.clone()))
            .copied()
            .collect();

        // Collect imports grouped by source file, and count total imports
        let mut imports_by_file: HashMap<String, Vec<&str>> = HashMap::new();
        for class in &classes {
            let source_path = PathBuf::from(&class.source_file);
            let import_path = self.import_resolver.resolve(output_path, &source_path);

            imports_by_file
                .entry(import_path)
                .or_insert_with(Vec::new)
                .push(class.name.as_str());
        }

        // Count total number of imported identifiers
        let total_imports: usize = imports_by_file.values().map(|v| v.len()).sum();

        // If under limit, generate single file (backward compatible)
        if total_imports <= MAX_IMPORTS_PER_FILE {
            let content = self.generate_single_file(&classes, &imports_by_file, default_module);
            return vec![BeansFile {
                filename: "beans.ts".to_string(),
                content,
            }];
        }

        // Need to split into multiple files
        self.generate_split_files(&classes, output_path, default_module)
    }

    /// Generate a single beans.ts file (original behavior)
    fn generate_single_file(
        &self,
        classes: &[&ClassInfo],
        imports_by_file: &HashMap<String, Vec<&str>>,
        default_module: &str,
    ) -> String {
        let mut lines = Vec::new();

        // Generate import statements (sorted for deterministic output)
        let mut sorted_imports: Vec<_> = imports_by_file.iter().collect();
        sorted_imports.sort_by(|a, b| a.0.cmp(b.0));

        for (import_path, class_names) in sorted_imports {
            let mut sorted_names = class_names.clone();
            sorted_names.sort();
            lines.push(format!(
                "import {{ {} }} from \"{}\";",
                sorted_names.join(", "),
                import_path
            ));
        }

        if !lines.is_empty() {
            lines.push(String::new());
        }

        // Generate Beans const object
        lines.push("export const Beans = {".to_string());

        // Collect bean entries and sort them
        let mut bean_entries: Vec<_> = classes
            .iter()
            .map(|class| {
                let key = self.get_bean_key(class, default_module);
                let value = &class.name;
                (key, value)
            })
            .collect();

        bean_entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Generate bean entries
        for (key, value) in bean_entries {
            lines.push(format!("    \"{}\": {},", key, value));
        }

        lines.push("} as const;".to_string());

        lines.join("\n")
    }

    /// Generate split beans files: beans_1.ts, beans_2.ts, ... and beans.ts (merger)
    fn generate_split_files(
        &self,
        classes: &[&ClassInfo],
        output_path: &Path,
        default_module: &str,
    ) -> Vec<BeansFile> {
        let mut result = Vec::new();

        // Sort classes by bean key for deterministic splitting
        let mut sorted_classes: Vec<_> = classes.to_vec();
        sorted_classes.sort_by(|a, b| {
            let key_a = self.get_bean_key(a, default_module);
            let key_b = self.get_bean_key(b, default_module);
            key_a.cmp(&key_b)
        });

        // Split classes into chunks based on import count
        let chunks = self.split_by_import_count(&sorted_classes, output_path);

        // Generate each chunk file (beans_1.ts, beans_2.ts, ...)
        for (chunk_index, chunk_classes) in chunks.iter().enumerate() {
            let chunk_num = chunk_index + 1;
            let filename = format!("beans_{}.ts", chunk_num);
            let export_name = format!("Beans_{}", chunk_num);

            let content = self.generate_chunk_file(
                chunk_classes,
                output_path,
                default_module,
                &export_name,
            );

            result.push(BeansFile { filename, content });
        }

        // Generate main beans.ts that merges all chunks
        let main_content = self.generate_main_file(chunks.len());
        result.push(BeansFile {
            filename: "beans.ts".to_string(),
            content: main_content,
        });

        result
    }

    /// Split classes into chunks, ensuring each chunk stays under the import limit
    fn split_by_import_count<'b>(
        &self,
        classes: &[&'b ClassInfo],
        output_path: &Path,
    ) -> Vec<Vec<&'b ClassInfo>> {
        let mut chunks: Vec<Vec<&'b ClassInfo>> = Vec::new();
        let mut current_chunk: Vec<&'b ClassInfo> = Vec::new();
        let mut current_imports: HashMap<String, usize> = HashMap::new();
        let mut current_import_count: usize = 0;

        for class in classes {
            let source_path = PathBuf::from(&class.source_file);
            let import_path = self.import_resolver.resolve(output_path, &source_path);

            if !current_chunk.is_empty() && current_import_count + 1 > MAX_IMPORTS_PER_FILE {
                // Start a new chunk
                chunks.push(current_chunk);
                current_chunk = Vec::new();
                current_imports.clear();
                current_import_count = 0;
            }

            // Add class to current chunk
            current_chunk.push(class);
            *current_imports.entry(import_path).or_insert(0) += 1;
            current_import_count += 1;
        }

        // Don't forget the last chunk
        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        chunks
    }

    /// Generate a chunk file (beans_N.ts)
    fn generate_chunk_file(
        &self,
        classes: &[&ClassInfo],
        output_path: &Path,
        default_module: &str,
        export_name: &str,
    ) -> String {
        let mut lines = Vec::new();

        // Collect imports grouped by source file
        let mut imports_by_file: HashMap<String, Vec<&str>> = HashMap::new();
        for class in classes {
            let source_path = PathBuf::from(&class.source_file);
            let import_path = self.import_resolver.resolve(output_path, &source_path);

            imports_by_file
                .entry(import_path)
                .or_insert_with(Vec::new)
                .push(class.name.as_str());
        }

        // Generate import statements (sorted for deterministic output)
        let mut sorted_imports: Vec<_> = imports_by_file.iter().collect();
        sorted_imports.sort_by(|a, b| a.0.cmp(b.0));

        for (import_path, class_names) in sorted_imports {
            let mut sorted_names = class_names.clone();
            sorted_names.sort();
            lines.push(format!(
                "import {{ {} }} from \"{}\";",
                sorted_names.join(", "),
                import_path
            ));
        }

        if !lines.is_empty() {
            lines.push(String::new());
        }

        // Generate export const
        lines.push(format!("export const {} = {{", export_name));

        // Collect bean entries and sort them
        let mut bean_entries: Vec<_> = classes
            .iter()
            .map(|class| {
                let key = self.get_bean_key(class, default_module);
                let value = &class.name;
                (key, value)
            })
            .collect();

        bean_entries.sort_by(|a, b| a.0.cmp(&b.0));

        // Generate bean entries
        for (key, value) in bean_entries {
            lines.push(format!("    \"{}\": {},", key, value));
        }

        lines.push("} as const;".to_string());

        lines.join("\n")
    }

    /// Generate main beans.ts that merges all chunk files
    fn generate_main_file(&self, chunk_count: usize) -> String {
        let mut lines = Vec::new();

        // Import all chunks
        for i in 1..=chunk_count {
            lines.push(format!(
                "import {{ Beans_{} }} from \"./beans_{}\";",
                i, i
            ));
        }

        lines.push(String::new());

        // Merge all chunks into Beans
        lines.push("export const Beans = {".to_string());
        for i in 1..=chunk_count {
            lines.push(format!("    ...Beans_{},", i));
        }
        lines.push("} as const;".to_string());

        lines.join("\n")
    }

    /// Get module name for a class
    fn get_module_name(&self, class: &ClassInfo, default_module: &str) -> String {
        class
            .module_name
            .as_deref()
            .unwrap_or(default_module)
            .to_string()
    }

    /// Get bean key with module prefix
    fn get_bean_key(&self, class: &ClassInfo, default_module: &str) -> String {
        let module = self.get_module_name(class, default_module);
        if module.is_empty() {
            class.name.clone()
        } else {
            format!("{}.{}", module, class.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::ClassInfo;

    fn make_class(name: &str, is_interface: bool) -> ClassInfo {
        make_class_with_source(name, is_interface, "test.ts")
    }

    fn make_class_with_source(name: &str, is_interface: bool, source_file: &str) -> ClassInfo {
        ClassInfo {
            name: name.to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: source_file.to_string(),
            file_hash: "".to_string(),
            is_interface,
            output_path: None,
            module_name: Some("test".to_string()),
            type_params: HashMap::new(),
            luban_table: None,
            table_config: None,
            input_path: None,
            imports: HashMap::new(),
        }
    }

    #[test]
    fn test_beans_generator_excludes_interfaces() {
        let import_resolver = ImportResolver::default();
        let generator = BeansGenerator::new(&import_resolver);

        let class1 = make_class("MyClass", false);
        let interface1 = make_class("MyInterface", true);
        let class2 = make_class("AnotherClass", false);

        let all_classes: Vec<&ClassInfo> = vec![&class1, &interface1, &class2];
        let files = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

        // Should generate single file for small number of classes
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].filename, "beans.ts");

        let output = &files[0].content;

        // Should include classes
        assert!(output.contains("MyClass"), "Should include MyClass");
        assert!(output.contains("AnotherClass"), "Should include AnotherClass");

        // Should NOT include interfaces
        assert!(
            !output.contains("MyInterface"),
            "Should NOT include MyInterface"
        );
    }

    #[test]
    fn test_beans_generator_only_interfaces_produces_empty_beans() {
        let import_resolver = ImportResolver::default();
        let generator = BeansGenerator::new(&import_resolver);

        let interface1 = make_class("Interface1", true);
        let interface2 = make_class("Interface2", true);

        let all_classes: Vec<&ClassInfo> = vec![&interface1, &interface2];
        let files = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

        assert_eq!(files.len(), 1);
        let output = &files[0].content;

        // Should have empty Beans object
        assert!(output.contains("export const Beans = {"));
        assert!(output.contains("} as const;"));
        // Should not have any imports or entries
        assert!(!output.contains("import"));
        assert!(!output.contains("Interface1"));
        assert!(!output.contains("Interface2"));
    }

    #[test]
    fn test_beans_generator_splits_when_over_limit() {
        let import_resolver = ImportResolver::default();
        let generator = BeansGenerator::new(&import_resolver);

        // Create 200 classes (over the 150 limit)
        let classes: Vec<ClassInfo> = (0..200)
            .map(|i| make_class_with_source(&format!("Class{}", i), false, &format!("file{}.ts", i)))
            .collect();

        let all_classes: Vec<&ClassInfo> = classes.iter().collect();
        let files = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

        // Should generate multiple files
        assert!(files.len() > 1, "Should generate multiple files for 200 classes");

        // Should have beans_1.ts, beans_2.ts, ... and beans.ts
        let filenames: Vec<_> = files.iter().map(|f| f.filename.as_str()).collect();
        assert!(filenames.contains(&"beans.ts"), "Should have main beans.ts");
        assert!(filenames.contains(&"beans_1.ts"), "Should have beans_1.ts");

        // Main beans.ts should import and spread chunks
        let main_file = files.iter().find(|f| f.filename == "beans.ts").unwrap();
        assert!(main_file.content.contains("import { Beans_1 }"), "Should import Beans_1");
        assert!(main_file.content.contains("...Beans_1"), "Should spread Beans_1");

        // Chunk files should export Beans_N
        let chunk1 = files.iter().find(|f| f.filename == "beans_1.ts").unwrap();
        assert!(chunk1.content.contains("export const Beans_1 = {"), "Should export Beans_1");
    }

    #[test]
    fn test_beans_generator_no_split_under_limit() {
        let import_resolver = ImportResolver::default();
        let generator = BeansGenerator::new(&import_resolver);

        // Create 50 classes (under the 150 limit)
        let classes: Vec<ClassInfo> = (0..50)
            .map(|i| make_class_with_source(&format!("Class{}", i), false, &format!("file{}.ts", i)))
            .collect();

        let all_classes: Vec<&ClassInfo> = classes.iter().collect();
        let files = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

        // Should generate single file
        assert_eq!(files.len(), 1, "Should generate single file for 50 classes");
        assert_eq!(files[0].filename, "beans.ts");

        // Should NOT have chunk imports
        assert!(!files[0].content.contains("Beans_1"), "Should not have chunk references");
    }
}
