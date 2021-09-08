use std::rc::Rc;
use fnv::FnvHashMap;

use crate::types::{MalVal};
use crate::types::MalVal::{List, Vector, Hash, Nil, Str, Sym, Bool, Int, Atom, Keyword, Func};


impl MalVal {

    pub fn pr_str(&self) -> String {

        match self {
            Nil => String::from("nil"),
            Str(s) => String::from(format!("\"{}\"", s)),
            Sym(s) => String::from(s),
            Keyword(s) => String::from(format!(":{}", s)),
            Bool(b) => String::from(b.to_string()),
            Int(i) => String::from(i.to_string()),
            List(mvs, _) => format!("({})", print_seq(&**mvs)),
            Vector(mvs, _) => format!("[{}]", print_seq(&**mvs)),
            Hash(mvs, _) => {
                let kvs: Vec<MalVal> = mvs
                                        .iter()
                                        .flat_map(|(k, v)| { vec![k.mal_val(), v.clone()] } )
                                        .collect();
                format!("{{{}}}", print_seq(&kvs))
            },
            Func(f) => format!("<func {:?}", f),
            Atom() => String::from("@")
        }
    }
}

pub fn print_seq(v: &Vec<MalVal>) -> String {
    // println!("SEQ {:?}", v);
    v.iter()
     .map(|mv| mv.pr_str())
     .collect::<Vec<String>>()
     .join(" ")
    
}