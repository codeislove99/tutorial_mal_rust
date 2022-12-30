extern crate im_rc;
extern crate mal_rust;

use im_rc::{HashMap, Vector};
use mal_rust::env::Env;
use mal_rust::functions::default_env;
use mal_rust::reader::*;
use mal_rust::types::EvalError::SymbolNotFound;
use mal_rust::types::MalType::Nil;
use mal_rust::types::*;
use std::error;
use std::fs::File;
use std::iter::FromIterator;

type ResultBox<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}
fn eval(ast: MalType, env: &Env) -> EvalResult {
    match &ast {
        MalType::List(list) => {
            let mut l = list.iter();
            match l.next() {
                None => Ok(ast),
                Some(head) => {
                    if let Ok(symbol) = head.clone().to_symbol() {
                        match symbol.as_str() {
                            "def!" => {
                                let mut l = list.iter();
                                l.next().expect("should have a value");
                                let key = l
                                    .next()
                                    .ok_or(EvalError::WrongArgAmount)?
                                    .clone()
                                    .to_symbol()?;
                                let value =
                                    eval(l.next().ok_or(EvalError::WrongArgAmount)?.clone(), env)?;
                                env.set(key, value.clone());
                                return Ok(Nil);
                            }
                            _ => {}
                        }
                    }
                    let mut new_list = eval_ast(ast, env)?.to_list().expect("should be a list");
                    let first = new_list.pop_front().unwrap().to_function()?;
                    first.call(new_list)
                }
            }
        }
        _ => eval_ast(ast, env),
    }
}
fn eval_ast(ast: MalType, env: &Env) -> EvalResult {
    match ast {
        MalType::Symbol(s) => env.get(&s).ok_or(SymbolNotFound(s)),
        MalType::List(l) => {
            let mut vec = Vector::new();
            for i in l.into_iter() {
                vec.push_back(eval(i, env)?)
            }
            Ok(MalType::List(vec))
        }
        MalType::Vector(v) => {
            let mut vec = Vector::new();
            for i in v.into_iter() {
                vec.push_back(eval(i.clone(), env)?);
            }
            Ok(MalType::Vector(vec))
        }
        MalType::HashMap(h) => {
            let mut map = HashMap::new();
            for (key, value) in h.into_iter() {
                let k = eval(key.clone(), env)?;
                if !k.is_hashable() {
                    return Err(EvalError::InvalidHashKey(k));
                }
                let value = eval(value.clone(), env)?;
                map.insert(k, value);
            }
            Ok(MalType::HashMap(map.into()))
        }
        other => Ok(other),
    }
}

fn print(evaluated_input: MalType) -> String {
    evaluated_input.to_string()
}

fn rep(text: String, env: &Env) -> ResultBox<String> {
    Ok(print(eval(read(text)?, env)?))
}

fn main() {
    let mut rl = rustyline::Editor::<()>::new().unwrap();
    let env = default_env();
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
        let result = rep(input, &env);
        match result {
            Ok(a) => {
                println!("{}", a)
            }
            Err(e) => {
                println!("{}", e)
            }
        }
    }
    rl.save_history("history.txt").unwrap();
}
