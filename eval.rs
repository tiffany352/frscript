use grammar;
use grammar::*;
use parse::*;
use context::*;
use context;

fn call(ctx: &mut Context, f: ~extern fn(&mut Context,~[FRValue]) -> Result<FRValue, ~str>, args: ~[Token<FRToken>], line: LineInfo) -> Result<FRValue, EvalError> {
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

fn eval(ctx: &mut Context, tok: Token<FRToken>) -> Result<FRValue, EvalError> {
    match tok.value.clone() {
        grammar::Label(x)   => match ctx.lookup(x.clone()) {
            Some(v) => Ok(v.clone()),
            None => Err(EvalError {msg: fmt!("No such atom %s", x), line: tok.line})
        },
        grammar::String(x)  => Ok(context::String(x.clone())),
        grammar::Number(x)  => Ok(context::Number(x.clone())),
        grammar::SExpr(arr) => {
            match arr[0].value.clone() {
                grammar::Label(x) => match ctx.lookup(x.clone()) {
                    Some(v) => match v {
                        context::Function(f) => call(ctx, f, arr.tail().to_owned(), tok.line),
                        _ => Err(EvalError {msg: fmt!("Expected function, got %?", v), line: arr[0].line})
                    },
                    None => Err(EvalError {msg: fmt!("No such atom %s", x), line: arr[0].line})
                },
                _ => Err(EvalError {msg: fmt!("Expected atom, got %?", arr[0].value), line: arr[0].line})
            }
        }
        _ => Err(EvalError {msg: fmt!("Unwanted value in AST: %?", tok.value), line: tok.line})
    }
}

