use std::hashmap::*;
use ast::*;

pub struct Scope {
    atoms: HashMap<~str, (FRValue, @FRType)>,
    types: HashMap<~str, @FRType>,
    macros: HashMap<~str, ~extern fn(~[AST]) -> AST>
}

impl Scope {
    pub fn new() -> Scope {
        Scope {atoms: HashMap::new(), types: HashMap::new(), macros: HashMap::new()}
    }
    pub fn lookup(&self, name: ~str) -> Option<(FRValue, @FRType)> {
        self.atoms.find(&name).and_then(|x| Some(x.clone()))
    }
    pub fn lookup_macro(&self, name: ~str) -> Option<~extern fn(~[AST]) -> AST> {
        self.macros.find(&name).and_then(|x| Some(x.clone()))
    }
    pub fn define(&mut self, name: ~str, val: FRValue, T: @FRType) {
        self.atoms.insert(name, (val, T));
    }
    pub fn macro(&mut self, name: ~str, f: ~extern fn(~[AST]) -> AST) {
        self.macros.insert(name, f);
    }
}

pub struct Context {
    global: Scope,
    stack: ~[Scope],
}

impl Context {
    pub fn new() -> Context {
        Context {global: Scope::new(), stack: ~[]}
    }
    pub fn lookup(&self, name: ~str) -> Option<(FRValue, @FRType)> {
        for elem in self.stack.iter() {
            match elem.lookup(name.clone()) {
                Some(x) => return Some(x),
                None => ()
            }
        }
        self.global.lookup(name.clone())
    }
    pub fn lookup_macro(&self, name: ~str) -> Option<~extern fn(~[AST]) -> AST> {
        for elem in self.stack.iter() {
            match elem.lookup_macro(name.clone()) {
                Some(x) => return Some(x),
                None => ()
            }
        }
        self.global.lookup_macro(name.clone())
    }
}

