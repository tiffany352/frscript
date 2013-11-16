use parse::*;
use context::*;
use ast::*;
use std::vec;

pub struct EvalError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for EvalError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + ": " + self.msg.clone()
    }
}

/*fn call(ctx: &mut Context, f: ~extern fn(&mut Context,~[FRValue]) -> Result<FRValue, ~str>, args: ~[AST], line: LineInfo) -> Result<FRValue, EvalError> {
    let mut res = ~[];
    for arg in args.iter() {
        match eval(ctx, arg.clone(), args) {
            Ok(x) => res.push(x),
            Err(e) => return Err(e)
        }
    }
    match (*f)(ctx, res) {
        Ok(v) => Ok(v),
        Err(s) => Err(EvalError {msg: s, line: line})
    }
}*/

pub fn eval(ctx: &mut Context, tok: AST, stack: ~[FRValue]) -> Result<~[FRValue], EvalError> {
    let mut stack = stack;
    match tok.node.clone() {
        Expr(arr) => {
            for ast in arr.iter() {
                match ast.node {
                    Var(ref name) => match ctx.lookup(name.clone()) {
                        Some((Function(f, nargs), _)) => match (*f)(ctx, stack.tailn(stack.len() - nargs).to_owned()) {
                            Ok(v) => {
                                let len = stack.len();
                                stack.truncate(len - nargs);
                                stack.push_all_move(v)
                            } 
                            Err(e) => return Err(EvalError {msg: e, line: tok.line})
                        },
                        Some((val, _)) => stack.push(val),
                        None => return Err(EvalError {msg: ~"ICE: Type checker didn't catch non-existent value", line: tok.line})
                    },
                    Literal(ref l) => stack.push(l.clone()),
                    Expr(_) => return Err(EvalError {msg: ~"NYI", line: tok.line})
                }
            }
            Ok(stack)
        }
        /*match ctx.lookup(l.clone()) {
            Some((v,_)) => match v {
                Function(f) => call(ctx, f, args, tok.line),
                _ => Err(EvalError {msg: ~"WTF: Function expected, got something else (this should have been caught by the type checker", line: tok.line})
            },
            None => Err(EvalError {msg: ~"WTF: Function expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },*/
        Var(name)   => match ctx.lookup(name.clone()) {
            Some((v,_)) => Ok(vec::append_one(stack, v)),
            None => Err(EvalError {msg: ~"WTF: Atom expected, got nothing (this should have been caught by the type checker", line: tok.line})
        },
        Literal(ref v) => Ok(vec::append_one(stack, v.clone())),
    }
}

