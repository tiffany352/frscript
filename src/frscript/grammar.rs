use parse::*;
use std::from_str;
use std::vec;
use std::str::*;

#[deriving(Clone)]
pub enum FRToken {
    Unparsed(~str),
    Whitespace,
    Label(~str),
    String(~str),
    Number(f32),
    Bool(bool),
    SExpr(~[Token<FRToken>]),
    FRSeq(~[Token<FRToken>]),
    Expr(~[Token<FRToken>])
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

fn make_expr(tok: FRToken) -> Result<FRToken, ~str> {
    /* 
    FRSeq(~[
        parse::Token<grammar::FRToken>{
            value: Number(4f32)
        }, 
        parse::Token<grammar::FRToken>{
            value: FRSeq(~[
                parse::Token<grammar::FRToken>{
                    value: FRSeq(~[
                        parse::Token<grammar::FRToken>{
                            value: Number(4f32), 
                        }
                    ])
                }, 
                parse::Token<grammar::FRToken>{
                    value: FRSeq(~[
                        parse::Token<grammar::FRToken>{
                            value: Number(4f32), 
                        }
                    ])
                }
            ])
        }
    ])
    */
    //println!("{:?}", tok);
    match tok {
        FRSeq([start, Token {value: FRSeq(rest), line: _}]) => Ok(Expr(vec::append(~[start], rest.map(|x| match x.value {FRSeq([ref v]) => v.clone(), _ => fail!("???")})))),
        _ => Err(~"Failed to construct expr")
    }
}

/*fn count_ws(s: &str) -> (uint, uint, uint) {
    let mut spaces = 0;
    let mut tabs = 0;
    let mut i = 0;
    while (i < s.len()) {
        let CharRange {ch, next} = s.char_range_at(i);
        match ch {
            ' '     => spaces+=1,
            '\t'    => tabs+=1,
            _       => break
        }
        i = next;
    }
    (spaces, tabs, i)
}

fn match_block(ctx: &mut ParseContext<FRToken>, s: &str) -> Result<(FRToken, uint), ~str> {
    let (nspaces, ntabs, i) = count_ws(s);
    let mut i = i;
    let mut res = ~[];
    loop {
        let res = parse(ctx, &Rule("expr"), s.slice_from(i), i);
        let (spaces, tabs, offset) = count_ws(s.slice_from(i));
        if spaces < nspaces || tabs < ntabs {
            break;
        }
        i += offset;
    }
}*/

fn make_bool(tok: FRToken) -> Result<FRToken, ~str> {
    match tok {
        Unparsed(~"true") => Ok(Bool(true)),
        Unparsed(~"false") => Ok(Bool(false)),
        _ => Err(~"Failed to parse boolean")
    }
}

pub fn grammar() -> ParseContext<FRToken> {
    let mut ctx = ParseContext::new();
    let sws = || ~Rule("sws");
    let ws = || ~Rule("ws");
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
    //ctx.rule("sexpr",       ~Map(~Literal("(") * ~Rule("ws") * ~LessThan(1, ~Rule("expr")) * (~Rule("sws") * ~Rule("expr"))[0] * ~Rule("ws") * ~Literal(")"), make_sexpr));
    //ctx.rule("expr",        ~Rule("sexpr") + ~Rule("number") + ~Rule("atom") + ~Rule("string"));
    ctx.rule("toplevel",    ~Rule("def") + ~Rule("data") + ~Rule("impl"));
    ctx.rule("repl-stat",   ~Rule("toplevel") + ~Rule("expr"));
    ctx.rule("expr",        ~Map(~Rule("expratom") * ~More(sws() * ~Rule("expratom")), make_expr));
    ctx.rule("expratom",    ~Rule("literal") + ~Rule("atom") + ~Rule("control"));
    ctx.rule("literal",     ~Rule("number") + ~Rule("string") + ~Rule("boolean"));
    ctx.rule("boolean",     ~Map(~Literal("true") + ~Literal("false"), make_bool));
    ctx.rule("control",     ~Rule("if"));
    ctx.rule("if",          ~Literal("if") * ~Rule("sws") * ~Rule("expr") * ~Rule("sws") * ~Literal(":") * ~Rule("block"));
    ctx.rule("def",         ~Literal("def") * sws() * ~Rule("atom") * ~Literal(":") * sws() * ~Rule("block"));
    ctx.rule("data",        ~Literal("data") * sws() * ~Rule("atom") * ws() * ~Literal("::") * ws() * ~Rule("typespec"));
    ctx.rule("impl",        ~Literal("impl") * sws() * ~Rule("atom") * sws() * ~Rule("atom") * ~Literal(":") * ~Rule("implblock"));
    //ctx.rule("block",       ~Match(match_block));

    ctx
}

