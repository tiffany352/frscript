use ast::*;
use context::*;
use parse::*;

struct MacroError {
    line: LineInfo
}

impl ToStr for MacroError {
    fn to_str(&self) -> ~str {
        ~"Macro Error"
    }
}

fn expand_macros(ctx: &mut Context, ast: AST) -> Result<AST, MacroError> {
    match ast.node.clone() {
        Expr(atom, args) => match ctx.lookup_macro(atom.clone()) {
            Some(f) => expand_macros(ctx, (*f)(args)),
            None => {
                let mut res = ~[];
                for a in args.iter() {
                    match expand_macros(ctx, a.clone()) {
                        Ok(v) => res.push(v),
                        Err(e) => return Err(e)
                    }
                }
                Ok(AST {node: Expr(atom.clone(), res), line: ast.line, typeinfo: ast.typeinfo})
            },
        },
        _ => Ok(ast)
    }
}

