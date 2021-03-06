extern crate regex;
extern crate rustyline;
extern crate fnv;
extern crate lazy_static;

use rustyline::error::ReadlineError;
use rustyline::Editor;
use fnv::FnvHashMap;
use std::rc::Rc;

use crate::types::MalErr::{ErrString, ErrMalVal};
use crate::types::{MalVal, MalArgs, MalRet, MalErr, MapKey};
use crate::types::MalVal::{Func, Int, Sym, List, Vector, Hash, Nil};
use crate::types::{error, format_error};

#[macro_use]
#[allow(dead_code)]
#[allow(unused_imports)]
mod types;
#[allow(dead_code)]
#[allow(unused_imports)]
mod reader;
#[allow(unused_variables)]
#[allow(unused_imports)]
mod printer;

pub type Env = FnvHashMap<String, MalVal>;

// read
fn read(str: &str) -> MalRet {
    reader::read_str(str.to_string())
}

// eval
fn eval_ast(ast: &MalVal, env: &Env) -> MalRet {
    // println!("  eval_ast {:?}", ast);
    match ast {
        Sym(s) => Ok(env
                    .get(s)
                    .ok_or(ErrString(format!("'{}' not found", s)))?
                    .clone()),
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

fn eval(ast: &MalVal, env: &Env) -> MalRet {
    // println!("Eval'ing {:?}", ast);
    match ast {
        // eval toplevel form
        List(v, _) => {
            if v.len() == 0 {
                return Ok(ast.clone())
            }
            // evaluate each list item individually
            let evaluated = eval_ast(ast, env)?;
            println!("Evaluated {:?}", evaluated);

            // now we should be able to apply the function
            // remember that in MAL, list are `(<fn> arg1 ... argN)`
            match evaluated {
                List(v, _) => {
                    let (fcall, fargs) = v.split_at(1);
                    match fcall {
                        [Func(f)] => (*f)(fargs.to_vec()),
                        _ => error(&format!("can't apply: {:?}", fcall)),
                    }
                    // Ok(Sym(format!("{:?} => {:?}", fcall, fargs)))
                },
                _ => error("expected a list"),
            }
        },
        rst => eval_ast(&rst, &env)
    }
}

// print
fn print(ast: &MalVal) -> String {
    ast.pr_str()
}

fn rep(str: &str, env: &Env) -> Result<String, MalErr> {
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
    let mut env = Env::default();
    env.insert("+".to_string(), Func(|args: MalArgs| op(args, |a,b| a+b)));
    env.insert("-".to_string(), Func(|args: MalArgs| op(args, |a,b| a-b)));
    env.insert("*".to_string(), Func(|args: MalArgs| op(args, |a,b| a*b)));
    env.insert("/".to_string(), Func(|args: MalArgs| op(args, |a,b| a/b)));

    loop {

        // read
        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".mal-history").unwrap();

                if line.len() > 0 {
                    let res = rep(&line, &env);
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
