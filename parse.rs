use std::str::*;
use std::hashmap::*;

pub enum Pattern<'self, T> {
    // reference into a hash table containing the pattern
    Rule(&'self str),

    // lexing
    Literal(&'self str),
    Range(char, char),
    Chars(uint),
    Set(~[char]),
    More(~Pattern<'self, T>),
    MoreThan(uint, ~Pattern<'self, T>),
    Exactly(uint, ~Pattern<'self, T>),
    LessThan(uint, ~Pattern<'self, T>),
    Seq(~[Pattern<'self, T>]),
    Or(~[Pattern<'self, T>]),
    Diff(~Pattern<'self, T>, ~Pattern<'self, T>),

    // matches but doesn't consume
    And(~Pattern<'self, T>),
    Always(T),

    // parsing
    Var(~Pattern<'self, T>),
    Ref(~Pattern<'self, T>),
    Build(~Pattern<'self, T>, ~fn(~str) -> Option<T>),
    Map(~Pattern<'self, T>, ~fn(T) -> Option<T>)
}

#[deriving(Clone)]
pub struct Token<T> {
    value: T,
    start: uint,
    end: uint
}

pub struct ParseContext<'self, T> {
    grammar: HashMap<&'self str, Pattern<'self, T>>,
    variables: HashMap<~str, Token<T>>,
    make_token: ~fn(~str) -> T,
    make_sequence: ~fn(~[Token<T>]) -> T
}

impl<'self, T> ParseContext<'self, T> {
    pub fn new(make_token: ~fn(~str) -> T, make_sequence: ~fn(~[Token<T>]) -> T) -> ParseContext<'self, T> {
        ParseContext {grammar: HashMap::new(), make_token: make_token, make_sequence: make_sequence, variables: HashMap::new()}
    }
    pub fn rule(&mut self, name: &'self str, rule: Pattern<'self, T>) {
        self.grammar.insert(name, rule);
    }
}

pub fn parse<'a,'b, T:'static+Clone>(ctx: &'a ParseContext<'a, T>, pat: &'a Pattern<'a, T>, text: &str, position: uint) -> Option<Token<T>> {
    let tok = |start, end| {
        Some(Token {value: (ctx.make_token)(text.slice(start, end).to_owned()), start: start+position, end: end+position})
    };
    let seq = |children, start:uint, end:uint| {
        Some(Token {value: (ctx.make_sequence)(children), start: start+position, end: end+position})
    };
    match *pat {
        Rule(name) => parse(ctx, ctx.grammar.get(&name), text, position),
        Literal(s) => {
            if text.len() >= s.len() && text.slice_to(s.len()) == s {
                tok(0, s.len())
            } else {
                None
            }
        }
        Range(x, y) => {
            if text.char_len() < 1 {
                return None
            }
            let CharRange{ch, next} = text.char_range_at(0);
            if ch <= y && ch >= x {
                tok(0, next)
            } else {
                None
            }
        }
        Chars(n) => {
            if text.len() >= n {
                tok(0, n)
            } else {
                None
            }
        }
        Set(ref arr) => {
            if text.char_len() < 1 {
                return None
            }
            let CharRange{ch, next} = text.char_range_at(0);
            for elem in arr.iter() {
                if *elem == ch {
                    return tok(0, next)
                }
            }
            None
        }
        More(ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            while acc <= text.len() {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => break
                }
            }
            seq(res, 0, acc)
        }
        MoreThan(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => return None
                }
            }
            while acc <= text.len() {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => break
                }
            }
            seq(res, 0, acc)
        }
        Exactly(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => return None
                }
            }
            seq(res, 0, acc)
        }
        LessThan(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => break
                }
            }
            seq(res, 0, acc)
        }
        Seq(ref arr) => {
            let mut acc = 0;
            let mut res = ~[];
            for elem in arr.iter() {
                match parse(ctx, elem, text.slice_from(acc), position + acc) {
                    Some(x) => {
                        acc = x.end - position;
                        res.push(x);
                    }
                    None => return None
                }
            }
            seq(res, 0, acc)
        }
        Or(ref arr) => {
            for elem in arr.iter() {
                match parse(ctx, elem, text, position) {
                    Some(x) => return Some(x),
                    None => {}
                }
            }
            None
        }
        Diff(ref a, ref b) => match parse(ctx, *b, text, position) {
            Some(_) => None,
            None => parse(ctx, *a, text, position)
        },
        And(ref p) => match parse(ctx, *p, text, position) {
            Some(x) => Some(Token {value: x.value, start: 0, end: 0}),
            None => None
        },
        Always(ref v) => Some(Token {value: v.clone(), start: 0, end: 0}),
        // var and ref would require adding another parameter to parse()
        Build(ref p, ref f) => match parse(ctx, *p, text, position) {
            Some(x) => match (*f)(text.slice(x.start-position, x.end-position).to_owned()) {
                Some(v) => {
                    let s = x.start;
                    let e = x.end;
                    Some(Token {value: v, start: s, end: e})
                }
                None => None
            },
            None => None
        },
        Map(ref p, ref f) => match parse(ctx, *p, text, position) {
            Some(x) => match (*f)(x.value.clone()) {
                Some(v) => Some(Token {value: v, start: x.start, end: x.end}),
                None => None
            },
            None => None
        },
        _ => fail!("NYI")
    }
}

