use env::Env;
use std::hash::Hash;
use im_rc::Vector;
use types::EvalError::WrongArgAmount;
use types::EvalResult;
use MalType;
use MalType::{Float, Integer};

pub fn default_env() -> Env {
    let env = Env::new();
    env.set("+".into(), Functions::new(add));
    env.set("-".into(), Functions::new(subtract));
    env.set("*".into(), Functions::new(times));
    env.set("/".into(), Functions::new(int_divide));
    env
}
type SimpleFn = fn(Vector<MalType>) -> EvalResult;
#[derive(Clone, Debug)]
pub enum Functions {
    Native(SimpleFn),
}

impl PartialEq<Self> for Functions {
    fn eq(&self, _other: &Self) -> bool {
        return false;
    }
}

impl Hash for Functions {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        panic!("should not be able to use function as a key since it is not hashable")
    }
}

impl Eq for Functions {}

impl Functions {
    fn new(f: SimpleFn) -> MalType {
        MalType::Function(Functions::Native(f))
    }
}

impl Functions {
    pub fn call(&self, args: Vector<MalType>) -> EvalResult {
        match self {
            Functions::Native(f) => f(args),
        }
    }
}

pub fn add(args: Vector<MalType>) -> EvalResult {
    let mut result = Integer(0);
    for arg in args.into_iter() {
        result = match (result.clone(), arg.clone()) {
            (Integer(a), Integer(b)) => Integer(a + b),
            _ => Float((result.clone().to_float()? + arg.clone().to_float()?).into()),
        }
    }
    Ok(result)
}

pub fn subtract(mut args: Vector<MalType>) -> EvalResult {
    let mut result = args.pop_front().ok_or(WrongArgAmount)?.clone();

    for arg in args.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a - b),
            _ => Float((result.to_float()? - arg.to_float()?).into()),
        };
    }
    Ok(result)
}
pub fn times(mut args: Vector<MalType>) -> EvalResult {
    let mut result = args.pop_front().ok_or(WrongArgAmount)?;
    for arg in args.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a * b),
            _ => Float((result.to_float()? * arg.to_float()?).into()),
        };
    }
    Ok(result)
}
pub fn int_divide(mut args: Vector<MalType>) -> EvalResult {
    let mut result = args.pop_front().ok_or(WrongArgAmount)?.clone();

    for arg in args.into_iter() {
        result = match (&result, &arg) {
            (Integer(a), Integer(b)) => Integer(a / b),
            _ => Integer((result.to_float()? / arg.to_float()?) as i64),
        };
    }
    Ok(result)
}