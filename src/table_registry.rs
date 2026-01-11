use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct TableEntry {
    pub namespace: String,
    /// Bean name (class name as-is)
    pub bean_name: String,
    /// Table name (class name + "Table" suffix)
    pub table_name: String,
    /// Full table reference for @Ref (namespace.TableName)
    pub full_table_ref: String,
}

#[derive(Debug, Default)]
pub struct TableRegistry {
    /// Map from class name to TableEntry
    entries: HashMap<String, TableEntry>,
}

impl TableRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a @LubanTable class
    /// class_name: the TypeScript class name (e.g., "Item")
    /// namespace: the module name (e.g., "examples")
    pub fn register(&mut self, class_name: &str, namespace: &str) {
        let bean_name = class_name.to_string();
        let table_name = format!("{}Table", class_name);

        let full_table_ref = if namespace.is_empty() {
            table_name.clone()
        } else {
            format!("{}.{}", namespace, table_name)
        };

        self.entries.insert(
            class_name.to_string(),
            TableEntry {
                namespace: namespace.to_string(),
                bean_name,
                table_name,
                full_table_ref,
            },
        );
    }

    pub fn get(&self, class_name: &str) -> Option<&TableEntry> {
        self.entries.get(class_name)
    }

    /// Resolve @Ref(ClassName) to full table reference (e.g., "examples.ItemTable")
    pub fn resolve_ref(&self, class_name: &str) -> Option<String> {
        self.get(class_name).map(|e| e.full_table_ref.clone())
    }
}
