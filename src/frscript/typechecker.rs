use parse::*;
use context;
use ast::*;

pub struct TypeError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for TypeError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + " " + self.msg
    }
}

pub fn get_type(scope: &context::Scope, name: ~str) -> Result<@FRType, ~str> {
    match scope.lookup(name.clone()) {
        Some((_, t)) => Ok(t),
        None => Err(format!("No such variable {:s}", name)),
    }
}

pub fn typecheck(scope: &mut context::Scope, token: AST) -> Result<AST, TypeError> {
    let build_var = |name, T| Ok(AST {node: Var(name), line: token.line, typeinfo: T});
    let build_expr: &fn(~str, ~[AST], @FRType) -> Result<AST,TypeError> = |atom, args, T| Ok(AST {node: Expr(atom, args), line: token.line, typeinfo: T});
    match token.node.clone() {
        Expr(atom, args) => match get_type(scope, atom.clone()) {
            Ok(t) => {
                match t.clone() {
                    @Func(ref sig) => {
                        let mut res = ~[];
                        if sig.len() - 1 != args.len() {
                            return Err(TypeError {msg: format!("Function takes {:u} parameters, {:u} provided", sig.len()-1, args.len()), line: token.line})
                        }
                        for e in args.iter() {
                            match typecheck(scope, e.clone()) {
                                Ok(v) => res.push(v),
                                Err(e) => return Err(e)
                            }
                        }
                        for (T, v) in sig.iter().zip(res.iter()) {
                            let T2 = match (v.node.clone(), v.typeinfo.clone()) {
                                (Expr(_, _), @Func(ref a)) => @a.last().clone(),
                                (_, x) => x
                            };
                            if T != T2 {
                                return Err(TypeError {msg: format!("Mismatched types: Expected {:s}, got {:s}", T.to_str(), v.typeinfo.to_str()), line: v.line})
                            }
                        }
                        build_expr(atom, res, t)
                    }
                    _ => Err(TypeError {msg: format!("Expected function, got {:s}", t.to_str()), line: token.line})
                }
            },
            Err(s) => Err(TypeError {msg: s, line: token.line})
        },
        Var(s) => match get_type(scope, s.clone()) {
            Ok(t) => build_var(s, t),
            Err(s) => Err(TypeError {msg: s, line: token.line})
        },
        Literal(_) => Ok(token),
    }
}

