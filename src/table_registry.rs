use crate::config::TableConfig;
use std::collections::HashMap;

/// Resolved table configuration with all defaults applied
#[derive(Debug, Clone)]
pub struct ResolvedTableConfig {
    /// Full bean name with module prefix (e.g., "role.RoleConfig")
    pub bean: String,
    /// Table name (e.g., "RoleConfigTable" or custom name)
    pub name: String,
    /// Input path (e.g., "../datas/role")
    pub input: String,
    /// Table mode: "map" | "list" | "one" | "singleton"
    pub mode: String,
    /// Index field (e.g., "id")
    pub index: String,
    /// Index field type (e.g., "int", "string") - used by @Ref/@RefKey
    pub index_type: Option<String>,
    /// Module name (e.g., "role")
    pub module: String,
    /// Class name without module prefix (e.g., "RoleConfig")
    pub class_name: String,
}

#[derive(Debug, Clone)]
pub struct TableEntry {
    pub namespace: String,
    /// Bean name (class name as-is)
    pub bean_name: String,
    /// Table name (class name + "Table" suffix)
    pub table_name: String,
    /// Full table reference for @Ref (namespace.TableName)
    pub full_table_ref: String,
    /// Index field type (e.g., "int", "string") - used by @Ref/@RefKey
    pub index_type: Option<String>,
}

#[derive(Debug, Default)]
pub struct TableRegistry {
    /// Map from class name (without module) to TableEntry (for @Ref resolution)
    entries: HashMap<String, TableEntry>,
    /// Map from full bean name (module.ClassName) to ResolvedTableConfig
    tables: HashMap<String, ResolvedTableConfig>,
}

impl TableRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Build registry from [tables] config
    pub fn from_config(tables_config: &HashMap<String, TableConfig>) -> Self {
        let mut registry = Self::new();

        for (full_name, config) in tables_config {
            // Parse "module.ClassName" format
            let (module, class_name) = if let Some(dot_pos) = full_name.rfind('.') {
                (
                    full_name[..dot_pos].to_string(),
                    full_name[dot_pos + 1..].to_string(),
                )
            } else {
                // No module prefix
                (String::new(), full_name.clone())
            };

            // Build table name: custom name or default "{ClassName}Table"
            let table_name = config
                .name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}Table", class_name));

            // Build full table reference for @Ref
            let full_table_ref = if module.is_empty() {
                table_name.clone()
            } else {
                format!("{}.{}", module, table_name)
            };

            let resolved = ResolvedTableConfig {
                bean: full_name.clone(),
                name: table_name.clone(),
                input: config.input().to_string(),
                mode: config.mode().to_string(),
                index: config.index().to_string(),
                index_type: None, // Will be set later by set_index_types
                module: module.clone(),
                class_name: class_name.clone(),
            };

            // Register in tables map (by full name)
            registry.tables.insert(full_name.clone(), resolved);

            // Register in entries map (by class name only, for @Ref resolution)
            registry.entries.insert(
                class_name.clone(),
                TableEntry {
                    namespace: module,
                    bean_name: class_name,
                    table_name,
                    full_table_ref,
                    index_type: None, // Will be set later by set_index_types
                },
            );
        }

        registry
    }

    /// Register a @LubanTable class (legacy method for backward compatibility)
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
                index_type: None,
            },
        );
    }

    /// Set index types for all registered tables based on parsed class information
    /// This should be called after parsing all TypeScript files
    pub fn set_index_types(&mut self, classes: &[crate::parser::ClassInfo], type_mapper: &crate::type_mapper::TypeMapper) {
        use std::collections::HashMap as StdHashMap;
        
        // Build a map from class name to its fields
        let class_fields: StdHashMap<&str, &[crate::parser::FieldInfo]> = classes
            .iter()
            .map(|c| (c.name.as_str(), c.fields.as_slice()))
            .collect();

        // Update index_type for each table
        for (_full_name, config) in &mut self.tables {
            let class_name = &config.class_name;
            if let Some(fields) = class_fields.get(class_name.as_str()) {
                // Find the index field
                if let Some(field) = fields.iter().find(|f| f.name == config.index) {
                    // Map the TypeScript type to Luban type
                    let mapped_type = type_mapper.map_full_type(&field.field_type);
                    config.index_type = Some(mapped_type.clone());
                    
                    // Also update the entry
                    if let Some(entry) = self.entries.get_mut(class_name) {
                        entry.index_type = Some(mapped_type);
                    }
                }
            }
        }
    }

    /// Get index type for a class (for @Ref resolution)
    pub fn get_index_type(&self, class_name: &str) -> Option<&str> {
        self.entries.get(class_name).and_then(|e| e.index_type.as_deref())
    }

    /// Get table entry by class name (for @Ref resolution)
    pub fn get(&self, class_name: &str) -> Option<&TableEntry> {
        self.entries.get(class_name)
    }

    /// Get resolved table config by full bean name (module.ClassName)
    pub fn get_table(&self, full_name: &str) -> Option<&ResolvedTableConfig> {
        self.tables.get(full_name)
    }

    /// Get resolved table config by class name only (searches all modules)
    pub fn get_table_by_class(&self, class_name: &str) -> Option<&ResolvedTableConfig> {
        // First try exact match with full name
        if let Some(config) = self.tables.get(class_name) {
            return Some(config);
        }

        // Then search by class name
        self.tables
            .values()
            .find(|config| config.class_name == class_name)
    }

    /// Get all registered tables
    pub fn all_tables(&self) -> impl Iterator<Item = &ResolvedTableConfig> {
        self.tables.values()
    }

    /// Check if a class is registered as a table
    pub fn is_table(&self, class_name: &str) -> bool {
        self.entries.contains_key(class_name)
    }

    /// Check if a full bean name is registered as a table
    pub fn has_table(&self, full_name: &str) -> bool {
        self.tables.contains_key(full_name)
    }

    /// Resolve @Ref(ClassName) to full table reference (e.g., "examples.ItemTable")
    pub fn resolve_ref(&self, class_name: &str) -> Option<String> {
        self.get(class_name).map(|e| e.full_table_ref.clone())
    }

    /// Validate that all configured tables have corresponding beans
    /// Returns a list of missing bean names (tables configured but beans not found)
    pub fn validate_beans_exist<'a>(
        &self,
        existing_beans: &'a std::collections::HashSet<String>,
    ) -> Vec<&str> {
        self.tables
            .keys()
            .filter(|full_name| !existing_beans.contains(*full_name))
            .map(|s| s.as_str())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_config_simple() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        // Check table lookup by full name
        let table = registry.get_table("role.RoleConfig").unwrap();
        assert_eq!(table.bean, "role.RoleConfig");
        assert_eq!(table.name, "RoleConfigTable");
        assert_eq!(table.input, "../datas/role");
        assert_eq!(table.mode, "map");
        assert_eq!(table.index, "id");
        assert_eq!(table.module, "role");
        assert_eq!(table.class_name, "RoleConfig");

        // Check @Ref resolution
        let ref_target = registry.resolve_ref("RoleConfig").unwrap();
        assert_eq!(ref_target, "role.RoleConfigTable");
    }

    #[test]
    fn test_from_config_full() {
        let mut config = HashMap::new();
        config.insert(
            "battle.BattleData".to_string(),
            TableConfig::Full {
                input: "../datas/battle".to_string(),
                name: Some("TbBattle".to_string()),
                mode: Some("one".to_string()),
                index: Some("battleId".to_string()),
            },
        );

        let registry = TableRegistry::from_config(&config);

        let table = registry.get_table("battle.BattleData").unwrap();
        assert_eq!(table.name, "TbBattle");
        assert_eq!(table.mode, "one");
        assert_eq!(table.index, "battleId");

        // Check @Ref resolution with custom table name
        let ref_target = registry.resolve_ref("BattleData").unwrap();
        assert_eq!(ref_target, "battle.TbBattle");
    }

    #[test]
    fn test_from_config_no_module() {
        let mut config = HashMap::new();
        config.insert(
            "GlobalConfig".to_string(),
            TableConfig::Simple("../datas/global".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        let table = registry.get_table("GlobalConfig").unwrap();
        assert_eq!(table.module, "");
        assert_eq!(table.class_name, "GlobalConfig");
        assert_eq!(table.name, "GlobalConfigTable");

        // @Ref without module prefix
        let ref_target = registry.resolve_ref("GlobalConfig").unwrap();
        assert_eq!(ref_target, "GlobalConfigTable");
    }

    #[test]
    fn test_get_table_by_class() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        // Can find by class name only
        let table = registry.get_table_by_class("RoleConfig").unwrap();
        assert_eq!(table.bean, "role.RoleConfig");
    }

    #[test]
    fn test_is_table() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        assert!(registry.is_table("RoleConfig"));
        assert!(!registry.is_table("NonExistent"));
    }

    #[test]
    fn test_has_table() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        assert!(registry.has_table("role.RoleConfig"));
        assert!(!registry.has_table("role.NonExistent"));
    }

    #[test]
    fn test_legacy_register() {
        let mut registry = TableRegistry::new();
        registry.register("Item", "examples");

        let ref_target = registry.resolve_ref("Item").unwrap();
        assert_eq!(ref_target, "examples.ItemTable");
    }

    #[test]
    fn test_validate_beans_exist_all_present() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );
        config.insert(
            "battle.BattleData".to_string(),
            TableConfig::Simple("../datas/battle".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        let mut existing_beans = std::collections::HashSet::new();
        existing_beans.insert("role.RoleConfig".to_string());
        existing_beans.insert("battle.BattleData".to_string());

        let missing = registry.validate_beans_exist(&existing_beans);
        assert!(missing.is_empty());
    }

    #[test]
    fn test_validate_beans_exist_some_missing() {
        let mut config = HashMap::new();
        config.insert(
            "role.RoleConfig".to_string(),
            TableConfig::Simple("../datas/role".to_string()),
        );
        config.insert(
            "battle.BattleData".to_string(),
            TableConfig::Simple("../datas/battle".to_string()),
        );
        config.insert(
            "UnitFlagsPreset".to_string(),
            TableConfig::Simple("../datas/unit-flags".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        let mut existing_beans = std::collections::HashSet::new();
        existing_beans.insert("role.RoleConfig".to_string());
        // battle.BattleData and UnitFlagsPreset are missing

        let missing = registry.validate_beans_exist(&existing_beans);
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"battle.BattleData"));
        assert!(missing.contains(&"UnitFlagsPreset"));
    }

    #[test]
    fn test_validate_beans_exist_no_module() {
        let mut config = HashMap::new();
        config.insert(
            "GlobalConfig".to_string(),
            TableConfig::Simple("../datas/global".to_string()),
        );

        let registry = TableRegistry::from_config(&config);

        // Bean exists without module prefix
        let mut existing_beans = std::collections::HashSet::new();
        existing_beans.insert("GlobalConfig".to_string());

        let missing = registry.validate_beans_exist(&existing_beans);
        assert!(missing.is_empty());

        // Bean does not exist
        let empty_beans: std::collections::HashSet<String> = std::collections::HashSet::new();
        let missing = registry.validate_beans_exist(&empty_beans);
        assert_eq!(missing.len(), 1);
        assert!(missing.contains(&"GlobalConfig"));
    }
}
