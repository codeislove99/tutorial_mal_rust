extern crate env_logger;
extern crate im_rc;
extern crate log;
extern crate mal_rust;

use im_rc::{HashMap, Vector};
use log::warn;
use mal_rust::env::Env;
use mal_rust::functions::{default_env, default_env_non_native, InnerFunction};
use mal_rust::logger;
use mal_rust::reader::*;
use mal_rust::types::EvalError::{SymbolNotFound, WrongArgAmount};
use mal_rust::types::MalType::{List, Nil};
use mal_rust::types::*;
use std::error;
use std::fs::File;

type ResultBox<T> = std::result::Result<T, Box<dyn error::Error>>;
fn read(input: String) -> ParseResult {
    read_str(input)
}

fn eval(mut ast: MalType, mut env: Env) -> EvalResult {
    loop {
        ast = match ast {
            MalType::List(list) => match list.head() {
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
                                let value = eval(
                                    l.next().ok_or(EvalError::WrongArgAmount)?.clone(),
                                    env.clone(),
                                )?;
                                warn!("set {} to {}", key.clone(), value.clone());
                                env.set(key, value.clone());
                                return Ok(value);
                            }
                            "let*" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                env = env.new_env();
                                let mut first_parameter =
                                    l.next().ok_or(WrongArgAmount)?.to_list()?.into_iter();
                                while let Some(k) = first_parameter.next() {
                                    let key = k.clone().to_symbol()?;
                                    let value = eval(
                                        first_parameter.next().ok_or(WrongArgAmount)?.clone(),
                                        env.clone(),
                                    )?;
                                    env.set(key, value);
                                }
                                l.next().ok_or(WrongArgAmount)?
                            }
                            "do" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                while l.len() > 1 {
                                    eval(l.next().unwrap(), env.clone())?;
                                }
                                l.next().unwrap()
                            }
                            "if" => {
                                let mut l = list;
                                l.pop_front().unwrap();
                                let cond = eval(l.pop_front().ok_or(WrongArgAmount)?, env.clone())?
                                    .to_bool();
                                warn!(
                                    "(if {} {} {})",
                                    cond.clone(),
                                    l[0].clone(),
                                    l.get(1).unwrap_or(&Nil)
                                );
                                if cond {
                                    l.pop_front().ok_or(WrongArgAmount)?
                                } else {
                                    l.pop_front().ok_or(WrongArgAmount)?;
                                    match l.pop_front() {
                                        None => Nil,
                                        Some(m) => m,
                                    }
                                }
                            }
                            "fn*" => {
                                let mut l = list.into_iter();
                                l.next().unwrap();
                                let variables = l.next().ok_or(WrongArgAmount)?.to_list()?;
                                let body = l.next().ok_or(WrongArgAmount)?;
                                let env_copy = env.clone();
                                let fun = InnerFunction{
                                    ast: body,
                                    params: variables,
                                    env: env_copy,
                                };
                                return fun.into()
                            }
                            _ => {
                                let mut new_list = eval_ast(list.into(), &env)?
                                    .to_list()
                                    .expect("should be a list");
                                warn!("{}", MalType::List(new_list.clone()));
                                match new_list.pop_front().unwrap() {
                                    MalType::Function(f) => {
                                        return f.call(new_list)
                                    }
                                    MalType::NonNativeFunction(f) => {
                                        env = env.new_bind(f.params.clone(), new_list)?;
                                        f.ast.clone()
                                    }
                                    other => return Err(EvalError::InvalidType("Function".to_string(), other.type_string())),
                                }
                            }
                        }
                    } else {
                        let mut new_list = eval_ast(list.into(), &env)?
                            .to_list()
                            .expect("should be a list");
                        warn!("{}", MalType::List(new_list.clone()));
                        match new_list.pop_front().unwrap() {
                            MalType::Function(f) => {
                                return f.call(new_list)
                            }
                            MalType::NonNativeFunction(f) => {
                                env = f.env.new_bind(f.params.clone(), new_list)?;
                                f.ast.clone()
                            }
                            other => return Err(EvalError::InvalidType("Function".to_string(), other.type_string())),
                        }
                    }
                }
                None => return Ok(List(list)),
            },
            ast => return eval_ast(ast, &env),
        };
    }
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
                vec.push_back(eval(i, env.clone())?)
            }
            Ok(MalType::List(vec))
        }
        MalType::Vector(v) => {
            let mut vec = Vector::new();
            for i in v.into_iter() {
                vec.push_back(eval(i.clone(), env.clone())?);
            }
            Ok(MalType::Vector(vec))
        }
        MalType::HashMap(h) => {
            let mut map = HashMap::new();
            for (key, value) in h.into_iter() {
                let k = eval(key.clone(), env.clone())?;
                if !k.is_hashable() {
                    return Err(EvalError::InvalidHashKey(k));
                }
                let value = eval(value.clone(), env.clone())?;
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

fn rep(text: String, env: Env) -> ResultBox<String> {
    Ok(print(eval(read(text)?, env)?))
}

fn main() {
    println!("{}", "hello".to_string());
    let mut rl = rustyline::Editor::<()>::new().unwrap();
    let env = default_env();
    default_env_non_native().into_iter().for_each(|s| {
        rep(s, env.clone()).unwrap();
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
        let result = rep(input, env.clone());
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
