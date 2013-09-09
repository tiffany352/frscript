use parse::*;
use types::*;
use context;
use grammar;

#[deriving(Clone)]
pub enum FRValue {
    String(~str),
    Number(float),
    List(~[FRValue]),
    Function(~extern fn(&mut context::Context,~[FRValue]) -> Result<FRValue, ~str>),
    Nil
}

impl ToStr for FRValue {
    fn to_str(&self) -> ~str {
        match self.clone() {
            String(s)       => fmt!("\"%s\"", s),
            Number(n)       => fmt!("%f", n),
            List(l)         => "(" + l.map(|x| x.to_str()).connect(" ") + ")",
            Function(_)     => ~"function",
            Nil             => ~"()",
        }
    }
}

pub enum ASTNode {
    Expr(~str, ~[AST]),
    Var(~str),
    Literal(FRValue)
}

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
    let build_expr = |atom, args| Ok(AST {node: Expr(atom, args), line: tok.line, typeinfo: @Unit});
    match tok.value.clone() {
        grammar::Unparsed(t) => Err(ParseError {msg: fmt!("Unexpected token: %?", t), line: tok.line}),
        grammar::Whitespace => Err(ParseError {msg: ~"Unexpected whitespace token", line: tok.line}),
        grammar::FRSeq(a) => Err(ParseError {msg: fmt!("Unexpected token: %?", a), line: tok.line}),
        grammar::Label(s) => build_var(s),
        grammar::String(s) => build_literal(String(s)),
        grammar::Number(v) => build_literal(Number(v)),
        grammar::SExpr(ref arr) => if arr.len() < 1 {
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
                t => Err(ParseError {msg: fmt!("Expected atom, got %?", t), line: arr[0].line})
            }
        },
    }
}

