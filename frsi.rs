extern mod frscript; 

use frscript::parse::*;
use frscript::grammar::*;
use std::io::*;
use frscript::eval::*;
use frscript::context::*;
use frscript::stdlib::*;
use frscript::types::*;

fn pretty_error(line: LineInfo, err: ~str) {
    for _ in range(0, line.startslice+2) {
        print(" ");
    }
    for _ in range(line.startslice, line.endslice) {
        print("^");
    }
    println("");
    println(err);
}

fn main() {
    let grammar = grammar();
    let mut state = Context::new();
    register_stdlib(&mut state);
    loop {
        print("= ");
        let line = stdin().read_line();
        match line {
            ~"quit" => return,
            ~"exit" => return,
            _ => {
                match parse(&grammar, grammar.grammar.get(& &"sexpr"), line, 0) {
                    Err(e) => pretty_error(e.line, e.to_str()),
                    Ok(x) => {
                        match typecheck(&mut state.global, x.clone()) {
                            Err(e) => pretty_error(e.line, e.to_str()),
                            Ok(x) => match eval(&mut state, x) {
                                Ok(v) => println(v.to_str()),
                                Err(e) => pretty_error(e.line, e.to_str())
                            }
                        }
                    }
                }
            }
        }
    }
}

