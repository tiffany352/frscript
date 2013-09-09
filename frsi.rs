extern mod frscript; 

use frscript::parse::*;
use frscript::grammar::*;
use std::io::*;
use frscript::eval::*;
use frscript::context::*;
use frscript::stdlib::*;
use frscript::typechecker::*;
use frscript::ast::*;

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
            _ => match parse(&grammar, grammar.grammar.get(& &"expr"), line, 0) {
                Err(e) => pretty_error(e.line, e.to_str()),
                Ok(tree) => match build_ast(&mut state.global, tree.clone()) {
                    Err(e) => pretty_error(e.line, e.to_str()),
                    Ok(ast) => match typecheck(&mut state.global, ast.clone()) {
                        Err(e) => pretty_error(e.line, e.to_str()),
                        Ok(ast) => match eval(&mut state, ast) {
                            Ok(v) => println(v.to_str()),
                            Err(e) => pretty_error(e.line, e.to_str())
                        }
                    }
                }
            }
        }
    }
}

