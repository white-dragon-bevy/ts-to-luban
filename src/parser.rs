pub mod class_info;
pub mod field_info;

pub use class_info::ClassInfo;
pub use field_info::FieldInfo;

use anyhow::Result;
use std::path::Path;
use std::collections::HashMap;
use swc_common::{sync::Lrc, SourceMap, FileName, BytePos, comments::{Comments, SingleThreadedComments}};
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

        let comments = SingleThreadedComments::default();

        let mut parser = Parser::new(
            Syntax::Typescript(TsSyntax {
                tsx: path.extension().map_or(false, |ext| ext == "tsx"),
                decorators: true,
                ..Default::default()
            }),
            StringInput::from(&*fm),
            Some(&comments),
        );

        let module = parser
            .parse_module()
            .map_err(|e| anyhow::anyhow!("Parse error: {:?}", e))?;

        let mut classes = Vec::new();

        for item in &module.body {
            match item {
                ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) => {
                    let export_pos = export.span.lo;
                    if let Decl::Class(class_decl) = &export.decl {
                        if let Some(class_info) = self.extract_class(class_decl, path, &file_hash, &comments, export_pos) {
                            classes.push(class_info);
                        }
                    }
                    if let Decl::TsInterface(iface_decl) = &export.decl {
                        if let Some(iface_info) = self.extract_interface(iface_decl, path, &file_hash, &comments, export_pos) {
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

    fn get_leading_comment(&self, pos: BytePos, comments: &SingleThreadedComments) -> Option<String> {
        comments.get_leading(pos).and_then(|cs| {
            // Try JSDoc block comment first (starts with *)
            if let Some(jsdoc) = cs.iter()
                .filter(|c| c.text.starts_with('*'))
                .last()
            {
                return Some(parse_jsdoc_description(&jsdoc.text));
            }
            // Fall back to line comment (//)
            cs.iter()
                .last()
                .map(|c| c.text.trim().to_string())
                .filter(|s| !s.is_empty())
        })
    }

    fn get_param_comments(&self, pos: BytePos, comments: &SingleThreadedComments) -> HashMap<String, String> {
        let mut params = HashMap::new();
        if let Some(cs) = comments.get_leading(pos) {
            for c in cs.iter() {
                if c.text.starts_with('*') {
                    parse_jsdoc_params(&c.text, &mut params);
                }
            }
        }
        params
    }

    fn extract_class(&self, class_decl: &ClassDecl, path: &Path, file_hash: &str, comments: &SingleThreadedComments, export_pos: BytePos) -> Option<ClassInfo> {
        let name = class_decl.ident.sym.to_string();
        let mut fields = Vec::new();
        let mut implements = Vec::new();
        let mut extends = None;

        // Get first decorator position if any (comment may be attached there)
        let first_decorator_pos = class_decl.class.decorators.first().map(|d| d.span.lo);

        // Extract class comment (try multiple positions)
        let class_comment = self.get_leading_comment(export_pos, comments)
            .or_else(|| first_decorator_pos.and_then(|pos| self.get_leading_comment(pos, comments)))
            .or_else(|| self.get_leading_comment(class_decl.ident.span.lo, comments))
            .or_else(|| self.get_leading_comment(class_decl.class.span.lo, comments));
        let mut param_comments = self.get_param_comments(export_pos, comments);
        if let Some(pos) = first_decorator_pos {
            param_comments.extend(self.get_param_comments(pos, comments));
        }

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
                    // Get @param comments from constructor JSDoc
                    let ctor_param_comments = self.get_param_comments(ctor.span.lo, comments);

                    // Extract constructor parameters with modifiers
                    for param in &ctor.params {
                        if let ParamOrTsParamProp::TsParamProp(prop) = param {
                            if let Some(mut field) = self.extract_param_prop(prop) {
                                // Check @param comment (constructor JSDoc first, then class-level)
                                if let Some(comment) = ctor_param_comments.get(&field.name)
                                    .or_else(|| param_comments.get(&field.name)) {
                                    field.comment = Some(comment.clone());
                                }
                                fields.push(field);
                            }
                        }
                    }
                }
                ClassMember::ClassProp(prop) => {
                    if let Some(mut field) = self.extract_class_prop(prop, comments) {
                        // Check @param comment if no inline comment
                        if field.comment.is_none() {
                            if let Some(comment) = param_comments.get(&field.name) {
                                field.comment = Some(comment.clone());
                            }
                        }
                        fields.push(field);
                    }
                }
                _ => {}
            }
        }

        Some(ClassInfo {
            name,
            comment: class_comment,
            fields,
            implements,
            extends,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: false,
            output_path: None,
            module_name: None,
        })
    }

    fn extract_interface(&self, iface_decl: &TsInterfaceDecl, path: &Path, file_hash: &str, comments: &SingleThreadedComments, export_pos: BytePos) -> Option<ClassInfo> {
        let name = iface_decl.id.sym.to_string();
        let mut fields = Vec::new();

        // Extract interface comment (try export position first)
        let iface_comment = self.get_leading_comment(export_pos, comments)
            .or_else(|| self.get_leading_comment(iface_decl.span.lo, comments));
        let param_comments = self.get_param_comments(export_pos, comments);

        for member in &iface_decl.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                if let Some(mut field) = self.extract_interface_prop(prop, comments) {
                    // Check @param comment if no inline comment
                    if field.comment.is_none() {
                        if let Some(comment) = param_comments.get(&field.name) {
                            field.comment = Some(comment.clone());
                        }
                    }
                    fields.push(field);
                }
            }
        }

        Some(ClassInfo {
            name,
            comment: iface_comment,
            fields,
            implements: vec![],
            extends: None,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
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

    fn extract_class_prop(&self, prop: &ClassProp, comments: &SingleThreadedComments) -> Option<FieldInfo> {
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

        // Extract field comment
        let comment = self.get_leading_comment(prop.span.lo, comments);

        Some(FieldInfo {
            name,
            field_type,
            comment,
            is_optional: prop.is_optional,
        })
    }

    fn extract_interface_prop(&self, prop: &TsPropertySignature, comments: &SingleThreadedComments) -> Option<FieldInfo> {
        let name = match &*prop.key {
            Expr::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type(&ann.type_ann))
            .unwrap_or_else(|| "string".to_string());

        // Extract field comment
        let comment = self.get_leading_comment(prop.span.lo, comments);

        Some(FieldInfo {
            name,
            field_type,
            comment,
            is_optional: prop.optional,
        })
    }

    fn convert_type(&self, ts_type: &TsType) -> String {
        match ts_type {
            TsType::TsKeywordType(kw) => match kw.kind {
                TsKeywordTypeKind::TsNumberKeyword => "number".to_string(),
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

/// Parse JSDoc comment to extract the description (first line before any @tags)
fn parse_jsdoc_description(text: &str) -> String {
    let mut description = String::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('*').trim();
        if line.starts_with('@') {
            break;
        }
        if !line.is_empty() {
            if !description.is_empty() {
                description.push(' ');
            }
            description.push_str(line);
        }
    }
    description
}

/// Parse JSDoc @param tags into a map of param_name -> description
fn parse_jsdoc_params(text: &str, params: &mut HashMap<String, String>) {
    for line in text.lines() {
        let line = line.trim().trim_start_matches('*').trim();
        if line.starts_with("@param") {
            // Format: @param name description or @param {type} name description
            let rest = line.strip_prefix("@param").unwrap().trim();

            // Skip type if present: {type}
            let rest = if rest.starts_with('{') {
                if let Some(end) = rest.find('}') {
                    rest[end + 1..].trim()
                } else {
                    rest
                }
            } else {
                rest
            };

            // Extract name and description
            if let Some(space_idx) = rest.find(|c: char| c.is_whitespace()) {
                let name = rest[..space_idx].to_string();
                let desc = rest[space_idx..].trim();
                // Remove leading "- " if present
                let desc = desc.strip_prefix("- ").unwrap_or(desc).to_string();
                if !name.is_empty() && !desc.is_empty() {
                    params.insert(name, desc);
                }
            }
        }
    }
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
        assert_eq!(classes[0].fields[1].field_type, "number");
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
        assert_eq!(classes[0].fields[1].field_type, "list,number");
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

        assert_eq!(classes[0].fields[0].field_type, "map,string,number");
        assert_eq!(classes[0].fields[1].field_type, "map,string,bool");
    }
}
