use std::str::*;
use std::hashmap::*;

pub enum Pattern<'self, T> {
    // reference into a hash table containing the pattern
    Rule(&'self str),

    // matches and consumes
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
    //Match(~Pattern<'self, T>, ~fn(&Pattern<'self, T>, &Token<'self, T>) -> uint),

    // matches but doesn't consume
    And(~Pattern<'self, T>),
    Always,

    // controls visibility
    Merge(~Pattern<'self, T>),
    Select(uint, ~Pattern<'self, T>),
    Var(~Pattern<'self, T>),
    Ref(~Pattern<'self, T>),
    //Fold(~Pattern<'self, T>, ~fn(&'self Pattern<'self, T>, Token<'self, T>) -> Option<Token<'self, T>>),
    Build(~Pattern<'self, T>, ~fn(~str) -> Option<T>)
}

pub struct Token<'self, T> {
    children: ~[Token<'self, T>],
    value: T,
    pat: &'self Pattern<'self, T>,
    start: uint,
    end: uint
}

pub struct ParseContext<'self, T> {
    grammar: HashMap<&'self str, Pattern<'self, T>>,
    variables: HashMap<~str, Token<'self, T>>,
    make_token: ~fn(~str) -> T
}

impl<'self, T> ParseContext<'self, T> {
    pub fn new(make_token: ~fn(~str) -> T) -> ParseContext<'self, T> {
        ParseContext {grammar: HashMap::new(), make_token: make_token, variables: HashMap::new()}
    }
    pub fn rule(&mut self, name: &'self str, rule: Pattern<'self, T>) {
        self.grammar.insert(name, rule);
    }
}

pub fn parse<'a,'b, T:'static>(ctx: &'a ParseContext<'a, T>, pat: &'a Pattern<'a, T>, text: &str, position: uint) -> Option<Token<'a,T>> {
    let tok = |children, start, end| {
        Some(Token {children: children, value: (ctx.make_token)(text.slice(start, end).to_owned()), pat: pat, start: start+position, end: end+position})
    };
    match *pat {
        Rule(name) => parse(ctx, ctx.grammar.get(&name), text, position),
        Literal(s) => {
            if text.len() >= s.len() && text.slice_to(s.len()) == s {
                tok(~[], 0, s.len())
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
                tok(~[], 0, next)
            } else {
                None
            }
        }
        Chars(n) => {
            if text.len() >= n {
                tok(~[], 0, n)
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
                    return tok(~[], 0, next)
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
            tok(res, 0, acc)
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
            tok(res, 0, acc)
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
            tok(res, 0, acc)
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
            tok(res, 0, acc)
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
            tok(res, 0, acc)
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
        /*Match(ref p, ref f) => match parse(ctx, *p, text, position) {
            Some(x) => {
                let res = (*f)(*p, &x);
                let xstart = x.start;
                tok(~[x], xstart-position, res-position)
            }
            None => None
        },*/
        And(ref p) => match parse(ctx, *p, text, position) {
            Some(x) => tok(~[x], 0, 0),
            None => None
        },
        Always => tok(~[], 0, 0),
        Merge(ref p) => match parse(ctx, *p, text, position) {
            Some(x) => tok(~[], x.start-position, x.end-position),
            None => None
        },
        Select(n, ref p) => match parse(ctx, *p, text, position) {
            Some(x) => match x.children.len() {
                l if l <= n => None,
                _ => {
                    let start = x.start;
                    let end = x.end;
                    let t = x.children[n];
                    let v = t.value;
                    Some(Token {children: ~[], value: v, pat: pat, start: start, end: end})
                }
            },
            None => None
        },
        // var and ref would require adding another parameter to parse()
        /*Fold(ref p, ref f) => match parse(ctx, *p, text, position) {
            Some(x) => (*f)(*p, x),
            None => None
        },*/
        Build(ref p, ref f) => match parse(ctx, *p, text, position) {
            Some(x) => match (*f)(text.slice(x.start-position, x.end-position).to_owned()) {
                Some(v) => {
                    let s = x.start;
                    let e = x.end;
                    Some(Token {children: ~[x], value: v, pat: pat, start: s, end: e})
                }
                None => None
            },
            None => None
        },
        _ => fail!("NYI")
    }
}

