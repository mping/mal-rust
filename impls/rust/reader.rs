use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::rc::Rc;

use crate::types::MalErr::ErrString;
use crate::types::{MalErr, MalVal, MalRet, error, hash_map};
use crate::types::MalVal::{List, Vector, Nil, Str, Int, Bool, Keyword, Sym};


pub struct Reader {
    tokens: Vec<String>,
    position: usize
}

impl Reader {
    fn get(&self, i: usize)-> Result<String, MalErr>  {
        return Ok(self
            .tokens
            .get(i)
            .ok_or(ErrString("reader: end of input".to_string()))?
            .to_string());
    }

    fn next(&mut self)-> Result<String, MalErr> {
        self.position = self.position + 1;
        return self.get(self.position - 1);
    }
    fn peek(&mut self) -> Result<String, MalErr> {
        return self.get(self.position);
    }
}

fn tokenize(s: &String) -> Vec<String>{
    lazy_static! {
        static ref RE: Regex = Regex::new(r###"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"###).unwrap();
    }
    let mut res = vec![];
    for cap in RE.captures_iter(s) {
        // regex is line based, we just ignore comments
        if cap[1].starts_with(";") {
            continue;
        }
        res.push(String::from(&cap[1]))
    }
    return res;
}


pub fn read_str(s: String)-> MalRet {
    let tokens = tokenize(&s);
    if tokens.len() == 0 {
        return error("no input");
    }
    read_form(&mut Reader {
        position:0, tokens: tokens
    })
}

fn read_list(r: &mut Reader) -> MalRet {
    let start = r.next()?;
    let stop = match &start[..] {
        "(" => ")",
        "[" => "]",
        "{" => "}",
        _ => panic!("reader: cannot read list starting with '{}'", start)
    };

    let mut seq: Vec<MalVal> = vec![];
    let finish: String;
    loop {
        let n = r.peek()?;
        if n == stop {
            finish = n;
            break;
        }
        let mv = read_form(r)?;
        seq.push(mv);
    }
    let _ = r.next();
    match &stop[..] {
        ")" => Ok(list!(seq)),
        "]" => Ok(vector!(seq)),
        "}" => {
            if seq.len() % 2 != 0 {
                error("reader: hashmap is unbalanced")
            } else {
                hash_map(seq)
            }
        },
        _ => panic!("reader: unknown finish symbol; '{}'", finish)
    }
}

fn read_atom(r: &mut Reader) -> MalRet {
    lazy_static! {
        static ref INT: Regex = Regex::new(r"^-?[0-9]+$").unwrap();
        static ref STR: Regex = Regex::new(r#""(?:\\.|[^\\"])*""#).unwrap();
        static ref ID: Regex = Regex::new(r###"\d+"###).unwrap();
    }
    let token = r.next()?;
    match &token[..] {
        "nil" =>  Ok(Nil),
        "true" => Ok(Bool(true)),
        "false" => Ok(Bool(false)),
        _ => {
            if INT.is_match(&token) {
                Ok(Int(token.parse().unwrap()))
            } else if STR.is_match(&token) {
                Ok(Str(token[1..token.len()-1].to_string()))
            } else if ID.is_match(&token) {
                Ok(Sym(token.parse().unwrap()))
            } else if token.starts_with(":") {
                Ok(Keyword(token[1..].to_string()))
            } else if token.starts_with("\"") {
                error("expected '\"', got EOF")
            } else {
                Ok(Sym(token.parse().unwrap()))
                // Err(ErrString(format!("reader: Unknown token: '{}'", &token[..])))
            }
        }
    }
}

fn read_form(r: &mut Reader) -> MalRet {
    let token = r.peek()?;
    match &token[..] {
        "(" => { read_list(r) }
        ")" => { error("reader: Unexpected character: ')'") }

        "[" => { read_list(r)}
        "]" => { error("reader: Unexpected character: ']'") }

        "{" => { read_list(r) }
        "}" => { error("reader: Unexpected character: '}'") }

        "~@" => {
            let _ = r.next();
            Ok(list![Sym("splice-unquote".to_string()), read_form(r)?])
        }
        "~" => {
            let _ = r.next();
            Ok(list![Sym("unquote".to_string()), read_form(r)?])
        }
        "@" => {
            let _ = r.next();
            Ok(list![Sym("deref".to_string()), read_form(r)?])
        }
        "'" => {
            let _ = r.next();
            Ok(list![Sym("quote".to_string()), read_form(r)?])
        }
        "`" => {
            let _ = r.next();
            Ok(list![Sym("quasiquote".to_string()), read_form(r)?])
        }
        "^" => {
            let _ = r.next();
            let meta = read_form(r)?;
            Ok(list![Sym("with-meta".to_string()), read_form(r)?, meta])
        }

        _ => {read_atom(r)}
    }
}
