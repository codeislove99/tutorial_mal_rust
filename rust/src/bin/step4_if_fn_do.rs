extern crate env_logger;
extern crate im_rc;
extern crate log;
extern crate mal_rust;

use im_rc::{HashMap, Vector};
use log::warn;
use mal_rust::env::Env;
use mal_rust::functions::Functions::NonNative;
use mal_rust::functions::{default_env, default_env_non_native};
use mal_rust::logger;
use mal_rust::reader::*;
use mal_rust::types::EvalError::{SymbolNotFound, WrongArgAmount};
use mal_rust::types::MalType::Nil;
use mal_rust::types::*;
use std::error;
use std::fs::File;
use std::rc::Rc;

type ResultBox<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}

fn eval(ast: MalType, env: &Env) -> EvalResult {
    let ast: MalType = match ast {
        MalType::List(list) => match list.head() {
            None => list.into(),
            Some(head) => {
                warn!("{}", MalType::List(list.clone()));
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
                            warn!("set {} to {}", key.clone(), value.clone());
                            env.set(key, value.clone());
                            value
                        }
                        "let*" => {
                            let mut l = list.into_iter();
                            l.next().unwrap();
                            let new_env = env.new_env();
                            let mut first_parameter = l
                                .next()
                                .ok_or(WrongArgAmount)?
                                .clone()
                                .to_list()?
                                .into_iter();
                            while let Some(k) = first_parameter.next() {
                                let key = k.clone().to_symbol()?;
                                let value = eval(
                                    first_parameter.next().ok_or(WrongArgAmount)?.clone(),
                                    &new_env,
                                )?;
                                new_env.set(key, value);
                            }
                            eval(l.next().ok_or(WrongArgAmount)?.clone(), &new_env)?
                        }
                        "do" => {
                            let mut l = list.into_iter();
                            l.next().unwrap();
                            while l.len() > 1 {
                                eval(l.next().unwrap(), env)?;
                            }
                            eval(l.next().unwrap(), env)?
                        }
                        "if" => {
                            let mut l = list;
                            l.pop_front().unwrap();
                            let cond = eval(l.pop_front().ok_or(WrongArgAmount)?, env)?.to_bool();
                            warn!(
                                "(if {} {} {})",
                                cond.clone(),
                                l[0].clone(),
                                l.get(1).unwrap_or(&Nil)
                            );
                            let result = if cond {
                                eval(l.pop_front().ok_or(WrongArgAmount)?, env)
                            } else {
                                l.pop_front().ok_or(WrongArgAmount)?;
                                match l.pop_front() {
                                    None => Ok(Nil),
                                    Some(m) => eval(m, env),
                                }
                            };
                            result?
                        }
                        "fn*" => {
                            let mut l = list.into_iter();
                            l.next().unwrap();
                            let variables = l.next().ok_or(WrongArgAmount)?.to_list()?;
                            let body = l.next().ok_or(WrongArgAmount)?;
                            let env_copy = env.clone();
                            MalType::Function(NonNative(Rc::new(move |m: Vector<MalType>| {
                                let env = env_copy.new_bind(variables.clone(), m)?;
                                eval(body.clone(), &env)
                            })))
                        }
                        _ => call_with_first_as_func(list, env)?,
                    }
                } else {
                    call_with_first_as_func(list, env)?
                }
            }
        },
        ast => eval_ast(ast, env)?,
    };
    Ok(ast)
}

fn call_with_first_as_func(list: Vector<MalType>, env: &Env) -> EvalResult {
    let mut new_list = eval_ast(list.into(), env)?
        .to_list()
        .expect("should be a list");
    warn!("{}", MalType::List(new_list.clone()));
    let first = new_list.pop_front().unwrap().to_function()?;
    let result = first.call(new_list);
    warn!("{}", result.clone()?);
    result
}
fn eval_ast(ast: MalType, env: &Env) -> EvalResult {
    let result = match ast {
        MalType::Symbol(s) => {
            let result = env.get(&s).ok_or(SymbolNotFound(s.clone()))?;
            match &result {
                MalType::Function(_) => {}
                _ => warn!("{} -> {}", s.clone(), result.clone()),
            }
            Ok(result)
        }
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
    };
    result
}

fn print(evaluated_input: MalType) -> String {
    evaluated_input.pr_str(true)
}

fn rep(text: String, env: &Env) -> ResultBox<String> {
    Ok(print(eval(read(text)?, env)?))
}

fn main() {
    println!("{}", "hello".to_string());
    let mut rl = rustyline::Editor::<()>::new().unwrap();
    let env = default_env();
    default_env_non_native().into_iter().for_each(|s| {
        rep(s, &env).unwrap();
    });
    File::create("history.txt").unwrap();
    rl.load_history("history.txt").unwrap();
    logger::init().unwrap();
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
