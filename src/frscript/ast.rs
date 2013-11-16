use parse::*;
use context;
use grammar;

#[deriving(Clone)]
pub enum FRType {
    ListT,
    Trait(~str),
    HasField(~str, ~FRType),
    Union(~[FRType]),
    Func(~[FRType]),
    ExprT {tin: ~[FRType], tout: ~[FRType]},
    StringT,
    Integer,
    Float,
    Any,
    Unit,
}

impl ToStr for FRType {
    fn to_str(&self) -> ~str {
        match self.clone() {
            ListT => ~"list",
            Trait(name) => name.clone(),
            HasField(name, T) => name + ": " + T.to_str(),
            Union(a) => a.map(|v| v.to_str()).connect(" + "),
            Func(a) => a.map(|v| v.to_str()).connect(" -> "),
            ExprT {tin: _, tout: tout} => tout.to_str(),
            StringT => ~"str",
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

impl FRTypeOf for FRValue {
    fn FRtype_of(&self) -> @FRType {
        match self.clone() {
            String(_)       => @StringT,
            Number(_)       => @Float,
            List(_)         => @ListT,
            Function(_,_)   => @Unit,
            Nil             => @Unit,
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
            (ListT, ListT)                      => true,
            (StringT, StringT)                  => true,
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
            (ListT, ListT)                      => true,
            (Trait(x), Trait(y))                => x == y, // TODO: trait inheritance
            (HasField(x, u), HasField(y, v))    => x == y && u == v,
            (Union(x), Union(y))                => x.iter().map(|u| y.iter().any(|v| u == v)).all(|w| w),
            (Func(x), Func(y))                  => x.iter().map(|u| y.iter().any(|v| u == v)).all(|w| w),
            (StringT, StringT)                  => true,
            (Integer, Integer)                  => true,
            (Float, Float)                      => true,
            (Any, _)                            => true,
            (Unit, Unit)                        => true,
            _                                   => false
        }
    }
}

#[deriving(Clone)]
pub enum FRValue {
    String(~str),
    Number(f32),
    List(~[FRValue]),
    Function(~extern fn(&mut context::Context,~[FRValue]) -> Result<~[FRValue], ~str>, uint),
    Nil
}

impl ToStr for FRValue {
    fn to_str(&self) -> ~str {
        match self.clone() {
            String(s)       => format!("\"{:s}\"", s),
            Number(n)       => format!("{:f}", n),
            List(l)         => "(" + l.map(|x| x.to_str()).connect(" ") + ")",
            Function(_,_)   => ~"function",
            Nil             => ~"()",
        }
    }
}

#[deriving(Clone)]
pub enum ASTNode {
    Expr(~[AST]),
    Var(~str),
    Literal(FRValue)
}

#[deriving(Clone)]
pub struct AST {
    node: ASTNode,
    line: LineInfo,
    typeinfo: @FRType,
}

pub struct ParseError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for ParseError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + " " + self.msg
    }
}

pub fn build_ast(scope: &mut context::Scope, tok: Token<grammar::FRToken>) -> Result<AST, ParseError> {
    let build_var = |name| Ok(AST {node: Var(name), line: tok.line, typeinfo: @Unit});
    let build_literal = |val: FRValue| Ok(AST {node: Literal(val.clone()), line: tok.line, typeinfo: val.FRtype_of()});
    let build_expr = |args| Ok(AST {node: Expr(args), line: tok.line, typeinfo: @Unit});
    match tok.value.clone() {
        grammar::Unparsed(_t) => Err(ParseError {msg: format!("Unexpected token: {:?}", tok.value), line: tok.line}),
        grammar::Whitespace => Err(ParseError {msg: ~"Unexpected whitespace token", line: tok.line}),
        grammar::FRSeq(_a) => Err(ParseError {msg: format!("Unexpected token: {:?}", tok.value), line: tok.line}),
        grammar::Label(s) => build_var(s),
        grammar::String(s) => build_literal(String(s)),
        grammar::Number(v) => build_literal(Number(v)),
        grammar::SExpr(_) => Err(ParseError {msg: format!("Unexpected token: {:?}", tok.value), line: tok.line}),
        /*if arr.len() < 1 {
            build_literal(Nil)
        } else {
            match arr[0].value.clone() {
                grammar::Label(s) => {
                    let mut res = ~[];
                    for t in arr.tail().iter() {
                        match build_ast(scope, t.clone()) {
                            Ok(v) => res.push(v),
                            Err(e) => return Err(e)
                        }
                    }
                    build_expr(s, res)
                }
                t => Err(ParseError {msg: format!("Expected atom, got {:?}", t), line: arr[0].line})
            }
        },*/
        grammar::Expr(ref arr) => {
            let mut res = ~[];
            for t in arr.iter() {
                match build_ast(scope, t.clone()) {
                    Ok(v) => res.push(v),
                    Err(e) => return Err(e)
                }
            }
            build_expr(res)
        }
    }
}

