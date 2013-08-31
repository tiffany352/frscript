extern mod frscript; 

use frscript::parse::*;
use frscript::grammar::*;
use std::io::*;

fn pretty_error(err: Error) {
    for _ in range(0, err.start+2) {
        print(" ");
    }
    for _ in range(err.start, err.end) {
        print("^");
    }
    println("");
    println(err.to_str());
}

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
                match res {
                    Ok(x) => println(fmt!("%?", x)),
                    Err(x) => pretty_error(x)
                }
            }
        }
    }
}

