extern crate rustyline;

use rustyline::error::ReadlineError;
use rustyline::Editor;

fn read(input: String) -> String {
    return input
}


fn eval(ast: String) -> String {
    return ast;
}

fn print(eval: String) {
    if eval.len() > 0 {
        println!("{}", eval);
    }
}

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
                
                let s = read(line);
                let e = eval(s);
                
                print(e);
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
