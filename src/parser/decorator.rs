use swc_ecma_ast::*;

#[derive(Debug, Clone)]
pub enum DecoratorArg {
    Number(f64),
    String(String),
    Identifier(String),
    Array(Vec<DecoratorArg>),
}

#[derive(Debug, Clone)]
pub struct ParsedDecorator {
    pub name: String,
    pub args: Vec<DecoratorArg>,
    pub named_args: std::collections::HashMap<String, DecoratorArg>,
    /// Type parameters for generic decorators like @RefReplace<T, "field">()
    pub type_params: Vec<String>,
}

pub fn parse_decorator(decorator: &Decorator) -> Option<ParsedDecorator> {
    match &*decorator.expr {
        Expr::Call(call) => {
            let (name, type_params) = match &call.callee {
                Callee::Expr(expr) => {
                    match &**expr {
                        Expr::Ident(ident) => {
                            // Check if call has type arguments (e.g., RefReplace<Item, "field">())
                            let type_params = if let Some(type_args) = &call.type_args {
                                extract_type_params(type_args)
                            } else {
                                Vec::new()
                            };
                            (ident.sym.to_string(), type_params)
                        }
                        // Handle generic call: RefReplace<T, "field">() (alternative AST structure)
                        Expr::TsInstantiation(inst) => {
                            let name = match &*inst.expr {
                                Expr::Ident(ident) => ident.sym.to_string(),
                                _ => return None,
                            };
                            let type_params = extract_type_params(&inst.type_args);
                            (name, type_params)
                        }
                        _ => return None,
                    }
                }
                _ => return None,
            };

            let mut args = Vec::new();
            let mut named_args = std::collections::HashMap::new();

            for arg in &call.args {
                match &*arg.expr {
                    Expr::Lit(Lit::Num(n)) => {
                        args.push(DecoratorArg::Number(n.value));
                    }
                    Expr::Lit(Lit::Str(s)) => {
                        // Use format!("{:?}", ...) to convert Wtf8Atom to string
                        let str_val = format!("{:?}", s.value).trim_matches('"').to_string();
                        args.push(DecoratorArg::String(str_val));
                    }
                    Expr::Ident(ident) => {
                        args.push(DecoratorArg::Identifier(ident.sym.to_string()));
                    }
                    Expr::Object(obj) => {
                        for prop in &obj.props {
                            if let PropOrSpread::Prop(prop) = prop {
                                if let Prop::KeyValue(kv) = &**prop {
                                    let key = match &kv.key {
                                        PropName::Ident(i) => i.sym.to_string(),
                                        PropName::Str(s) => {
                                            format!("{:?}", s.value).trim_matches('"').to_string()
                                        }
                                        _ => continue,
                                    };
                                    let value = parse_expr_to_arg(&kv.value);
                                    if let Some(v) = value {
                                        named_args.insert(key, v);
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            Some(ParsedDecorator {
                name,
                args,
                named_args,
                type_params,
            })
        }
        Expr::Ident(ident) => Some(ParsedDecorator {
            name: ident.sym.to_string(),
            args: Vec::new(),
            named_args: std::collections::HashMap::new(),
            type_params: Vec::new(),
        }),
        _ => None,
    }
}

fn parse_expr_to_arg(expr: &Expr) -> Option<DecoratorArg> {
    match expr {
        Expr::Lit(Lit::Num(n)) => Some(DecoratorArg::Number(n.value)),
        Expr::Lit(Lit::Str(s)) => {
            let str_val = format!("{:?}", s.value).trim_matches('"').to_string();
            Some(DecoratorArg::String(str_val))
        }
        Expr::Ident(ident) => Some(DecoratorArg::Identifier(ident.sym.to_string())),
        _ => None,
    }
}

/// Extract type parameters from TsTypeParamInstantiation
/// Handles both type references (T) and literal types ("field")
fn extract_type_params(type_args: &TsTypeParamInstantiation) -> Vec<String> {
    type_args
        .params
        .iter()
        .filter_map(|param| match &**param {
            // Type reference: T, Item, etc.
            TsType::TsTypeRef(type_ref) => {
                if let TsEntityName::Ident(ident) = &type_ref.type_name {
                    Some(ident.sym.to_string())
                } else {
                    None
                }
            }
            // Literal type: "fieldName"
            TsType::TsLitType(lit_type) => {
                if let TsLit::Str(s) = &lit_type.lit {
                    Some(format!("{:?}", s.value).trim_matches('"').to_string())
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect()
}
