use context::*;
use parse::*;
use ast::*;

fn add(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    let mut sum = 0f;
    for v in args.iter() {
        match *v {
            Number(n) => sum += n,
            _ => return Err(fmt!("WTF: Expected number, got %?, this should have been caught by the type checker", v))
        }
    }
    Ok(Number(sum))
}

fn list(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(List(args))
}

fn typeof(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(String(args.head().FRtype_of().to_str()))
}

fn typeeq(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(String(fmt!("%b", args[0].FRtype_of() == args[1].FRtype_of())))
}

fn test_macro(args: ~[AST]) -> AST {
    AST {
        node: Literal(String(~"hi")),
        line: LineInfo {line: 0, startslice: 0, endslice: 0, startcol: 0, endcol: 0},
        typeinfo: @StringT
    }
}

fn register_stdlib(ctx: &mut Context) {
    ctx.global.define(~"+",      Function(~add),    @Func(~[Float, Float, Float]));
    ctx.global.define(~"list",   Function(~list),   @Func(~[Any, ListT]));
    ctx.global.define(~"typeof", Function(~typeof), @Func(~[Any, StringT]));
    ctx.global.define(~"typeeq", Function(~typeeq), @Func(~[Any, Any, StringT]));
    ctx.global.macro(~"test_macro", ~test_macro);
}

