use std::hash::Hash;
use lazy_static::lazy_static;
use rpds::List;
use MalType;
use MalType::{Float, HashMap, Integer};
use types::EvalError::WrongArgAmount;
use types::{EvalResult, MalFloat};
use env::Env;
use functions::Functions::Native;

pub fn default_env() -> Env {
        let env = Env::new();
        env.set("+".into(), Functions::new(add));
        env.set("-".into(), Functions::new(subtract));
        env.set("*".into(), Functions::new(times));
        env.set("/".into(), Functions::new(int_divide));
        env
}
type simple_fn = fn(List<MalType>) -> EvalResult;
#[derive(Clone, Debug)]
pub enum Functions{
    Native(simple_fn),
}

impl PartialEq<Self> for Functions {
    fn eq(&self, other: &Self) -> bool {
        return false;
    }
}

impl Hash for Functions {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        panic!("should not be able to use function as a key since it is not hashable")
    }
}

impl Eq for Functions {

}


impl Functions {
    fn new(f: simple_fn) -> MalType{
        MalType::Function(Functions::Native(f))
    }
}


impl Functions {
    pub fn call(&self, args: List<MalType>) -> EvalResult {
        match self {
            Functions::Native(f) => f(args),
        }
    }
}

pub fn add(args: List<MalType>) -> EvalResult {
    let mut result = Integer(0);
    for arg in args.into_iter() {
        result = match (result.clone(), arg.clone()) {
            (Integer(a), Integer(b)) => Integer(a + b),
            _ => Float((result.clone().to_float()? + arg.clone().to_float()?).into()),
        }
    }
    Ok(result)
}

pub fn subtract(args: List<MalType>) -> EvalResult {
    let mut result  = args.first().ok_or(WrongArgAmount)?.clone();
    let rest = args.drop_first().unwrap();

    for arg in rest.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a - b),
            _ => Float((result.to_float()? - arg.to_float()?).into()),
        };
    }
    Ok(result)
}
pub fn times(args: List<MalType>) -> EvalResult {
    let mut result = args.first().ok_or(WrongArgAmount)?.clone();
    let rest = args.drop_first().unwrap();
    for arg in rest.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a * b),
            _ => Float((result.to_float()? * arg.to_float()?).into()),
        };
    }
    Ok(result)
}
pub fn int_divide(args: List<MalType>) -> EvalResult {
    let mut result = args.first().ok_or(WrongArgAmount)?.clone();
    let rest = args.drop_first().unwrap();
    for arg in rest.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a / b),
            _ => Integer((result.to_float()? / arg.to_float()?) as i64),
        };
    }
    Ok(result)
}
