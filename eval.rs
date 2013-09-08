use grammar::*;
use parse::*;
use context::*;
use context;
use types::*;

fn call(ctx: &mut Context, f: ~extern fn(&mut Context,~[FRValue]) -> Result<FRValue, ~str>, args: ~[Token<FRAst>], line: LineInfo) -> Result<FRValue, EvalError> {
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

fn eval(ctx: &mut Context, tok: Token<FRAst>) -> Result<FRValue, EvalError> {
    match tok.value.clone() {
        Expr(l, _, args) => match ctx.lookup(l.clone()) {
            Some(v) => match v {
                Function(_, f) => call(ctx, f, args, tok.line),
                _ => Err(EvalError {msg: ~"WTF: Function expected, got something else (this should have been caught by the type checker", line: tok.line})
            },
            None => Err(EvalError {msg: ~"WTF: Function expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },
        Var(name, _)   => match ctx.lookup(name.clone()) {
            Some(v) => Ok(v.clone()),
            None => Err(EvalError {msg: ~"WTF: Atom expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },
        StringL(s)  => Ok(context::String(s.clone())),
        FloatL(v)  => Ok(context::Number(v.clone())),
        IntegerL(v) => Ok(context::Number(v as float))
    }
}

