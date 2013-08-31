extern mod frscript; 

use frscript::parse::*;
use frscript::grammar::*;
use std::io::*;

fn main() {
    let ctx = grammar();
    loop {
        print("= ");
        let line = stdin().read_line();
        match line {
            ~"quit" => return,
            ~"exit" => return,
            _ => {
                let res = parse(&ctx, ctx.grammar.get(& &"sexpr"), line, 0);
                println(fmt!("%?", res));
            }
        }
    }
}

