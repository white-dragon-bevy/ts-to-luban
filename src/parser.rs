pub mod class_info;
pub mod enum_info;
pub mod field_info;

pub use class_info::ClassInfo;
pub use enum_info::{EnumInfo, EnumVariant};
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

    pub fn parse_enums(&self, path: &Path) -> Result<Vec<EnumInfo>> {
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

        let mut enums = Vec::new();

        for item in &module.body {
            if let ModuleItem::ModuleDecl(ModuleDecl::ExportDecl(export)) = item {
                if let Decl::TsEnum(enum_decl) = &export.decl {
                    let export_pos = export.span.lo;
                    if let Some(enum_info) = self.extract_enum(enum_decl, path, &file_hash, &comments, export_pos) {
                        enums.push(enum_info);
                    }
                }
            }
        }

        Ok(enums)
    }

    fn extract_enum(&self, enum_decl: &TsEnumDecl, path: &Path, file_hash: &str, comments: &SingleThreadedComments, export_pos: BytePos) -> Option<EnumInfo> {
        let name = enum_decl.id.sym.to_string();

        // Get enum comment (raw JSDoc text)
        let raw_enum_comment = self.get_raw_jsdoc_comment(export_pos, comments)
            .or_else(|| self.get_raw_jsdoc_comment(enum_decl.span.lo, comments));

        // Parse @flags tag from enum comment
        let is_flags = raw_enum_comment.as_ref()
            .map(|c| parse_jsdoc_tag(c, "flags").is_some())
            .unwrap_or(false);

        // Get cleaned comment (without @flags line)
        let enum_comment = raw_enum_comment.as_ref()
            .map(|c| parse_jsdoc_description_excluding_tags(c, &["flags"]));

        let mut variants = Vec::new();
        let mut is_string_enum = false;
        let mut auto_value = 1i64;
        let mut member_values: HashMap<String, i64> = HashMap::new();

        for member in &enum_decl.members {
            let member_name = match &member.id {
                TsEnumMemberId::Ident(ident) => ident.sym.to_string(),
                TsEnumMemberId::Str(s) => format!("{:?}", s.value).trim_matches('"').to_string(),
            };

            // Get raw member comment
            let raw_member_comment = self.get_raw_jsdoc_comment(member.span.lo, comments);

            // Parse @alias tag from member comment
            let alias = raw_member_comment.as_ref()
                .and_then(|c| parse_jsdoc_tag(c, "alias"))
                .unwrap_or_else(|| member_name.to_lowercase());

            // Get cleaned comment (without @alias line)
            let member_comment = raw_member_comment.as_ref()
                .map(|c| parse_jsdoc_description_excluding_tags(c, &["alias"]));

            // Determine value and whether it's a string enum
            let (value, member_is_string) = if let Some(init) = &member.init {
                match &**init {
                    Expr::Lit(Lit::Str(_)) => {
                        // String enum - use auto-incremented value
                        let v = auto_value;
                        auto_value += 1;
                        (v, true)
                    }
                    Expr::Lit(Lit::Num(n)) => {
                        // Number enum - use actual value
                        let v = n.value as i64;
                        auto_value = v + 1;
                        (v, false)
                    }
                    _ => {
                        // Binary expression or identifier reference (e.g., 1 << 0 or CAN_MOVE | CAN_ATTACK)
                        if let Some(v) = Self::eval_const_expr(init, &member_values) {
                            auto_value = v + 1;
                            (v, false)
                        } else {
                            let v = auto_value;
                            auto_value += 1;
                            (v, false)
                        }
                    }
                }
            } else {
                let v = auto_value;
                auto_value += 1;
                (v, false)
            };

            if member_is_string {
                is_string_enum = true;
            }

            // Track this member's value for later references
            member_values.insert(member_name.clone(), value);

            variants.push(EnumVariant {
                name: member_name.clone(),
                alias,
                value,
                comment: member_comment,
            });
        }

        Some(EnumInfo {
            name,
            comment: enum_comment,
            is_string_enum,
            is_flags,
            variants,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            output_path: None,
            module_name: None,
        })
    }

    /// Evaluate a constant expression (supports number literals, bit shift operators, and enum member references)
    fn eval_const_expr(expr: &Expr, member_values: &HashMap<String, i64>) -> Option<i64> {
        match expr {
            Expr::Lit(Lit::Num(n)) => Some(n.value as i64),
            Expr::Ident(ident) => {
                // Look up identifier in already-computed member values
                member_values.get(&ident.sym.to_string()).copied()
            }
            Expr::Bin(bin_expr) => {
                let left = Self::eval_const_expr(&bin_expr.left, member_values)?;
                let right = Self::eval_const_expr(&bin_expr.right, member_values)?;
                match bin_expr.op {
                    BinaryOp::LShift => Some(left << right),
                    BinaryOp::RShift => Some(left >> right),
                    BinaryOp::BitOr => Some(left | right),
                    BinaryOp::BitAnd => Some(left & right),
                    BinaryOp::BitXor => Some(left ^ right),
                    BinaryOp::Add => Some(left + right),
                    BinaryOp::Sub => Some(left - right),
                    BinaryOp::Mul => Some(left * right),
                    BinaryOp::Div if right != 0 => Some(left / right),
                    _ => None,
                }
            }
            Expr::Paren(paren) => Self::eval_const_expr(&paren.expr, member_values),
            _ => None,
        }
    }

    /// Get raw JSDoc comment text (without parsing)
    fn get_raw_jsdoc_comment(&self, pos: BytePos, comments: &SingleThreadedComments) -> Option<String> {
        comments.get_leading(pos).and_then(|cs| {
            // Try JSDoc block comment first (starts with *)
            cs.iter()
                .filter(|c| c.text.starts_with('*'))
                .last()
                .map(|c| c.text.to_string())
        })
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

    /// Extract type parameters from a type parameter declaration
    /// Returns a map of type param name -> constraint type
    /// Warns if a type param has no constraint (max 3 params supported)
    fn extract_type_params(&self, type_params: Option<&Box<TsTypeParamDecl>>, class_name: &str) -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(params) = type_params {
            if params.params.len() > 3 {
                eprintln!("  Warning: {} has more than 3 type parameters, only first 3 will be processed", class_name);
            }

            for (i, param) in params.params.iter().take(3).enumerate() {
                let param_name = param.name.sym.to_string();

                if let Some(constraint) = &param.constraint {
                    // Extract constraint type
                    let constraint_type = self.convert_type_with_params(constraint, &HashMap::new());
                    result.insert(param_name, constraint_type);
                } else {
                    eprintln!("  Warning: {}<{}> - type parameter {} at position {} has no constraint (extends), skipping",
                        class_name, param_name, param_name, i);
                }
            }
        }

        result
    }

    fn extract_class(&self, class_decl: &ClassDecl, path: &Path, file_hash: &str, comments: &SingleThreadedComments, export_pos: BytePos) -> Option<ClassInfo> {
        let name = class_decl.ident.sym.to_string();
        let mut fields = Vec::new();
        let mut implements = Vec::new();
        let mut extends = None;

        // Extract type parameters
        let type_params = self.extract_type_params(class_decl.class.type_params.as_ref(), &name);

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
                            if let Some(mut field) = self.extract_param_prop_with_type_params(prop, &type_params) {
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
                    if let Some(mut field) = self.extract_class_prop_with_type_params(prop, comments, &type_params) {
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
            type_params,
        })
    }

    fn extract_interface(&self, iface_decl: &TsInterfaceDecl, path: &Path, file_hash: &str, comments: &SingleThreadedComments, export_pos: BytePos) -> Option<ClassInfo> {
        let name = iface_decl.id.sym.to_string();
        let mut fields = Vec::new();

        // Extract type parameters
        let type_params = self.extract_type_params(iface_decl.type_params.as_ref(), &name);

        // Extract interface comment (try export position first)
        let iface_comment = self.get_leading_comment(export_pos, comments)
            .or_else(|| self.get_leading_comment(iface_decl.span.lo, comments));
        let param_comments = self.get_param_comments(export_pos, comments);

        // Extract extends (first parent interface only)
        let extends = iface_decl.extends.first().and_then(|ext| {
            if let Expr::Ident(ident) = &*ext.expr {
                Some(ident.sym.to_string())
            } else {
                None
            }
        });

        for member in &iface_decl.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                if let Some(mut field) = self.extract_interface_prop_with_type_params(prop, comments, &type_params) {
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
            extends,
            source_file: path.to_string_lossy().to_string(),
            file_hash: file_hash.to_string(),
            is_interface: true,
            output_path: None,
            module_name: None,
            type_params,
        })
    }

    #[allow(dead_code)]
    fn extract_param_prop(&self, prop: &TsParamProp) -> Option<FieldInfo> {
        self.extract_param_prop_with_type_params(prop, &HashMap::new())
    }

    fn extract_param_prop_with_type_params(&self, prop: &TsParamProp, type_params: &HashMap<String, String>) -> Option<FieldInfo> {
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
            .map(|ann| self.convert_type_with_params(&ann.type_ann, type_params))
            .unwrap_or_else(|| "string".to_string());

        Some(FieldInfo {
            name,
            field_type,
            comment: None,
            is_optional,
        })
    }

    #[allow(dead_code)]
    fn extract_class_prop(&self, prop: &ClassProp, comments: &SingleThreadedComments) -> Option<FieldInfo> {
        self.extract_class_prop_with_type_params(prop, comments, &HashMap::new())
    }

    fn extract_class_prop_with_type_params(&self, prop: &ClassProp, comments: &SingleThreadedComments, type_params: &HashMap<String, String>) -> Option<FieldInfo> {
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
            .map(|ann| self.convert_type_with_params(&ann.type_ann, type_params))
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

    #[allow(dead_code)]
    fn extract_interface_prop(&self, prop: &TsPropertySignature, comments: &SingleThreadedComments) -> Option<FieldInfo> {
        self.extract_interface_prop_with_type_params(prop, comments, &HashMap::new())
    }

    fn extract_interface_prop_with_type_params(&self, prop: &TsPropertySignature, comments: &SingleThreadedComments, type_params: &HashMap<String, String>) -> Option<FieldInfo> {
        let name = match &*prop.key {
            Expr::Ident(ident) => ident.sym.to_string(),
            _ => return None,
        };

        let field_type = prop
            .type_ann
            .as_ref()
            .map(|ann| self.convert_type_with_params(&ann.type_ann, type_params))
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

    #[allow(dead_code)]
    fn convert_type(&self, ts_type: &TsType) -> String {
        self.convert_type_with_params(ts_type, &HashMap::new())
    }

    fn convert_type_with_params(&self, ts_type: &TsType, type_params: &HashMap<String, String>) -> String {
        match ts_type {
            TsType::TsKeywordType(kw) => match kw.kind {
                TsKeywordTypeKind::TsNumberKeyword => "number".to_string(),
                TsKeywordTypeKind::TsStringKeyword => "string".to_string(),
                TsKeywordTypeKind::TsBooleanKeyword => "bool".to_string(),
                _ => "string".to_string(),
            },
            TsType::TsArrayType(arr) => {
                let element_type = self.convert_type_with_params(&arr.elem_type, type_params);
                format!("list,{}", element_type)
            }
            TsType::TsTypeRef(type_ref) => {
                let type_name = match &type_ref.type_name {
                    TsEntityName::Ident(ident) => ident.sym.to_string(),
                    TsEntityName::TsQualifiedName(_) => return "string".to_string(),
                };

                // Check if this is a type parameter that should be replaced
                if let Some(constraint_type) = type_params.get(&type_name) {
                    return constraint_type.clone();
                }

                match type_name.as_str() {
                    "Array" | "ReadonlyArray" => {
                        if let Some(params) = &type_ref.type_params {
                            if let Some(first) = params.params.first() {
                                let element_type = self.convert_type_with_params(first, type_params);
                                return format!("list,{}", element_type);
                            }
                        }
                        "list,string".to_string()
                    }
                    "Map" | "Record" => {
                        if let Some(params) = &type_ref.type_params {
                            if params.params.len() >= 2 {
                                let key_type = self.convert_type_with_params(&params.params[0], type_params);
                                let value_type = self.convert_type_with_params(&params.params[1], type_params);
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
                        _ => return self.convert_type_with_params(member, type_params),
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

/// Parse a JSDoc tag value like @flags="true" or @alias="移动"
/// Returns the value inside quotes, or None if tag not found
fn parse_jsdoc_tag(text: &str, tag_name: &str) -> Option<String> {
    let tag_prefix = format!("@{}=", tag_name);
    for line in text.lines() {
        let line = line.trim().trim_start_matches('*').trim();
        if let Some(rest) = line.strip_prefix(&tag_prefix) {
            // Extract value inside quotes
            let rest = rest.trim();
            if rest.starts_with('"') {
                if let Some(end) = rest[1..].find('"') {
                    return Some(rest[1..end+1].to_string());
                }
            }
        }
    }
    None
}

/// Parse JSDoc description excluding specific tags
/// Returns description text without lines containing the specified tags
fn parse_jsdoc_description_excluding_tags(text: &str, exclude_tags: &[&str]) -> String {
    let mut description = String::new();
    for line in text.lines() {
        let line = line.trim().trim_start_matches('*').trim();
        // Skip lines with excluded tags
        let is_excluded = exclude_tags.iter().any(|tag| {
            let tag_prefix = format!("@{}=", tag);
            line.starts_with(&tag_prefix)
        });
        if is_excluded {
            continue;
        }
        // Stop at other @tags (like @param)
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

    #[test]
    fn test_parse_interface_extends() {
        let ts_code = r#"
export interface BaseTrigger {
    id: number;
}

export interface EntityTrigger extends BaseTrigger {
    num: number;
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let classes = parser.parse_file(file.path()).unwrap();

        assert_eq!(classes.len(), 2);

        // BaseTrigger has no extends
        assert_eq!(classes[0].name, "BaseTrigger");
        assert_eq!(classes[0].extends, None);
        assert!(classes[0].is_interface);

        // EntityTrigger extends BaseTrigger
        assert_eq!(classes[1].name, "EntityTrigger");
        assert_eq!(classes[1].extends, Some("BaseTrigger".to_string()));
        assert!(classes[1].is_interface);
    }

    #[test]
    fn test_parse_string_enum() {
        let ts_code = r#"
/**
 * 物品类型枚举
 */
export enum ItemType {
    /** 角色 */
    Role = "role",
    /** 消耗品 */
    Consumable = "consumable",
    /** 货币 */
    Currency = "currency",
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let enums = parser.parse_enums(file.path()).unwrap();

        assert_eq!(enums.len(), 1);
        let e = &enums[0];
        assert_eq!(e.name, "ItemType");
        assert_eq!(e.comment, Some("物品类型枚举".to_string()));
        assert!(e.is_string_enum);

        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "Role");
        assert_eq!(e.variants[0].alias, "role");
        assert_eq!(e.variants[0].value, 1);
        assert_eq!(e.variants[0].comment, Some("角色".to_string()));

        assert_eq!(e.variants[1].name, "Consumable");
        assert_eq!(e.variants[1].alias, "consumable");
        assert_eq!(e.variants[1].value, 2);

        assert_eq!(e.variants[2].name, "Currency");
        assert_eq!(e.variants[2].alias, "currency");
        assert_eq!(e.variants[2].value, 3);
    }

    #[test]
    fn test_parse_number_enum() {
        let ts_code = r#"
/**
 * 技能类型
 */
export enum SkillStyle {
    /** 攻击技能 */
    Attack = 1,
    /** 防御技能 */
    Defense = 2,
    /** 辅助技能 */
    Support = 3,
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let enums = parser.parse_enums(file.path()).unwrap();

        assert_eq!(enums.len(), 1);
        let e = &enums[0];
        assert_eq!(e.name, "SkillStyle");
        assert_eq!(e.comment, Some("技能类型".to_string()));
        assert!(!e.is_string_enum);

        assert_eq!(e.variants.len(), 3);
        assert_eq!(e.variants[0].name, "Attack");
        assert_eq!(e.variants[0].alias, "attack");
        assert_eq!(e.variants[0].value, 1);
        assert_eq!(e.variants[0].comment, Some("攻击技能".to_string()));

        assert_eq!(e.variants[1].name, "Defense");
        assert_eq!(e.variants[1].value, 2);

        assert_eq!(e.variants[2].name, "Support");
        assert_eq!(e.variants[2].value, 3);
    }

    #[test]
    fn test_parse_enum_flags_tag() {
        let ts_code = r#"
/**
 * 位标志枚举
 * @flags="true"
 */
export enum UnitFlag {
    /** 可以移动 */
    CAN_MOVE = 1 << 0,
    /** 可以攻击 */
    CAN_ATTACK = 1 << 1,
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let enums = parser.parse_enums(file.path()).unwrap();

        assert_eq!(enums.len(), 1);
        let e = &enums[0];
        assert_eq!(e.name, "UnitFlag");
        assert!(e.is_flags, "Enum should have is_flags=true");
        // Comment should exclude the @flags line
        assert_eq!(e.comment, Some("位标志枚举".to_string()));
    }

    #[test]
    fn test_parse_enum_variant_alias_tag() {
        let ts_code = r#"
/**
 * 位标志枚举
 * @flags="true"
 */
export enum UnitFlag {
    /**
     * 可以移动
     * @alias="移动"
     */
    CAN_MOVE = 1 << 0,
    /**
     * 可以攻击
     * @alias="攻击"
     */
    CAN_ATTACK = 1 << 1,
    /** 无别名 */
    NO_ALIAS = 1 << 2,
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let enums = parser.parse_enums(file.path()).unwrap();

        assert_eq!(enums.len(), 1);
        let e = &enums[0];
        assert!(e.is_flags);

        // CAN_MOVE should use custom alias
        assert_eq!(e.variants[0].name, "CAN_MOVE");
        assert_eq!(e.variants[0].alias, "移动");
        // Comment should exclude @alias line
        assert_eq!(e.variants[0].comment, Some("可以移动".to_string()));

        // CAN_ATTACK should use custom alias
        assert_eq!(e.variants[1].name, "CAN_ATTACK");
        assert_eq!(e.variants[1].alias, "攻击");
        assert_eq!(e.variants[1].comment, Some("可以攻击".to_string()));

        // NO_ALIAS should use lowercase name
        assert_eq!(e.variants[2].name, "NO_ALIAS");
        assert_eq!(e.variants[2].alias, "no_alias");
        assert_eq!(e.variants[2].comment, Some("无别名".to_string()));
    }

    #[test]
    fn test_parse_bitshift_enum() {
        let ts_code = r#"
/**
 * 位标志枚举
 */
export enum Flags {
    /** 标志A */
    A = 1 << 0,
    /** 标志B */
    B = 1 << 1,
    /** 标志C */
    C = 1 << 2,
    /** 标志D */
    D = 1 << 4,
}
"#;
        let mut file = NamedTempFile::with_suffix(".ts").unwrap();
        file.write_all(ts_code.as_bytes()).unwrap();

        let parser = TsParser::new();
        let enums = parser.parse_enums(file.path()).unwrap();

        assert_eq!(enums.len(), 1);
        let e = &enums[0];
        assert_eq!(e.name, "Flags");
        assert!(!e.is_string_enum);

        assert_eq!(e.variants.len(), 4);
        assert_eq!(e.variants[0].name, "A");
        assert_eq!(e.variants[0].value, 1);  // 1 << 0 = 1

        assert_eq!(e.variants[1].name, "B");
        assert_eq!(e.variants[1].value, 2);  // 1 << 1 = 2

        assert_eq!(e.variants[2].name, "C");
        assert_eq!(e.variants[2].value, 4);  // 1 << 2 = 4

        assert_eq!(e.variants[3].name, "D");
        assert_eq!(e.variants[3].value, 16); // 1 << 4 = 16
    }
}
