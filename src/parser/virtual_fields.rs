use crate::config::{FieldValidators, SizeConstraint, VirtualField, VirtualFieldsConfig};
use crate::parser::class_info::ClassInfo;
use crate::parser::field_info::FieldInfo;
use std::collections::HashMap;

/// Inject virtual fields from config into class infos
pub fn inject_virtual_fields(
    class_infos: &mut HashMap<String, ClassInfo>,
    virtual_fields: &[VirtualFieldsConfig],
) -> anyhow::Result<()> {
    // Group fields by class name
    let mut fields_by_class: HashMap<String, Vec<&VirtualField>> = HashMap::new();

    for config in virtual_fields {
        for field in &config.fields {
            fields_by_class
                .entry(config.class.clone())
                .or_insert_with(Vec::new)
                .push(field);
        }
    }

    // Inject fields into classes
    for (class_name, fields) in fields_by_class {
        if let Some(class_info) = class_infos.get_mut(&class_name) {
            for field_config in fields {
                let field_info = convert_virtual_field(field_config);
                class_info.fields.push(field_info);
            }
        }
        // Note: We don't error if class not found - user will fix it later based on Luban's error
    }

    Ok(())
}

/// Convert VirtualField config to FieldInfo
fn convert_virtual_field(field_config: &VirtualField) -> FieldInfo {
    let validators = convert_validators(&field_config.validators);
    let tags = generate_relocate_tags(&field_config.relocate_to);

    FieldInfo {
        name: field_config.name.clone(),
        field_type: field_config.field_type.clone(),
        comment: field_config.comment.clone(),
        is_optional: field_config.optional.unwrap_or(false),
        validators,
        is_object_factory: false,
        factory_inner_type: None,
        is_constructor: false,
        constructor_inner_type: None,
        original_type: field_config.field_type.clone(),
        // Store relocate tags for XML generation
        relocate_tags: tags,
    }
}

/// Convert config validators to parser validators
fn convert_validators(
    validators: &Option<FieldValidators>,
) -> crate::parser::field_info::FieldValidators {
    match validators {
        Some(v) => crate::parser::field_info::FieldValidators {
            ref_target: v.ref_target.clone(),
            range: v.range,
            required: v.required,
            size: v.size.as_ref().map(|s| match s {
                SizeConstraint::Exact(n) => crate::parser::field_info::SizeConstraint::Exact(*n),
                SizeConstraint::Range(a, b) => {
                    crate::parser::field_info::SizeConstraint::Range(*a, *b)
                }
            }),
            set_values: v.set_values.clone(),
            index_field: v.index_field.clone(),
            nominal: v.nominal,
        },
        None => crate::parser::field_info::FieldValidators::default(),
    }
}

/// Generate relocateTo tags string
fn generate_relocate_tags(relocate_to: &Option<crate::config::RelocateToConfig>) -> Option<String> {
    if let Some(config) = relocate_to {
        let tags = format!("relocateTo={},prefix={}", config.target, config.prefix);

        if let Some(target_bean) = &config.target_bean {
            Some(format!("{},targetBean={}", tags, target_bean))
        } else {
            Some(tags)
        }
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_relocate_tags() {
        let relocate_to = Some(crate::config::RelocateToConfig {
            target: "TScalingStat".to_string(),
            prefix: "_main_stat".to_string(),
            target_bean: None,
        });

        let tags = generate_relocate_tags(&relocate_to).unwrap();
        assert_eq!(tags, "relocateTo=TScalingStat,prefix=_main_stat");
    }

    #[test]
    fn test_generate_relocate_tags_with_target_bean() {
        let relocate_to = Some(crate::config::RelocateToConfig {
            target: "TScalingStat".to_string(),
            prefix: "_main_stat".to_string(),
            target_bean: Some("ScalingStat".to_string()),
        });

        let tags = generate_relocate_tags(&relocate_to).unwrap();
        assert_eq!(
            tags,
            "relocateTo=TScalingStat,prefix=_main_stat,targetBean=ScalingStat"
        );
    }

    #[test]
    fn test_generate_relocate_tags_none() {
        let tags = generate_relocate_tags(&None);
        assert!(tags.is_none());
    }

    #[test]
    fn test_convert_validators() {
        let config_validators = Some(FieldValidators {
            ref_target: Some("Item".to_string()),
            range: Some((0.0, 100.0)),
            required: true,
            size: Some(SizeConstraint::Exact(5)),
            set_values: vec!["a".to_string(), "b".to_string()],
            index_field: Some("id".to_string()),
            nominal: true,
        });

        let parser_validators = convert_validators(&config_validators);
        assert_eq!(parser_validators.ref_target, Some("Item".to_string()));
        assert_eq!(parser_validators.range, Some((0.0, 100.0)));
        assert!(parser_validators.required);
        assert!(matches!(
            parser_validators.size,
            Some(crate::parser::field_info::SizeConstraint::Exact(5))
        ));
        assert_eq!(
            parser_validators.set_values,
            vec!["a".to_string(), "b".to_string()]
        );
        assert_eq!(parser_validators.index_field, Some("id".to_string()));
        assert!(parser_validators.nominal);
    }

    #[test]
    fn test_inject_virtual_fields() {
        let mut class_infos = HashMap::new();
        class_infos.insert(
            "WeaponConfig".to_string(),
            ClassInfo {
                name: "WeaponConfig".to_string(),
                comment: None,
                alias: None,
                fields: vec![],
                implements: vec![],
                extends: None,
                source_file: "test.ts".to_string(),
                file_hash: "abc123".to_string(),
                is_interface: false,
                output_path: None,
                module_name: None,
                type_params: HashMap::new(),
                luban_table: None,
            },
        );

        let virtual_fields = vec![VirtualFieldsConfig {
            class: "WeaponConfig".to_string(),
            fields: vec![VirtualField {
                name: "mainStat".to_string(),
                field_type: "ScalingStat".to_string(),
                comment: Some("Main stat".to_string()),
                optional: None,
                relocate_to: Some(crate::config::RelocateToConfig {
                    target: "TScalingStat".to_string(),
                    prefix: "_main".to_string(),
                    target_bean: None,
                }),
                validators: None,
            }],
        }];

        inject_virtual_fields(&mut class_infos, &virtual_fields).unwrap();

        let weapon_config = class_infos.get("WeaponConfig").unwrap();
        assert_eq!(weapon_config.fields.len(), 1);
        assert_eq!(weapon_config.fields[0].name, "mainStat");
        assert_eq!(weapon_config.fields[0].field_type, "ScalingStat");
        assert_eq!(
            weapon_config.fields[0].comment,
            Some("Main stat".to_string())
        );
        assert_eq!(
            weapon_config.fields[0].relocate_tags,
            Some("relocateTo=TScalingStat,prefix=_main".to_string())
        );
    }

    #[test]
    fn test_inject_virtual_fields_multiple_blocks() {
        let mut class_infos = HashMap::new();
        class_infos.insert(
            "WeaponConfig".to_string(),
            ClassInfo {
                name: "WeaponConfig".to_string(),
                comment: None,
                alias: None,
                fields: vec![],
                implements: vec![],
                extends: None,
                source_file: "test.ts".to_string(),
                file_hash: "abc123".to_string(),
                is_interface: false,
                output_path: None,
                module_name: None,
                type_params: HashMap::new(),
                luban_table: None,
            },
        );

        let virtual_fields = vec![
            VirtualFieldsConfig {
                class: "WeaponConfig".to_string(),
                fields: vec![VirtualField {
                    name: "mainStat".to_string(),
                    field_type: "ScalingStat".to_string(),
                    comment: None,
                    optional: None,
                    relocate_to: Some(crate::config::RelocateToConfig {
                        target: "TScalingStat".to_string(),
                        prefix: "_main".to_string(),
                        target_bean: None,
                    }),
                    validators: None,
                }],
            },
            VirtualFieldsConfig {
                class: "WeaponConfig".to_string(),
                fields: vec![VirtualField {
                    name: "subStat".to_string(),
                    field_type: "ScalingStat".to_string(),
                    comment: None,
                    optional: None,
                    relocate_to: Some(crate::config::RelocateToConfig {
                        target: "TScalingStat".to_string(),
                        prefix: "_sub".to_string(),
                        target_bean: None,
                    }),
                    validators: None,
                }],
            },
        ];

        inject_virtual_fields(&mut class_infos, &virtual_fields).unwrap();

        let weapon_config = class_infos.get("WeaponConfig").unwrap();
        assert_eq!(weapon_config.fields.len(), 2);
        assert_eq!(weapon_config.fields[0].name, "mainStat");
        assert_eq!(weapon_config.fields[1].name, "subStat");
    }

    #[test]
    fn test_inject_virtual_fields_nonexistent_class() {
        let mut class_infos = HashMap::new();

        let virtual_fields = vec![VirtualFieldsConfig {
            class: "NonExistentClass".to_string(),
            fields: vec![VirtualField {
                name: "someField".to_string(),
                field_type: "string".to_string(),
                comment: None,
                optional: None,
                relocate_to: None,
                validators: None,
            }],
        }];

        // Should not error - user will fix based on Luban's error
        let result = inject_virtual_fields(&mut class_infos, &virtual_fields);
        assert!(result.is_ok());
        assert!(class_infos.is_empty());
    }
}
