extern crate mal_rust;
extern crate im_rc;

use mal_rust::env::Env;
use mal_rust::functions::{default_env, Functions};
use mal_rust::reader::*;
use mal_rust::types::EvalError::{SymbolNotFound, WrongArgAmount};
use mal_rust::types::*;
use std::error;
use std::fs::File;
use std::rc::Rc;
use im_rc::{HashMap, Vector};
use mal_rust::functions::Functions::NonNative;
use mal_rust::types::MalType::Nil;

type ResultBox<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}

fn eval(ast: MalType, env: &Env) -> EvalResult {
    match ast {
        MalType::List(list) => {
            match list.head() {
                None => Ok(list.into()),
                Some(head) => {
                    if let Ok(symbol) = head.clone().to_symbol() {
                        match symbol.as_str() {
                            "def!" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                let key = l
                                    .next()
                                    .ok_or(EvalError::WrongArgAmount)?
                                    .clone()
                                    .to_symbol()?;
                                let value =
                                    eval(l.next().ok_or(EvalError::WrongArgAmount)?.clone(), env)?;
                                env.set(key, value.clone());
                                Ok(value)
                            }
                            "let*" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                let new_env = env.new_env();
                                let mut first_parameter = l.next().ok_or(WrongArgAmount)?.clone().to_list()?.into_iter();
                                while let Some(k) = first_parameter.next()  {
                                    let key = k.clone().to_symbol()?;
                                    let value = eval(first_parameter.next().ok_or(WrongArgAmount)?.clone(), &new_env)?;
                                    new_env.set(key, value);
                                }
                                eval(l.next().ok_or(WrongArgAmount)?.clone(), &new_env)
                            }
                            "do" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                l.map(|m| eval_ast(m, env)).last().unwrap()
                            }
                            "if" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                let cond = eval(l.next().ok_or(WrongArgAmount)?, env)?.to_bool();
                                if cond{
                                    eval(l.next().ok_or(WrongArgAmount)?, env)
                                } else {
                                    l.next().ok_or(WrongArgAmount)?;
                                    match l.next() {
                                        None => {Ok(Nil)}
                                        Some(m) => {eval(m, env)}
                                    }
                                }
                            }
                            "fn*" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                let variables = l.next().ok_or(WrongArgAmount)?.to_list()?;
                                let body = l.next().ok_or(WrongArgAmount)?;
                                let env_copy = env.new_env();
                                Ok(MalType::Function(NonNative(Rc::new(
                                    move |m: Vector<MalType>| {
                                        env_copy.bind(variables.clone(), m)?;
                                        eval(body.clone(), &env_copy)
                                    }
                                ))))
                            }
                            _ => {
                                call_with_first_as_func(list, env)
                            }
                        }
                    }
                     else {
                         call_with_first_as_func(list, env)
                     }
                }
            }
        }
        ast => eval_ast(ast, env),
    }
}

fn call_with_first_as_func(list: Vector<MalType>, env: &Env) -> EvalResult{
    let mut new_list = eval_ast(list.into(), env)?.to_list().expect("should be a list");
    let first = new_list.pop_front().unwrap().to_function()?;
    first.call(new_list)
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
