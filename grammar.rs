use parse::*;
use std::float;

pub enum FRToken {
    Unparsed(~str),
    Label(~str),
    String(~str),
    Number(float),
    SExpr
}

fn make_number(s: ~str) -> Option<FRToken> {
    match float::from_str(s) {
        Some(x) => Some(Number(x)),
        None => None
    }
}

fn grammar() -> ParseContext<FRToken> {
    let mut ctx = ParseContext::new(|s| Unparsed(s));
    ctx.rule("space", Set(" \t\n".iter().collect()));
    ctx.rule("ws", More(~Rule("space")));
    ctx.rule("sws", MoreThan(1, ~Rule("space")));
    ctx.rule("digit", Range('0','9'));
    ctx.rule("digits", MoreThan(1, ~Rule("digit")));
    ctx.rule("alpha", Or(~[Range('a','z'), Range('A','Z')]));
    ctx.rule("number", Build(~Seq(~[LessThan(1, ~Literal("-")), Rule("digits"), LessThan(1, ~Seq(~[Literal("."), Rule("digits")])), LessThan(1, ~Seq(~[Set("eE".iter().collect()), LessThan(1, ~Literal("-")), Rule("digits")]))]), make_number));
    ctx.rule("symbol", Set("~!@#$%^&*_-+=/<>'".iter().collect()));
    ctx.rule("atom", Build(~MoreThan(1, ~Or(~[Rule("alpha"), Rule("digit"), Rule("symbol")])), |s| Some(Label(s))));
    ctx.rule("string_mid", Build(~More(~Diff(~Or(~[Literal("\\\""), Chars(1)]), ~Literal("\""))), |s| Some(String(s))));
    ctx.rule("string", Select(1, ~Seq(~[Literal("\""), Rule("string_mid"), Literal("\"")])));
    ctx.rule("sexpr", Build(~Seq(~[Literal("("), More(~Seq(~[Rule("ws"), Rule("expr")])), Rule("ws"), Literal(")")]), |s| Some(SExpr)));
    ctx.rule("expr", Or(~[Rule("sexpr"), Rule("atom"), Rule("number")]));

    ctx
}

