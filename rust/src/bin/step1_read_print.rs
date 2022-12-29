extern crate mal_rust;
use mal_rust::reader::*;
use mal_rust::types::*;
use std::error;
use std::fs::File;
type Result<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}
fn eval(parsed_input: MalType) -> MalType {
    parsed_input
}

fn print(evaluated_input: MalType) -> String {
    evaluated_input.to_string()
}

fn rep(text: String) -> Result<String> {
    Ok(print(eval(read(text)?)))
}

fn main() {
    let mut rl = rustyline::Editor::<()>::new().unwrap();
    File::create("history.txt").unwrap();
    rl.load_history("history.txt").unwrap();
    loop {
        let input = match rl.readline("user> ") {
            Ok(i) => {
                rl.add_history_entry(i.as_str());
                i
            }
            Err(_) => break,
        };
        let result = rep(input);
        match result {
            Ok(a) => {
                println!("{}", a)
            }
            Err(e) => {
                println!(" {}", e)
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
