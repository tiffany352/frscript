use parse::*;
use std::from_str;

#[deriving(Clone)]
pub enum FRToken {
    Unparsed(~str),
    Whitespace,
    Label(~str),
    String(~str),
    Number(f32),
    SExpr(~[Token<FRToken>]),
    FRSeq(~[Token<FRToken>])
}

impl TokenCreator for FRToken {
    fn sequence(arr: ~[Token<FRToken>]) -> FRToken {
        FRSeq(arr.iter()
                 .filter(|x| match x.value {Whitespace=> false, _=>true})
                 .map(|x| x.clone())
                 .collect())
    }
    fn raw(s: ~str) -> FRToken {
        Unparsed(s)
    }
}

fn make_number(s: ~str) -> Result<FRToken, ~str> {
    match from_str::from_str::<f32>(s) {
        Some(x) => Ok(Number(x)),
        None => Err(~"Failed to parse number")
    }
}

fn make_sexpr(tok: FRToken) -> Result<FRToken, ~str> {
    fn recurse(arr: ~[Token<FRToken>]) -> ~[Token<FRToken>] {
        arr.flat_map(|x| match x.value.clone() {FRSeq(a) => recurse(a), Unparsed(_) => ~[], _ => ~[x.clone()]})
    }
    match tok {
        FRSeq(a) => Ok(SExpr(recurse(a))),
        _ => Err(~"Failed to constuct SExpr")
    }
}

fn make_string_mid(text: ~str) -> Result<FRToken, ~str> {
    Ok(Unparsed(text))
}

fn make_string(tok: FRToken) -> Result<FRToken, ~str> {
    match tok {
        FRSeq(a) => match a[1].value {
            Unparsed(s) => Ok(String(s)),
            _ => Err(~"Failed to construct string")
        },
        _ => Err(~"Failed to construct string")
    }
}

fn make_whitespace(_: ~str) -> Result<FRToken, ~str> {
    Ok(Whitespace)
}

fn make_label(s: ~str) -> Result<FRToken, ~str> {
    Ok(Label(s))
}

pub fn grammar() -> ParseContext<FRToken> {
    let mut ctx = ParseContext::new();
    ctx.rule("space",       ~Set(" \t\n".iter().collect()));
    ctx.rule("ws",          ~Build(~More(~Rule("space")), make_whitespace));
    ctx.rule("sws",         ~Build(~MoreThan(1, ~Rule("space")), make_whitespace));
    ctx.rule("digit",       ~Range('0','9'));
    ctx.rule("digits",      ~MoreThan(1, ~Rule("digit")));
    ctx.rule("alpha",       ~Range('a','z') + ~Range('A','Z'));
    ctx.rule("number",      ~Build(~LessThan(1, ~Literal("-")) * ~Rule("digits") * ~LessThan(1, ~Literal(".") * ~Rule("digits")) * ~LessThan(1, ~Set("eE".iter().collect()) * ~LessThan(1, ~Literal("-")) * ~Rule("digits")), make_number));
    ctx.rule("symbol",      ~Set("~!@#$%^&*_-+=/<>'".iter().collect()));
    ctx.rule("atom",        ~Build((~Rule("alpha") + ~Rule("digit") + ~Rule("symbol"))[1], make_label));
    ctx.rule("string_mid",  ~Build(~More(~Diff(~Literal("\\\"") + ~Chars(1), ~Literal("\""))), make_string_mid));
    ctx.rule("string",      ~Map(~Literal("\"") * ~Rule("string_mid") * ~Literal("\""), make_string));
    ctx.rule("sexpr",       ~Map(~Literal("(") * ~Rule("ws") * ~LessThan(1, ~Rule("expr")) * (~Rule("sws") * ~Rule("expr"))[0] * ~Rule("ws") * ~Literal(")"), make_sexpr));
    ctx.rule("expr",        ~Rule("sexpr") + ~Rule("number") + ~Rule("atom") + ~Rule("string"));

    ctx
}

