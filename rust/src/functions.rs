use env::Env;
use im_rc::Vector;
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::rc::Rc;
use types::EvalError::WrongArgAmount;
use types::{EvalError, EvalResult};
use MalType;
use MalType::{Bool, Float, Integer, Nil};

pub fn default_env_non_native() -> Vec<String> {
    let mut v = Vec::new();
    v.push("(def! not (fn* (a) (if a false true)))");
    v.iter().map(|s| s.to_string()).collect()
}
pub fn default_env() -> Env {
    let env = Env::new();

    let v: Vec<(&str, SimpleFn)> = vec![
        ("+", add),
        ("-", subtract),
        ("*", times),
        ("/", int_divide),
        ("=", equal),
        ("prn", prn),
        ("list", list),
        ("list?", is_list),
        ("empty?", is_empty),
        ("count", count),
        ("<", less_than),
        ("<=", less_than_or_equal),
        (">", greater_than),
        (">=", greater_than_or_equal),
        ("pr-str", pr_str),
        ("str", str),
        ("println", println),
    ];
    v.into_iter().for_each(|(k, f)| {
        env.set(k.into(), Functions::new_native(f));
    });
    env
}
type SimpleFn = fn(Vector<MalType>) -> EvalResult;

#[derive(Clone)]
pub enum Functions {
    Native(SimpleFn),
    NonNative(Rc<dyn Fn(Vector<MalType>) -> EvalResult>),
}

impl Debug for InnerFunction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} -> {}", self.params, self.ast)
    }
}

pub struct InnerFunction {
    pub ast: MalType,
    pub params: Vector<MalType>,
    pub env: Env,
}

impl Into<EvalResult> for InnerFunction{
    fn into(self) -> EvalResult {
        Ok(MalType::NonNativeFunction(Rc::new(self)))
    }
}

impl PartialOrd for InnerFunction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Hash for InnerFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        panic!("InnerFunction is not hashable and should never be able to be hashed");
    }
}

impl PartialEq<Self> for InnerFunction {
    fn eq(&self, other: &Self) -> bool {
        false
    }
}

impl Eq for InnerFunction {

}

impl PartialOrd for Functions {
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        None
    }
}

impl Debug for Functions {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "#<function>")
    }
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
    fn new_native(f: SimpleFn) -> MalType {
        MalType::Function(Functions::Native(f))
    }
}

impl Functions {
    pub fn call(&self, args: Vector<MalType>) -> EvalResult {
        match self {
            Functions::Native(f) => f(args),
            Functions::NonNative(f) => f(args),
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

pub fn list(args: Vector<MalType>) -> EvalResult {
    Ok(args.into())
}

pub fn is_list(args: Vector<MalType>) -> EvalResult {
    match args.front().ok_or(WrongArgAmount)? {
        MalType::List(_) => Ok(Bool(true)),
        _ => Ok(Bool(false)),
    }
}

pub fn is_empty(mut args: Vector<MalType>) -> EvalResult {
    Ok(Bool(
        args.pop_front().ok_or(WrongArgAmount)?.to_list()?.len() == 0,
    ))
}

pub fn count(mut args: Vector<MalType>) -> EvalResult {
    Ok(Integer(
        args.pop_front().ok_or(WrongArgAmount)?.to_list()?.len() as i64,
    ))
}

pub fn equal(mut args: Vector<MalType>) -> EvalResult {
    let first = args.pop_front().ok_or(WrongArgAmount)?;
    let second = args.pop_front().ok_or(WrongArgAmount)?;
    Ok(Bool(first == second))
}

fn get_first(args: &mut Vector<MalType>) -> Result<MalType, EvalError> {
    args.pop_front().ok_or(WrongArgAmount)
}

pub fn less_than(mut args: Vector<MalType>) -> EvalResult {
    let first = args.pop_front().ok_or(WrongArgAmount)?;
    let second = args.pop_front().ok_or(WrongArgAmount)?;
    Ok(Bool(first < second))
}

fn greater_than(mut args: Vector<MalType>) -> EvalResult {
    let first = get_first(&mut args)?;
    let second = get_first(&mut args)?;
    Ok(Bool(first > second))
}

fn greater_than_or_equal(mut args: Vector<MalType>) -> EvalResult {
    let first = get_first(&mut args)?;
    let second = get_first(&mut args)?;
    Ok(Bool(first >= second))
}

fn less_than_or_equal(mut args: Vector<MalType>) -> EvalResult {
    let first = get_first(&mut args)?;
    let second = get_first(&mut args)?;
    Ok(Bool(first <= second))
}

fn pr_str(args: Vector<MalType>) -> EvalResult {
    join(args, " ", false, true)
}

fn str(args: Vector<MalType>) -> EvalResult {
    join(args, "", false, false)
}

fn prn(args: Vector<MalType>) -> EvalResult {
    join(args, " ", true, true)
}

fn println(args: Vector<MalType>) -> EvalResult {
    join(args, " ", true, false)
}

fn join(args: Vector<MalType>, sep: &str, print: bool, readably: bool) -> EvalResult {
    let s = args
        .into_iter()
        .map(|x| x.pr_str(readably))
        .collect::<Vec<String>>()
        .join(sep);
    if print {
        println!("{}", s);
        Ok(Nil)
    } else {
        Ok(s.into())
    }
}
