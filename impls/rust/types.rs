use std::rc::Rc;
use fnv::FnvHashMap;
use std::fmt;
//use std::collections::HashMap;

use crate::types::MalErr::{ErrMalVal, ErrString};
use crate::types::MalVal::{List, Vector, Hash, Nil, Str, Sym, Bool, Int, Atom, Keyword};
use crate::types::MapKey::{Ks, Kw};


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MapKey {
  Ks(String),
  Kw(String)
}

impl fmt::Display for MapKey {
  // This trait requires `fmt` with this exact signature.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Ks(s) => write!(f, "{}", s),
      Kw(s) => write!(f, ":{}", s)
    }
  }
}

impl MapKey {
  pub fn mal_val(&self) -> MalVal {
    match self {
      Ks(s) => Str(s.to_string()),
      Kw(s) => Keyword(s.to_string())
    }
  }
}

#[derive(Debug, Clone)]
pub enum MalVal {
    Nil,
    Str(String),
    Bool(bool),
    Int(i64),
    Sym(String),
    Keyword(String),
    List(Rc<Vec<MalVal>>, Rc<MalVal>),
    Vector(Rc<Vec<MalVal>>, Rc<MalVal>),
    Hash(Rc<FnvHashMap<MapKey, MalVal>>, Rc<MalVal>),
    Atom(),
}



impl fmt::Display for MalVal {
  // This trait requires `fmt` with this exact signature.
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
      // Write strictly the first element into the supplied output
      // stream: `f`. Returns `fmt::Result` which indicates whether the
      // operation succeeded or failed. Note that `write!` uses syntax which
      // is very similar to `println!`.
      match self {
        Nil => write!(f, "{}", "Nil"),
        Str(s) => write!(f, "{}", s),
        Sym(s) => write!(f, "{}", s),
        Keyword(s) => write!(f, "{}", s),
        Bool(b) => write!(f, "{}", b),
        Int(i) => write!(f, "{}", i),
        List(mvs, _) => write!(f, "{}", "List"),
        Vector(mvs, _) => write!(f, "{}", "Vector"),
        Hash(mvs, _) => write!(f, "{}", "Hash"),
        Atom() => write!(f, "{}", "Atom")
      }
  }
}

#[derive(Debug)]
pub enum MalErr {
    ErrString(String),
    ErrMalVal(MalVal),
}

pub type MalArgs = Vec<MalVal>;
pub type MalRet = Result<MalVal, MalErr>;

pub fn error(s: &str) -> MalRet {
    Err(ErrString(s.to_string()))
}

pub fn format_error(e: MalErr) -> String {
    match e {
        ErrString(s) => s.clone(),
        ErrMalVal(mv) => mv.pr_str(),
    }
}

// type utility macros
  
macro_rules! list {
  ($seq:expr) => {{
    List(Rc::new($seq),Rc::new(Nil))
  }};
  [$($args:expr),*] => {{
    let v: Vec<MalVal> = vec![$($args),*];
    List(Rc::new(v),Rc::new(Nil))
  }}
}

macro_rules! vector {
  ($seq:expr) => {{
    Vector(Rc::new($seq),Rc::new(Nil))
  }};
  [$($args:expr),*] => {{
    let v: Vec<MalVal> = vec![$($args),*];
    Vector(Rc::new(v),Rc::new(Nil))
  }}
}

pub fn hash_map(kvs: MalArgs) -> MalRet {
  let mut hm: FnvHashMap<MapKey, MalVal> = FnvHashMap::default();
  for pair in kvs.chunks(2) {
    let mvk = &pair[0];
    let mvv = &pair[1];
    match mvk {
      Str(s)     => { hm.insert(Ks(s.to_string()), mvv.clone()); },
      Keyword(s) => { hm.insert(Kw(s.to_string()), mvv.clone()); },
      _          => {
        let s = format!("hashmap: key is not a string nor a keyword: '{}'", mvk); 
        return error(&s); 
      }
    }
  }

  Ok(Hash(Rc::new(hm), Rc::new(Nil)))
}
