use crate::parser::ClassInfo;
use crate::ts_generator::import_resolver::ImportResolver;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Beans generator - generates a dictionary of all classes with module.name keys
pub struct BeansGenerator<'a> {
    import_resolver: &'a ImportResolver,
}

impl<'a> BeansGenerator<'a> {
    pub fn new(import_resolver: &'a ImportResolver) -> Self {
        Self { import_resolver }
    }

    /// Generate beans.ts with all classes and interfaces
    pub fn generate(
        &self,
        all_classes: &[&ClassInfo],
        output_path: &Path,
        default_module: &str,
    ) -> String {
        let mut lines = Vec::new();

        // Only include classes (not interfaces), deduplicate by name
        let mut seen = std::collections::HashSet::new();
        let classes: Vec<_> = all_classes
            .iter()
            .filter(|c| !c.is_interface && seen.insert(c.name.clone()))
            .copied()
            .collect();

        // Collect imports grouped by source file
        let mut imports = HashMap::new();
        for class in &classes {
            let source_path = PathBuf::from(&class.source_file);
            let import_path = self.import_resolver.resolve(output_path, &source_path);

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
        ClassInfo {
            name: name.to_string(),
            comment: None,
            alias: None,
            fields: vec![],
            implements: vec![],
            extends: None,
            source_file: "test.ts".to_string(),
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
        let output = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

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
        let output = generator.generate(&all_classes, Path::new("out/beans.ts"), "test");

        // Should have empty Beans object
        assert!(output.contains("export const Beans = {"));
        assert!(output.contains("} as const;"));
        // Should not have any imports or entries
        assert!(!output.contains("import"));
        assert!(!output.contains("Interface1"));
        assert!(!output.contains("Interface2"));
    }
}
