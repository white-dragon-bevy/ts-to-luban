use std::collections::HashMap;

pub struct TypeMapper {
    mappings: HashMap<String, String>,
}

impl TypeMapper {
    pub fn new(custom_mappings: &HashMap<String, String>) -> Self {
        let mut mappings = Self::builtin_mappings();

        // Merge custom mappings (case-insensitive keys)
        for (key, value) in custom_mappings {
            mappings.insert(key.to_lowercase(), value.clone());
        }

        Self { mappings }
    }

    fn builtin_mappings() -> HashMap<String, String> {
        let mut m = HashMap::new();

        // Basic types
        m.insert("number".to_string(), "double".to_string());
        m.insert("string".to_string(), "string".to_string());
        m.insert("boolean".to_string(), "bool".to_string());

        // Numeric types
        m.insert("float".to_string(), "float".to_string());
        m.insert("double".to_string(), "double".to_string());
        m.insert("int".to_string(), "int".to_string());
        m.insert("long".to_string(), "long".to_string());

        // Roblox types
        m.insert("vector3".to_string(), "Vector3".to_string());
        m.insert("vector2".to_string(), "Vector2".to_string());
        m.insert("cframe".to_string(), "CFrame".to_string());
        m.insert("color3".to_string(), "Color3".to_string());

        // Entity types
        m.insert("anyentity".to_string(), "long".to_string());
        m.insert("entity".to_string(), "long".to_string());
        m.insert("entityid".to_string(), "long".to_string());
        m.insert("assetpath".to_string(), "string".to_string());

        // Cast system types
        m.insert(
            "castactiontarget".to_string(),
            "CastActionTarget".to_string(),
        );
        m.insert("castcontext".to_string(), "CastContext".to_string());

        m
    }

    pub fn map(&self, ts_type: &str) -> String {
        // Check case-insensitive match
        if let Some(mapped) = self.mappings.get(&ts_type.to_lowercase()) {
            return mapped.clone();
        }

        // Return original type if no mapping found
        ts_type.to_string()
    }

    pub fn map_full_type(&self, field_type: &str) -> String {
        // Handle list,T and map,K,V and set,T types
        if field_type.starts_with("list,") {
            let element = &field_type[5..];
            return format!("list,{}", self.map(element));
        }

        if field_type.starts_with("set,") {
            let element = &field_type[4..];
            return format!("set,{}", self.map(element));
        }

        if field_type.starts_with("map,") {
            let parts: Vec<&str> = field_type[4..].splitn(2, ',').collect();
            if parts.len() == 2 {
                return format!("map,{},{}", self.map(parts[0]), self.map(parts[1]));
            }
        }

        self.map(field_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_type_mapping() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map("number"), "double");
        assert_eq!(mapper.map("string"), "string");
        assert_eq!(mapper.map("boolean"), "bool");
    }

    #[test]
    fn test_custom_type_mapping() {
        let mut custom = HashMap::new();
        custom.insert("Vector3".to_string(), "Vector3".to_string());
        custom.insert("Entity".to_string(), "long".to_string());

        let mapper = TypeMapper::new(&custom);
        assert_eq!(mapper.map("Vector3"), "Vector3");
        assert_eq!(mapper.map("Entity"), "long");
    }

    #[test]
    fn test_case_insensitive() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map("Number"), "double");
        assert_eq!(mapper.map("STRING"), "string");
        assert_eq!(mapper.map("AnyEntity"), "long");
    }

    #[test]
    fn test_map_full_type() {
        let mapper = TypeMapper::new(&HashMap::new());
        assert_eq!(mapper.map_full_type("list,number"), "list,double");
        assert_eq!(
            mapper.map_full_type("map,string,number"),
            "map,string,double"
        );
    }
}
