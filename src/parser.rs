pub mod class_info;
pub mod field_info;

pub use class_info::ClassInfo;
pub use field_info::FieldInfo;

use anyhow::Result;
use std::path::Path;
use swc_common::{sync::Lrc, SourceMap, FileName};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsSyntax};
use swc_ecma_ast::*;

pub struct TsParser {
    source_map: Lrc<SourceMap>,
}

impl TsParser {
    pub fn new() -> Self {
        Self {
            source_map: Default::default(),
        }
    }

    pub fn parse_file(&self, path: &Path) -> Result<Vec<ClassInfo>> {
        let content = std::fs::read_to_string(path)?;
        let file_hash = compute_hash(&content);

        let fm = self.source_map.new_source_file(
            FileName::Real(path.to_path_buf()).into(),
            content,
        );

        let mut parser = Parser::new(
            Syntax::Typescript(TsSyntax {
                tsx: path.extension().map_or(false, |ext| ext == "tsx"),
                decorators: true,
                ..Default::default()
            }),
            StringInput::from(&*fm),
            None,
        );

        let module = parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

        let mut classes = Vec::new();

        for item in &module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
                    if let Decl::Class(class_decl) = &export.decl {
                        if let Some(class_info) = self.extract_class(class_decl, path, &file_hash) {
                            classes.push(class_info);
                        }
                    }
                    if let Decl::TsInterface(iface_decl) = &export.decl {
                        if let Some(iface_info) = self.extract_interface(iface_decl, path, &file_hash) {
                            classes.push(iface_info);
                        }
                    }
                }
                ModuleItem::Stmt(Stmt::Decl(Decl::Class(_class_decl))) => {
                    // Non-exported class - skip
                }
                _ => {}
            }
        }

        Ok(classes)
    }

    fn extract_class(&self, class_decl: &ClassDecl, path: &Path, file_hash: &str) -> Option<ClassInfo> {
        let name = class_decl.ident.sym.to_string();
        let mut fields = Vec::new();
        let mut implements = Vec::new();
        let mut extends = None;

        // Extract implements
        for clause in &class_decl.class.implements {
            if let Expr::Ident(ident) = &*clause.expr {
                implements.push(ident.sym.to_string());
            }
        }

        // Extract extends
        if let Some(super_class) = &class_decl.class.super_class {
            if let Expr::Ident(ident) = &**super_class {
                extends = Some(ident.sym.to_string());
            }
        }

        // Extract fields from class body
        for member in &class_decl.class.body {
            match member {
                ClassMember::Constructor(ctor) => {
                    // Extract constructor parameters with modifiers
                    for param in &ctor.params {
                        if let ParamOrTsParamProp::TsParamProp(prop) = param {
                            if let Some(field) = self.extract_param_prop(prop) {
                                fields.push(field);
                            }
                        }
                    }
                }
                ClassMember::ClassProp(prop) => {
                    if let Some(field) = self.extract_class_prop(prop) {
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }

        Some(ClassInfo {
            name,
            comment: None, // TODO: Extract JSDoc
            fields,
            implements,
            extends,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: false,
        })
    }

    fn extract_interface(&self, iface_decl: &TsInterfaceDecl, path: &Path, file_hash: &str) -> Option<ClassInfo> {
        let name = iface_decl.id.sym.to_string();
        let mut fields = Vec::new();

        for member in &iface_decl.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                if let Some(field) = self.extract_interface_prop(prop) {
                    fields.push(field);
                }
            }
        }

        Some(ClassInfo {
            name,
            comment: None,
            fields,
            implements: vec![],
            extends: None,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: true,
        })
    }

    fn extract_param_prop(&self, prop: &TsParamProp) -> Option<FieldInfo> {
        let (name, type_ann, is_optional) = match &prop.param {
            TsParamPropParam::Ident(ident) => {
                (ident.id.sym.to_string(), ident.type_ann.as_ref(), ident.id.optional)
            }
            TsParamPropParam::Assign(_) => return None,
        };

        // Skip internal marker fields
        if name.contains("_nominal_") || name == "_is_trigger_combinator" || name == "_trigger_type" {
            return None;
        }

        let field_type = type_ann
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional,
        })
    }

    fn extract_class_prop(&self, prop: &ClassProp) -> Option<FieldInfo> {
        // Skip private/protected
        if prop.accessibility == Some(Accessibility::Private)
            || prop.accessibility == Some(Accessibility::Protected) {
            return None;
        }

        let name = match &prop.key {
            PropName::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        // Skip internal marker fields
        if name.contains("_nominal_") || name == "_is_trigger_combinator" || name == "_trigger_type" {
            return None;
        }

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional: prop.is_optional,
        })
    }

    fn extract_interface_prop(&self, prop: &TsPropertySignature) -> Option<FieldInfo> {
        let name = match &*prop.key {
            Expr::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional: prop.optional,
        })
    }

    fn convert_type(&self, ts_type: &TsType) -> String {
        match ts_type {
            TsType::TsKeywordType(kw) => match kw.kind {
                TsKeywordTypeKind::TsNumberKeyword => "int".to_string(),
                TsKeywordTypeKind::TsStringKeyword => "string".to_string(),
                TsKeywordTypeKind::TsBooleanKeyword => "bool".to_string(),
                _ => "string".to_string(),
            },
            TsType::TsArrayType(arr) => {
                let element_type = self.convert_type(&arr.elem_type);
                format!("list,{}", element_type)
            }
            TsType::TsTypeRef(type_ref) => {
                let type_name = match &type_ref.type_name {
                    TsEntityName::Ident(ident) => ident.sym.to_string(),
                    TsEntityName::TsQualifiedName(_) => return "string".to_string(),
                };

                match type_name.as_str() {
                    "Array" | "ReadonlyArray" => {
                        if let Some(params) = &type_ref.type_params {
                            if let Some(first) = params.params.first() {
                                let element_type = self.convert_type(first);
                                return format!("list,{}", element_type);
                            }
                        }
                        "list,string".to_string()
                    }
                    "Map" | "Record" => {
                        if let Some(params) = &type_ref.type_params {
                            if params.params.len() >= 2 {
                                let key_type = self.convert_type(&params.params[0]);
                                let value_type = self.convert_type(&params.params[1]);
                                return format!("map,{},{}", key_type, value_type);
                            }
                        }
                        "map,string,string".to_string()
                    }
                    _ => type_name,
                }
            }
            TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(union)) => {
                // Take first non-undefined/null type
                for member in &union.types {
                    match &**member {
                        TsType::TsKeywordType(kw) if matches!(
                            kw.kind,
                            TsKeywordTypeKind::TsUndefinedKeyword | TsKeywordTypeKind::TsNullKeyword
                        ) => continue,
                        _ => return self.convert_type(member),
                    }
                }
                "string".to_string()
            }
            _ => "string".to_string(),
        }
    }
}

impl Default for TsParser {
    fn default() -> Self {
        Self::new()
    }
}

fn compute_hash(content: &str) -> String {
    use md5::{Md5, Digest};
    let mut hasher = Md5::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_simple_class() {
        let ts_code = r#"
export class MyClass {
    public name: string;
    public count: number;
    public active?: boolean;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "MyClass");
        assert_eq!(classes[0].fields.len(), 3);
        assert_eq!(classes[0].fields[0].name, "name");
        assert_eq!(classes[0].fields[0].field_type, "string");
        assert_eq!(classes[0].fields[1].field_type, "int");
        assert!(classes[0].fields[2].is_optional);
    }

    #[test]
    fn test_parse_class_with_implements() {
        let ts_code = r#"
interface EntityTrigger {}

export class MyTrigger implements EntityTrigger {
    public damage: number;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].implements, vec!["EntityTrigger"]);
    }

    #[test]
    fn test_parse_array_types() {
        let ts_code = r#"
export class MyClass {
    public items: string[];
    public numbers: Array<number>;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes[0].fields[0].field_type, "list,string");
        assert_eq!(classes[0].fields[1].field_type, "list,int");
    }

    #[test]
    fn test_parse_map_types() {
        let ts_code = r#"
export class MyClass {
    public data: Map<string, number>;
    public record: Record<string, boolean>;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes[0].fields[0].field_type, "map,string,int");
        assert_eq!(classes[0].fields[1].field_type, "map,string,bool");
    }
}
