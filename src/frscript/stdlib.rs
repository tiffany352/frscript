use context::*;
use parse::*;
use ast::*;

fn add(_: &mut Context, args: ~[FRValue]) -> Result<~[FRValue], ~str> {
    let mut sum = 0f32;
    for v in args.iter() {
        match *v {
            Number(n) => sum += n,
            _ => return Err(format!("WTF: Expected number, got {:?}, this should have been caught by the type checker", v))
        }
    }
    Ok(~[Number(sum)])
}

fn list(_: &mut Context, args: ~[FRValue]) -> Result<~[FRValue], ~str> {
    Ok(~[List(args)])
}

fn FRtypeof(_: &mut Context, args: ~[FRValue]) -> Result<~[FRValue], ~str> {
    Ok(~[String(args.head().FRtype_of().to_str())])
}

fn typeeq(_: &mut Context, args: ~[FRValue]) -> Result<~[FRValue], ~str> {
    Ok(~[String(format!("{}", args[0].FRtype_of() == args[1].FRtype_of()))])
}

fn test_macro(_args: ~[AST]) -> AST {
    AST {
        node: Literal(String(~"hi")),
        line: LineInfo {line: 0, startslice: 0, endslice: 0, startcol: 0, endcol: 0},
        typeinfo: @StringT
    }
}

pub fn register_stdlib(ctx: &mut Context) {
    ctx.global.define(~"+",      Function(~add, 2),         @Func(~[Float, Float, Float]));
    ctx.global.define(~"list",   Function(~list, 1),        @Func(~[Any, ListT]));
    ctx.global.define(~"typeof", Function(~FRtypeof, 1),    @Func(~[Any, StringT]));
    ctx.global.define(~"typeeq", Function(~typeeq, 2),      @Func(~[Any, Any, StringT]));
    ctx.global.macro(~"test_macro", ~test_macro);
}

