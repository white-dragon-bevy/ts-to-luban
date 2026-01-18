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

    /// Generate beans.ts with all classes (not interfaces)
    pub fn generate(
        &self,
        all_classes: &[&ClassInfo],
        output_path: &Path,
        default_module: &str,
    ) -> String {
        let mut lines = Vec::new();

        // Filter to only classes (not interfaces) and deduplicate by name
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
