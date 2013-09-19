extern mod frscript; 

use frscript::parse::*;
use frscript::grammar::*;
use std::io::*;
use frscript::eval::*;
use frscript::context::*;
use frscript::stdlib::*;
use frscript::typechecker::*;
use frscript::ast::*;
use frscript::macro::*;

fn pretty_error(line: LineInfo, err: ~str) -> ~str {
    let mut s = ~"";
    for _ in range(0, line.startslice+2) {
        s = s + " "
    }
    for _ in range(line.startslice, line.endslice) {
        s = s + "^";
    }
    s + "\n" + err
}

fn main() {
    let grammar = grammar();
    let mut state = Context::new();
    register_stdlib(&mut state);
    loop {
        print("= ");
        let line = stdin().read_line();
        if line == ~"quit" || line == ~"exit" {
            return
        }
        let res = parse(&grammar, grammar.grammar.get(& &"expr"), line, 0)      .map_err(|e| pretty_error(e.line, e.to_str()))
                 .and_then(|tree|  build_ast(&mut state.global, tree.clone())   .map_err(|e| pretty_error(e.line, e.to_str())))
                 .and_then(|ast|   expand_macros(&mut state, ast.clone())       .map_err(|e| pretty_error(e.line, e.to_str())))
                 .and_then(|ast|   typecheck(&mut state.global, ast.clone())    .map_err(|e| pretty_error(e.line, e.to_str())))
                 .and_then(|ast|   eval(&mut state, ast)                        .map_err(|e| pretty_error(e.line, e.to_str())));
        println(match res {
            Ok(v) => v.to_str(),
            Err(e) => e.to_str()
        })
    }
}

