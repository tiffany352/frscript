use std::str::*;
use std::hashmap::*;

#[deriving(Clone)]
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
    Build(~Pattern<'self, T>, extern fn(~str) -> Result<T, ~str>),
    Map(~Pattern<'self, T>, extern fn(T) -> Result<T, ~str>)
}

pub trait TokenCreator {
    fn sequence(~[Token<Self>]) -> Self;
    fn raw(~str) -> Self;
}

impl<'self, T:Clone> Mul<~Pattern<'self, T>, ~Pattern<'self, T>> for ~Pattern<'self, T> {
    fn mul(&self, rhs: &~Pattern<'self, T>) -> ~Pattern<'self, T> {
        match (*self.clone(), *rhs.clone()) {
            (Seq(x), y      ) => ~Seq(x + ~[y]),
            (x,      Seq(y) ) => ~Seq(~[x] + y),
            (x,      y      ) => ~Seq(~[x, y])
        }
    }
}

impl<'self, T:Clone> Add<~Pattern<'self, T>, ~Pattern<'self, T>> for ~Pattern<'self, T> {
    fn add(&self, rhs: &~Pattern<'self, T>) -> ~Pattern<'self, T> {
        match (*self.clone(), *rhs.clone()) {
            (Or(x), y     ) => ~Or(x + ~[y]),
            (x,     Or(y) ) => ~Or(~[x] + y),
            (x,     y     ) => ~Or(~[x, y])
        }
    }
}

impl<'self, T:Clone> Index<int, ~Pattern<'self, T>> for ~Pattern<'self, T> {
    fn index(&self, rhs: &int) -> ~Pattern<'self, T> {
        match (*rhs) {
            0 => ~More(self.clone()),
            x if x < 0 => ~LessThan(-x as uint, self.clone()),
            x => ~MoreThan(x as uint, self.clone())
        }
    }
}

#[deriving(Clone)]
pub struct LineInfo {
    line: int,
    startcol: uint,
    endcol: uint,
    startslice: uint,
    endslice: uint
}

impl LineInfo {
    pub fn new(text: &str, start: uint, end: uint) -> LineInfo {
        fn compute_line(text: &str, max: uint) -> (int, uint) {
            let mut line = 0;
            let mut offset = 0;
            let mut temp = 0;
            for c in text.iter() {
                if c == '\n' {
                    line += 1;
                    offset += temp;
                    temp = 0;
                }
                temp += 1;
                if temp >= max {
                    break;
                }
            }
            (line, offset)
        }
        let (line, offset) = compute_line(text, start);
        LineInfo {line: line, startcol: start-offset, endcol: end-offset, startslice: start, endslice: end}
    }
}

impl ToStr for LineInfo {
    fn to_str(&self) -> ~str {
        match (self.line, self.startcol, self.endcol) {
            (0, -1, -1) => ~"",
            (0, s, e)   => fmt!("[%u:%u]", s, e),
            (l, -1, -1) => fmt!("[line %i]", l),
            (l, s, e)   => fmt!("[line %i @ %u:%u]", l, s, e),
        }
    }
}

#[deriving(Clone)]
pub struct Token<T> {
    value: T,
    line: LineInfo
}

pub struct SyntaxError {
    pats: ~[~str],
    instead: Option<~str>,
    user_msg: Option<~str>,
    line: LineInfo
}

impl ToStr for SyntaxError {
    fn to_str(&self) -> ~str {
        fn pretty_arr(arr: ~[~str]) -> ~str {
            let len = arr.len();
            let mut s = arr[0].clone();
            if len > 2 {
                for i in range(1, len - 2) {
                    s.push_str(", ");
                    s.push_str(arr[i]);
                }
            }
            if len > 1 {
                s + ", or " + *arr.last()
            } else {
                s
            }
        }
        let mut err = self.line.to_str();
        err.push_str(fmt!(" Expected %s", pretty_arr(self.pats.clone())));
        match self.instead.clone() {
            Some(x) => err.push_str(fmt!(", got %s", x)),
            None => ()
        };
        match self.user_msg.clone() {
            Some(x) => err.push_str(fmt!(": %s", x)),
            None => ()
        };
        err
    }
}

pub struct ParseContext<'self, T> {
    grammar: HashMap<&'self str, Pattern<'self, T>>,
    variables: HashMap<~str, Token<T>>,
}

impl<'self, T> ParseContext<'self, T> {
    pub fn new() -> ParseContext<'self, T> {
        ParseContext {grammar: HashMap::new(), variables: HashMap::new()}
    }
    pub fn rule(&mut self, name: &'self str, rule: ~Pattern<'self, T>) {
        self.grammar.insert(name, *rule);
    }
}

pub fn parse<'a,'b, T:'static+Clone+TokenCreator>(ctx: &'a ParseContext<'a, T>, pat: &'a Pattern<'a, T>, text: &str, position: uint) -> Result<Token<T>, SyntaxError> {
    let tok: &fn(uint, uint) -> Result<Token<T>, SyntaxError> = |start, end| {
        Ok(Token {value: TokenCreator::raw(text.slice(start, end).to_owned()), line: LineInfo::new(text, start+position, end+position)})
    };
    let seq: &fn(~[Token<T>], uint, uint) -> Result<Token<T>, SyntaxError> = |children, start, end| {
        Ok(Token {value: TokenCreator::sequence(children), line: LineInfo::new(text, start+position, end+position)})
    };
    let err = |name, instead, start:uint, end:uint| {
        Err(SyntaxError {pats: ~[name], instead: instead, user_msg: None, line: LineInfo::new(text, start+position, end+position)})
    };
    match *pat {
        Rule(name) => parse(ctx, ctx.grammar.get(&name), text, position),
        Literal(s) => {
            if text.len() >= s.len() && text.slice_to(s.len()) == s {
                tok(0, s.len())
            } else {
                err(s.to_owned(), None, 0, s.len())
            }
        }
        Range(x, y) => {
            if text.char_len() < 1 {
                return err(fmt!("Character between %c and %c", x, y), Some(~"EOF"), 0, 1)
            }
            let CharRange{ch, next} = text.char_range_at(0);
            if ch <= y && ch >= x {
                tok(0, next)
            } else {
                err(fmt!("Character between %c and %c", x, y), None, 0, 1)
            }
        }
        Chars(n) => {
            if text.len() >= n {
                tok(0, n)
            } else {
                err(fmt!("%u characters", n), Some(~"EOF"), 0, n)
            }
        }
        Set(ref arr) => {
            if text.char_len() < 1 {
                return err(fmt!("One of %?", from_chars(*arr)), Some(~"EOF"), 0, 1)
            }
            let CharRange{ch, next} = text.char_range_at(0);
            for elem in arr.iter() {
                if *elem == ch {
                    return tok(0, next)
                }
            }
            err(fmt!("One of %?", from_chars(*arr)), None, 0, 1)
        }
        More(ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            while acc <= text.len() {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(_) => break
                }
            }
            seq(res, 0, acc)
        }
        MoreThan(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(x) => return Err(x)
                }
            }
            while acc <= text.len() {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(_) => break
                }
            }
            seq(res, 0, acc)
        }
        Exactly(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(x) => return Err(x)
                }
            }
            seq(res, 0, acc)
        }
        LessThan(n, ref p) => {
            let mut acc = 0;
            let mut res = ~[];
            for _ in range(0, n) {
                match parse(ctx, *p, text.slice_from(acc), acc + position) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(_) => break
                }
            }
            seq(res, 0, acc)
        }
        Seq(ref arr) => {
            let mut acc = 0;
            let mut res = ~[];
            for elem in arr.iter() {
                match parse(ctx, elem, text.slice_from(acc), position + acc) {
                    Ok(x) => {
                        acc = x.line.endslice - position;
                        res.push(x);
                    }
                    Err(x) => return Err(x)
                }
            }
            seq(res, 0, acc)
        }
        Or(ref arr) => {
            for elem in arr.iter() {
                match parse(ctx, elem, text, position) {
                    Ok(x) => return Ok(x),
                    Err(_) => {}
                }
            }
            err(~"Or", None, 0, text.len()) // TODO: Proper error messages here
        }
        Diff(ref a, ref b) => match parse(ctx, *b, text, position) {
            Ok(_) => err(fmt!("Not %?",b), None, 0, text.len()),
            Err(_) => parse(ctx, *a, text, position)
        },
        And(ref p) => match parse(ctx, *p, text, position) {
            Ok(x) => Ok(Token {value: x.value, line: LineInfo::new(text, position, position)}),
            Err(x) => Err(x)
        },
        Always(ref v) => Ok(Token {value: v.clone(), line: LineInfo::new(text, position, position)}),
        // var and ref would require adding another parameter to parse()
        Build(ref p, ref f) => match parse(ctx, *p, text, position) {
            Ok(x) => match (*f)(text.slice(x.line.startslice-position, x.line.endslice-position).to_owned()) {
                Ok(v) => {
                    Ok(Token {value: v, line: x.line})
                }
                Err(s) => {
                    Err(SyntaxError {pats: ~[fmt!("%?", p)], instead: None, user_msg: Some(s), line: x.line})
                }
            },
            Err(x) => Err(x)
        },
        Map(ref p, ref f) => match parse(ctx, *p, text, position) {
            Ok(x) => match (*f)(x.value.clone()) {
                Ok(v) => Ok(Token {value: v, line: x.line}),
                Err(s) => {
                    Err(SyntaxError {pats: ~[fmt!("%?", p)], instead: None, user_msg: Some(s), line: x.line})
                }
            },
            Err(x) => Err(x)
        },
        _ => fail!("NYI")
    }
}

