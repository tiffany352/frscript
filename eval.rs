use parse::*;
use context::*;
use ast::*;

pub struct EvalError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for EvalError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + ": " + self.msg.clone()
    }
}

fn call(ctx: &mut Context, f: ~extern fn(&mut Context,~[FRValue]) -> Result<FRValue, ~str>, args: ~[AST], line: LineInfo) -> Result<FRValue, EvalError> {
    let mut res = ~[];
    for arg in args.iter() {
        match eval(ctx, arg.clone()) {
            Ok(x) => res.push(x),
            Err(e) => return Err(e)
        }
    }
    match (*f)(ctx, res) {
        Ok(v) => Ok(v),
        Err(s) => Err(EvalError {msg: s, line: line})
    }
}

fn eval(ctx: &mut Context, tok: AST) -> Result<FRValue, EvalError> {
    match tok.node.clone() {
        Expr(l, args) => match ctx.lookup(l.clone()) {
            Some((v,_)) => match v {
                Function(f) => call(ctx, f, args, tok.line),
                _ => Err(EvalError {msg: ~"WTF: Function expected, got something else (this should have been caught by the type checker", line: tok.line})
            },
            None => Err(EvalError {msg: ~"WTF: Function expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },
        Var(name)   => match ctx.lookup(name.clone()) {
            Some((v,_)) => Ok(v.clone()),
            None => Err(EvalError {msg: ~"WTF: Atom expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },
        Literal(v) => Ok(v.clone()),
    }
}

