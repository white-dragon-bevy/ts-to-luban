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
}

pub fn parse_decorator(decorator: &Decorator) -> Option<ParsedDecorator> {
    match &*decorator.expr {
        Expr::Call(call) => {
            let name = match &call.callee {
                Callee::Expr(expr) => match &**expr {
                    Expr::Ident(ident) => ident.sym.to_string(),
                    _ => return None,
                },
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

            Some(ParsedDecorator { name, args, named_args })
        }
        Expr::Ident(ident) => {
            Some(ParsedDecorator {
                name: ident.sym.to_string(),
                args: Vec::new(),
                named_args: std::collections::HashMap::new(),
            })
        }
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
