use grammar;
use parse::*;
use context;

#[deriving(Clone)]
pub enum FRType {
    List,
    Trait(~str),
    HasField(~str, ~FRType),
    Union(~[FRType]),
    Func(~[FRType]),
    String,
    Integer,
    Float,
    Any,
    Unit,
}

impl ToStr for FRType {
    fn to_str(&self) -> ~str {
        match self.clone() {
            List => ~"list",
            Trait(name) => name.clone(),
            HasField(name, T) => name + ": " + T.to_str(),
            Union(a) => a.map(|v| v.to_str()).connect(" + "),
            Func(a) => a.map(|v| v.to_str()).connect(" -> "),
            String => ~"str",
            Integer => ~"int",
            Float => ~"float",
            Any => ~"any",
            Unit => ~"()",
        }
    }
}

pub trait FRTypeOf {
    fn FRtype_of(&self) -> @FRType;
}

#[deriving(Clone)]
pub enum FRAst {
    Expr(~str, @FRType, ~[Token<FRAst>]),
    StringL(~str),
    IntegerL(int),
    FloatL(float),
    Var(~str, @FRType),
}

impl FRTypeOf for FRAst {
    fn FRtype_of(&self) -> @FRType {
        match self.clone() {
            Expr(_, t, _) => t,
            StringL(_) => @String,
            IntegerL(_) => @Integer,
            FloatL(_) => @Float,
            Var(_, t) => t
        }
    }
}

impl Eq for FRType {
    fn eq(&self, other: &FRType) -> bool {
        match (self.clone(), other.clone()) {
            (Union(x), Union(y))                => x.iter().zip(y.iter()).map(|(u, v)| u == v).all(|w| w),
            (Func(x), Func(y))                  => x.iter().zip(y.iter()).map(|(u, v)| u == v).all(|w| w),
            (Trait(x), Trait(y))                => x == y,
            (HasField(x, u), HasField(y, v))    => x == y && u == v,
            (List, List)                        => true,
            (String, String)                    => true,
            (Integer, Integer)                  => true,
            (Float, Float)                      => true,
            (Any, _)                            => true,
            (Unit, Unit)                        => true,
            _                                   => false
        }
    }
}

impl FRType {
    fn compatible(&self, other: &FRType) -> bool {
        match (self.clone(), other.clone()) {
            (List, List)                        => true,
            (Trait(x), Trait(y))                => x == y, // TODO: trait inheritance
            (HasField(x, u), HasField(y, v))    => x == y && u == v,
            (Union(x), Union(y))                => x.iter().map(|u| y.iter().any(|v| u == v)).all(|w| w),
            (Func(x), Func(y))                  => x.iter().map(|u| y.iter().any(|v| u == v)).all(|w| w),
            (String, String)                    => true,
            (Integer, Integer)                  => true,
            (Float, Float)                      => true,
            (Any, _)                            => true,
            (Unit, Unit)                        => true,
            _                                   => false
        }
    }
}

pub struct TypeError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for TypeError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + " " + self.msg
    }
}

pub fn get_type(scope: &context::Scope, name: ~str) -> Result<@FRType, ~str> {
    match scope.lookup(name.clone()) {
        Some(v) => Ok(v.FRtype_of()),
        None => Err(fmt!("No such variable %s", name)),
    }
}

pub fn typecheck(scope: &mut context::Scope, token: Token<grammar::FRToken>) -> Result<Token<FRAst>, TypeError> {
    match token.value.clone() {
        grammar::Unparsed(t) => Err(TypeError {msg: fmt!("Unexpected token: %?", t), line: token.line}),
        grammar::Whitespace => Err(TypeError {msg: fmt!("Unexpected whitespace token"), line: token.line}),
        grammar::Label(s) => match get_type(scope, s.clone()) {
            Ok(t) => Ok(Token {value: Var(s, t), line: token.line}),
            Err(s) => Err(TypeError {msg: s, line: token.line})
        },
        grammar::String(s) => Ok(Token {value: StringL(s), line: token.line}),
        grammar::Number(v) => Ok(Token {value: FloatL(v), line: token.line}),
        grammar::SExpr(expr) => {
            if expr.len() < 1 {
                return Err(TypeError {msg: ~"Null expression is invalid", line: token.line})
            }
            match expr.head().value.clone() {
                grammar::Label(s) => match get_type(scope, s.clone()) {
                    Ok(t) => {
                        match t.clone() {
                            @Func(ref sig) => {
                                let mut res = ~[];
                                if sig.len() != expr.len() {
                                    return Err(TypeError {msg: fmt!("Function takes %u parameters, %u provided", sig.len()-1, expr.len()-1), line: token.line})
                                }
                                for e in expr.tail().iter() {
                                    match typecheck(scope, e.clone()) {
                                        Ok(v) => res.push(v),
                                        Err(e) => return Err(e)
                                    }
                                }
                                for (T, v) in sig.iter().zip(res.iter()) {
                                    let T2 = match v.value.clone() {
                                        Expr(_, @Func(ref a), _) => @a.last().clone(),
                                        x => x.FRtype_of()
                                    };
                                    if T != T2 {
                                        return Err(TypeError {msg: fmt!("Mismatched types: Expected %s, got %s", T.to_str(), v.value.FRtype_of().to_str()), line: v.line})
                                    }
                                }
                                Ok(Token {value: Expr(s, t, res), line: token.line})
                            }
                            _ => Err(TypeError {msg: fmt!("Expected function, got %s", t.to_str()), line: token.line})
                        }
                    },
                    Err(s) => Err(TypeError {msg: s, line: token.line})
                },
                t => Err(TypeError {msg: fmt!("Expected label, got %?", t), line: token.line})
            }
        }
        grammar::FRSeq(t) => Err(TypeError {msg: fmt!("Unexpected token: %?", t), line: token.line}),
    }
}

