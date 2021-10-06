extern crate regex;
extern crate rustyline;
extern crate fnv;
extern crate lazy_static;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use fnv::FnvHashMap;
use std::rc::Rc;



#[macro_use]
#[allow(dead_code)]
#[allow(unused_imports)]
mod types;
use crate::types::MalErr::{ErrString, ErrMalVal};
use crate::types::{MalVal, MalArgs, MalRet, MalErr, MapKey};
use crate::types::MalVal::{Func, Int, Sym, List, Vector, Hash, Nil};
use crate::types::{error, format_error};

#[allow(dead_code)]
#[allow(unused_imports)]
mod reader;

#[allow(unused_variables)]
#[allow(unused_imports)]
mod printer;

#[allow(unused_imports)]
mod env;
use crate::env::{Env, make_env};

// read
fn read(str: &str) -> MalRet {    
    reader::read_str(str.to_string())
}

// eval
fn eval_ast(ast: &MalVal, env: &mut Env) -> MalRet {
    // println!("eval_ast {:?}", ast);
    match ast {
        Sym(s) => env.get(s.to_string()),
        // eval list args
        List(args, _) => {
            let mut v: MalArgs = vec![];
            for mv in args.iter() {                
                v.push(eval(mv, env)?)
            }
            Ok(list!(v))
        },
        // eval vectors
        Vector(args, _) => {
            let mut v: MalArgs = vec![];
            for mv in args.iter() {
                v.push(eval(mv, env)?)
            }
            Ok(vector!(v))
        },
        // eval hash keys and vals
        Hash(kvs, _ ) => {
            let mut hm: FnvHashMap<MapKey, MalVal> = FnvHashMap::default();
            for (k, v) in kvs.iter() {
                // TODO: why clone??
                // TODO: should eval k too
                hm.insert(k.clone(), eval(v, env)?);
            }
            Ok(Hash(Rc::new(hm), Rc::new(Nil)))
        }
        _ => Ok(ast.clone()),
    }
}

// toplevel eval
fn eval(ast: &MalVal, env: &mut Env) -> MalRet {    
    match ast {
        // eval toplevel form: if it's a list it can be multiple things
        List(v, _) => {
            if v.len() == 0 {
                return Ok(ast.clone())
            }
            
            let first = &v[0];
            match first {
                // (def! binding val)
                Sym(s) if s == "def!" => {                            
                    let binding = v.get(1).ok_or(ErrString(format!("No binding for expression: {:?}", v))).unwrap();
                    let val = v.get(2).ok_or(ErrString(format!("No value for expression: {:?}", v))).unwrap();
                    let bindval = eval(val, env)?;
                    env.set(binding.to_string(), bindval.clone())?;
                                                
                    // println!("sym: {:?} binding {:?}", binding, val);
                    return Ok(bindval);
                },
                // (let* [binding1 val1
                //        bindingN valN] 
                //    body)
                Sym(s) if s == "let*" => {
                    let bindings = v.get(1).ok_or(ErrString(format!("No bindings for expression: {:?}", v))).unwrap();
                    let body = v.get(2).ok_or(Nil).unwrap(); // let can have empty bindings
                    let mut let_env = make_env(Some(env.clone()));

                    // TODO bindings should be a List of (sym, something, sym something)
                    // body can be anything
                    // - should eval the list within a new environment
                    // - the eval the body with this new environment
                    // - then discard/pop the new environment
                    match bindings {
                        List(bindvec, _) | Vector(bindvec, _) => {
                            if bindvec.len() %2 != 0 {
                                return error(&format!("let* bindings: mismatch"))
                            }

                            // create bindings
                            for pair in bindvec.chunks(2) {
                                match pair {
                                    [Sym(binding), expr] => {
                                        let res = eval(expr, &mut let_env)?;
                                        let_env.set(binding.to_string(), res)?;
                                    },
                                    _ => return error(&format!("let* bindings: binding is not a symbol"))
                                }
                            }

                            // evaluate body with inner env
                            // println!("Gonna evaluate body {:?} with env {:?}", body, let_env);
                            return eval(body, &mut let_env);
                        },
                        _  => error(&format!("let* bindings: not a list"))
                    }
                },

                // regular function call
                // (+ 1 1)
                _ => match eval_ast(ast, env)? {
                    List(v, _) => {
                        let (fcall, fargs) = v.split_at(1);
                        match fcall {
                            [Func(f)]  => return (*f)(fargs.to_vec()),
                            [Sym(unk)] => error(&format!("'{:}' not found", unk)),
                            _          => error(&format!("'{:?}' not found", fcall.get(0))),
                        }
                    },
                    _ => error("Expected a list")
                }
            }
        },
        // if toplevel is not a list, evaluate its AST
        rst => eval_ast(&rst, env)
    }
}

// print
fn print(ast: &MalVal) -> String {
    ast.pr_str()
}

fn rep(str: &str, env: &mut Env) -> Result<String, MalErr> {
    let ast = read(str)?;    
    let exp = eval(&ast, env)?;
    Ok(print(&exp))
}

fn op(args: MalArgs, f: fn(i1: i64, i2: i64) -> i64) -> MalRet {
    if args.len() == 0 || args.len() < 2 {
        return Err(ErrString(format!("Insufficient arguments: {} ", args.len())));
    }
    // println!("eval {:?}", args);
    let res = args
                .iter()
                .map(|mv| match &mv {
                    Int(i) => *i,
                    _      => panic!("Eval op {:?}; cannot eval datatype", args)
                })
                .reduce(f)
                .map(|i| Int(i))
                .ok_or(ErrString(format!("Could not apply op: {:?} ", args)));
    return res;
}


fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history.");
    }

    // prepare Env
    let mut env = make_env(None);
    env.set("+".to_string(), Func(|args: MalArgs| op(args, |a,b| a+b)));
    env.set("-".to_string(), Func(|args: MalArgs| op(args, |a,b| a-b)));
    env.set("*".to_string(), Func(|args: MalArgs| op(args, |a,b| a*b)));
    env.set("/".to_string(), Func(|args: MalArgs| op(args, |a,b| a/b)));
    
    loop {

        // read
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".mal-history").unwrap();

                if line.len() > 0 {
                    let res = rep(&line, &mut env);
                    match res {
                        Ok(out) => println!("{}", out),
                        Err(e) => println!("Error: {}", format_error(e)),
                    }
                }
            }
            Err(ReadlineError::Interrupted) => continue,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}
