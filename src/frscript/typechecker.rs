use parse::*;
use context;
use ast::*;

pub struct TypeError {
    msg: ~str,
    line: LineInfo
}

impl ToStr for TypeError {
    fn to_str(&self) -> ~str {
        self.line.to_str() + " " + self.msg
    }
}

pub fn get_type(scope: &context::Scope, name: ~str) -> Result<@FRType, ~str> {
    match scope.lookup(name.clone()) {
        Some((_, t)) => Ok(t),
        None => Err(format!("No such variable {:s}", name)),
    }
}

pub fn typecheck(scope: &mut context::Scope, token: AST, typestack: ~[FRType]) -> Result<AST, TypeError> {
    //println!("typecheck(scope, {:?}, {:?})", token, typestack);
    let build_var = |name, T| Ok(AST {node: Var(name), line: token.line, typeinfo: T});
    let build_expr: &fn(~[AST], @FRType) -> Result<AST,TypeError> = |args, T| Ok(AST {node: Expr(args), line: token.line, typeinfo: T});
    let mut typestack = typestack;
    let oldstack = typestack.clone();
    match token.node.clone() {
        Expr(arr) => {
            for ast in arr.iter() {
                //println("----------------");
                //println!("ast {:?}", ast);
                match typecheck(scope, ast.clone(), typestack.clone()) {
                    Ok(AST {node: _, line: line, typeinfo: @ExprT {tin: ref tin, tout: ref tout}}) => {
                        //println!("expr {:?} {:?} {:?}", tin, tout, typestack);
                        if typestack.len() < tin.len() {
                            return Err(TypeError {msg: format!("Expression requires {:u} parameters, {:u} were on stack", tin.len(), typestack.len()), line: token.line})
                        }
                        for i in range(0, tin.len()) {
                            let ti = i + typestack.len() - tin.len();
                            if tin[i] != typestack[ti] {
                                return Err(TypeError {msg: format!(r"Bad argument \#{:u}: Expected {:s}, got {:s}", 
                                                                   i+1, 
                                                                   tin[i].to_str(), 
                                                                   typestack[ti].to_str()
                                                                  ), line: line})
                            }
                        }
                        let len = typestack.len();
                        typestack.truncate(len - tin.len());
                        typestack.push_all_move(tout.clone());
                    }
                    Ok(AST {node: ref node, line: line, typeinfo: @Func(ref arr)}) => {
                        //println!("func {:?}", arr);
                        if typestack.len() < (arr.len() - 1) {
                            return Err(TypeError {msg: format!("Function requires {:u} parameters, {:u} were on stack", arr.len()-1, typestack.len()), line: token.line})
                        }
                        for i in range(0, arr.len()-1) {
                            let ti = i + typestack.len() - (arr.len()-1);
                            if arr[i] != typestack[ti] {
                                return Err(TypeError {msg: format!(r"Bad argument \#{:u}{:s}: Expected {:s}, got {:s}",
                                                                   i+1,
                                                                   match node.clone() {
                                                                       Var(name) => ~" to "+name,
                                                                       _ => ~""
                                                                   },
                                                                   arr[i].to_str(),
                                                                   typestack[ti].to_str()
                                                                  ), line: line})
                            }
                        }
                        let len = typestack.len();
                        typestack.truncate(len - (arr.len() - 1));
                        typestack.push(arr.last().clone());
                    }
                    Ok(AST {node: _, line: _, typeinfo: @ref T}) => {
                        //println!("val {:?}", T);
                        typestack.push(T.clone())
                    }
                    Err(x) => return Err(x)
                }
            }
            build_expr(arr, @ExprT {tin: oldstack, tout: typestack})
        },
        /*match get_type(scope, atom.clone()) {
            Ok(t) => {
                match t.clone() {
                    @Func(ref sig) => {
                        let mut res = ~[];
                        if sig.len() - 1 != args.len() {
                            return Err(TypeError {msg: format!("Function takes {:u} parameters, {:u} provided", sig.len()-1, args.len()), line: token.line})
                        }
                        for e in args.iter() {
                            match typecheck(scope, e.clone()) {
                                Ok(v) => res.push(v),
                                Err(e) => return Err(e)
                            }
                        }
                        for (T, v) in sig.iter().zip(res.iter()) {
                            let T2 = match (v.node.clone(), v.typeinfo.clone()) {
                                (Expr(_, _), @Func(ref a)) => @a.last().clone(),
                                (_, x) => x
                            };
                            if T != T2 {
                                return Err(TypeError {msg: format!("Mismatched types: Expected {:s}, got {:s}", T.to_str(), v.typeinfo.to_str()), line: v.line})
                            }
                        }
                        build_expr(atom, res, t)
                    }
                    _ => Err(TypeError {msg: format!("Expected function, got {:s}", t.to_str()), line: token.line})
                }
            },
            Err(s) => Err(TypeError {msg: s, line: token.line})
        },*/
        Var(s) => match get_type(scope, s.clone()) {
            Ok(t) => build_var(s, t),
            Err(s) => Err(TypeError {msg: s, line: token.line})
        },
        Literal(_) => Ok(token),
    }
}

