use std::hashmap::*;
use parse::*;

pub struct EvalError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for EvalError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + ": " + self.msg.clone()
    }
}

#[deriving(Clone)]
pub enum FRValue {
    String(~str),
    Number(float),
    List(~[FRValue]),
    Function(~extern fn(&mut Context,~[FRValue]) -> Result<FRValue, ~str>)
}

impl ToStr for FRValue {
    fn to_str(&self) -> ~str {
        match self.clone() {
            String(s)   => fmt!("\"%s\"", s),
            Number(n)   => fmt!("%f", n),
            List(l)     => "(" + l.map(|x| x.to_str()).connect(" ") + ")",
            Function(_) => ~"<Function>"
        }
    }
}

pub struct Scope {
    atoms: HashMap<~str, FRValue>
}

impl Scope {
    pub fn new() -> Scope {
        Scope {atoms: HashMap::new()}
    }
    pub fn lookup(&self, name: ~str) -> Option<FRValue> {
        self.atoms.find(&name).chain(|x| Some(x.clone()))
    }
}

pub struct Context {
    global: Scope,
    stack: ~[Scope]
}

impl Context {
    pub fn new() -> Context {
        Context {global: Scope::new(), stack: ~[]}
    }
    pub fn lookup(&self, name: ~str) -> Option<FRValue> {
        for elem in self.stack.iter() {
            match elem.lookup(name.clone()) {
                Some(x) => return Some(x),
                None => ()
            }
        }
        match self.global.lookup(name.clone()) {
            Some(x) => Some(x),
            None => None
        }
    }
}

