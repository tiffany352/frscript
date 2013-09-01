use context::*;

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
    Ok(List(args))
}

fn register_stdlib(ctx: &mut Context) {
    ctx.global.atoms.insert(~"+", Function(~add));
    ctx.global.atoms.insert(~"list", Function(~list));
}

