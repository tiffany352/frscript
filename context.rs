use std::hashmap::*;
use parse::*;
use types;
use ast::*;

pub struct EvalError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for EvalError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + ": " + self.msg.clone()
    }
}

impl types::FRTypeOf for FRValue {
    fn FRtype_of(&self) -> @types::FRType {
        match self.clone() {
            String(_) => @types::String,
            Number(_) => @types::Float,
            List(_) => @types::List,
            Function(_) => @types::Unit,
            Nil => @types::Unit,
        }
    }
}

pub struct Scope {
    atoms: HashMap<~str, (FRValue, @types::FRType)>,
    types: HashMap<~str, @types::FRType>,
    macros: HashMap<~str, ~extern fn(~[AST]) -> AST>
}

impl Scope {
    pub fn new() -> Scope {
        Scope {atoms: HashMap::new(), types: HashMap::new(), macros: HashMap::new()}
    }
    pub fn lookup(&self, name: ~str) -> Option<(FRValue, @types::FRType)> {
        self.atoms.find(&name).chain(|x| Some(x.clone()))
    }
    pub fn define(&mut self, name: ~str, val: FRValue, T: @types::FRType) {
        self.atoms.insert(name, (val, T));
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
    pub fn lookup(&self, name: ~str) -> Option<(FRValue, @types::FRType)> {
        for elem in self.stack.iter() {
            match elem.lookup(name.clone()) {
                Some(x) => return Some(x),
                None => ()
            }
        }
        self.global.lookup(name.clone())
    }
}

