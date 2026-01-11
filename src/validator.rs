use crate::parser::field_info::{FieldValidators, SizeConstraint};
use crate::table_registry::TableRegistry;

pub struct ValidatorGenerator<'a> {
    registry: &'a TableRegistry,
}

impl<'a> ValidatorGenerator<'a> {
    pub fn new(registry: &'a TableRegistry) -> Self {
        Self { registry }
    }

    /// Generate Luban type string with validators
    /// e.g., "double#range=[1,100]" or "int!#ref=item.TbItem"
    pub fn generate_type(&self, base_type: &str, validators: &FieldValidators) -> String {
        let mut result = base_type.to_string();

        // Handle required (!)
        if validators.required {
            result.push('!');
        }

        // Handle ref
        if let Some(ref_target) = &validators.ref_target {
            if let Some(full_name) = self.registry.resolve_ref(ref_target) {
                result.push_str(&format!("#ref={}", full_name));
            } else {
                eprintln!("Warning: Could not resolve ref target: {}", ref_target);
            }
        }

        // Handle range
        if let Some((min, max)) = &validators.range {
            result.push_str(&format!("#range=[{},{}]", min, max));
        }

        // Handle set
        if !validators.set_values.is_empty() {
            let set_str = validators.set_values.join(",");
            result.push_str(&format!("#set={}", set_str));
        }

        result
    }

    /// Generate container type with size/index validators
    /// e.g., "(list#size=4),Foo" or "(list#index=id),Foo"
    pub fn generate_container_type(
        &self,
        container: &str,
        element_type: &str,
        validators: &FieldValidators,
    ) -> String {
        let mut container_mods = Vec::new();

        if let Some(size) = &validators.size {
            match size {
                SizeConstraint::Exact(n) => container_mods.push(format!("size={}", n)),
                SizeConstraint::Range(min, max) => {
                    container_mods.push(format!("size=[{},{}]", min, max))
                }
            }
        }

        if let Some(index) = &validators.index_field {
            container_mods.push(format!("index={}", index));
        }

        if container_mods.is_empty() {
            format!("{},{}", container, element_type)
        } else {
            format!(
                "({}#{}),{}",
                container,
                container_mods.join(","),
                element_type
            )
        }
    }
}
