extern crate regex;
extern crate rustyline;
extern crate fnv;
extern crate lazy_static;

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::types::{MalVal, MalRet};
use crate::types::format_error;

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

fn main() {
    // `()` can be used when no completer is required
    let mut rl = Editor::<()>::new();
    if rl.load_history(".mal-history").is_err() {
        eprintln!("No previous history.");
    }

    loop {

        let readline = rl.readline("user> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(&line);
                rl.save_history(".mal-history").unwrap();

                if line.len() > 0 {
                    match reader::read_str(line) {
                        Ok(mv) => {
                            println!("{}", mv.pr_str());
                        }
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
