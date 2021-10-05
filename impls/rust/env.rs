use std::rc::Rc;
use fnv::FnvHashMap;
use std::fmt;

use crate::types::{MalVal, MalArgs, MalRet, MalErr, MapKey};
use crate::types::MapKey::{Ks, Kw};
use crate::types::MalVal::{List, Vector, Hash, Nil, Str, Sym, Bool, Int, Atom, Keyword, Func};
use crate::types::MalErr::ErrString;

#[derive(Debug)]
pub struct Env {
    data: FnvHashMap<String, MalVal>,
    outer: Option<Box<Env>>
}


pub fn make_env(outer: Option<Env>) -> Env {
    Env {
        data: FnvHashMap::default(),
        outer: match outer {
            Some(o) => Some(Box::new(o)),
            None    => None
        },
    }
}


impl Env {
    pub fn set(&mut self, sym: String, val: MalVal) -> MalRet {
        // Why dup?
        let dup = val.clone();
        self.data.insert(sym, val);
        Ok(dup)
    }

    pub fn find(&self, sym: String) -> MalRet {
        let mk = sym.to_string();
        match self.data.get(&mk) {
            Some(mr) => Ok(mr.clone()), // TODO why clone?
            None => match &self.outer {
                Some(o) => o.find(sym),
                // unknow symbols evalutate to themselves, for builtins like def! and let*
                None => Ok(Sym(sym)) 
            }
        }
    }

    pub fn get(&self, sym: String) -> MalRet {
      self.find(sym)
  }
}