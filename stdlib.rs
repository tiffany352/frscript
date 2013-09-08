use context::*;
use context;
use types::*;

fn add(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    let mut sum = 0f;
    for v in args.iter() {
        match *v {
            Number(n) => sum += n,
            _ => return Err(fmt!("Expected number, got %?", v))
        }
    }
    Ok(Number(sum))
}

fn list(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(context::List(args))
}

fn typeof(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(context::String(args.head().FRtype_of().to_str()))
}

fn typeeq(_: &mut Context, args: ~[FRValue]) -> Result<FRValue, ~str> {
    Ok(context::String(fmt!("%b", args[0].FRtype_of() == args[1].FRtype_of())))
}

fn register_stdlib(ctx: &mut Context) {
    ctx.global.atoms.insert(~"+", Function(@Func(~[Float, Float, Float]), ~add));
    ctx.global.atoms.insert(~"list", Function(@Func(~[Any, List]), ~list));
    ctx.global.atoms.insert(~"typeof", Function(@Func(~[Any, String]), ~typeof));
    ctx.global.atoms.insert(~"typeeq", Function(@Func(~[Any, Any, String]), ~typeeq));
}

