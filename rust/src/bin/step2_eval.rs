extern crate mal_rust;
extern crate rpds;

use mal_rust::reader::*;
use mal_rust::types::*;
use std::{error};
use std::fs::File;
use std::iter::FromIterator;
use rpds::{HashTrieMap, List, Vector};
use mal_rust::env::Env;
use mal_rust::functions::default_env;
use mal_rust::types::EvalError::SymbolNotFound;

type ResultBox<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}
fn eval(ast: MalType, env: &Env) -> EvalResult {
    match &ast {
        MalType::List(list) => {
            match list.first() {
                None => {Ok(ast)}
                Some(head) => {
                    let new_list = eval_ast(ast, env)?.to_list().expect("should be a list");
                    let first = new_list.first().unwrap().clone().to_function()?;
                    first.call(new_list.drop_first().unwrap())
                }
            }
        }
        _ => {eval_ast(ast, env)}
    }
}
fn eval_ast(ast: MalType, env: &Env) -> EvalResult {
    match ast {
        MalType::Symbol(s) => env.get(&s).ok_or(SymbolNotFound(s)),
        MalType::List(l) => {
            Ok(MalType::List(map_list(l, |x| eval(x, env))?))
            }
        MalType::Vector(v) => {
            let mut vec = Vec::new();
            for i in v.into_iter(){
                vec.push(eval(i.clone(), env)?);
            }
            Ok(MalType::Vector(Vector::from_iter(vec)))
        }
        MalType::HashMap(h) => {
            let mut map = HashTrieMap::new();
            for (key, value) in h.into_iter(){
                let k = eval(key.clone(), env)?;
                if !k.is_hashable(){
                    return Err(EvalError::InvalidHashKey(k));
                }
                let value = eval(value.clone(), env)?;
                map.insert_mut(k, value);
            }
            Ok(MalType::HashMap(map.into()))

        }
        other => {Ok(other)}
    }
}

fn print(evaluated_input: MalType) -> String {
    evaluated_input.to_string()
}

fn rep(text: String) -> ResultBox<String> {
    let env = default_env();
    Ok(print(eval(read(text)?, &env)?))
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
            Err(_) => {break}
        };
        let result = rep(input);
        match result {
            Ok(a) => {println!("{}", a)}
            Err(e) => {println!("{}", e)}
        }
    }
    rl.save_history("history.txt").unwrap();
}
