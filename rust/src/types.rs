use std::cell::RefCell;
use functions::{Functions, InnerFunction};
use im_rc::{HashMap, Vector};
use std::error;
use std::fmt::{Debug, Display, Formatter, write};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::rc::Rc;
use types::EvalError::{InvalidType};

type Sym = String;

pub type ParseResult = Result<MalType, ParseError>;
pub type EvalResult = Result<MalType, EvalError>;
pub type MidResult<T> = Result<T, EvalError>;

#[derive(Debug, Clone, Copy, PartialOrd)]
pub struct MalFloat(pub f64);

impl Deref for MalFloat {
    type Target = f64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for MalFloat {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let i = self.0 as i64;
        i.hash(state);
    }
}

impl PartialEq<Self> for MalFloat {
    fn eq(&self, other: &Self) -> bool {
        self.0 as i64 == other.0 as i64
    }
}

impl Eq for MalFloat {}

impl From<f64> for MalFloat {
    fn from(f: f64) -> Self {
        MalFloat(f)
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone, PartialOrd)]
pub enum MalType {
    Nil,
    Bool(bool),
    Integer(i64),
    Float(MalFloat),
    List(Vector<MalType>),
    Symbol(Sym),
    String(Vector<char>),
    Vector(Vector<MalType>),
    HashMap(HashMap<MalType, MalType>),
    Function(Functions),
    NonNativeFunction(Rc<InnerFunction>),
    Atom(Atom)
}

#[derive(Debug, Eq, PartialEq, Clone, PartialOrd)]
pub struct Atom(pub Rc<RefCell<MalType>>);

impl Atom {
    pub fn get_value(self) -> MalType{
        self.0.borrow().clone()
    }
}

impl Hash for Atom{
    fn hash<H: Hasher>(&self, state: &mut H) {
        panic!("cannot hash an atom")
    }
}



impl From<Rc<dyn Fn(Vector<MalType>) -> EvalResult>> for MalType {
    fn from(f: Rc<dyn Fn(Vector<MalType>) -> EvalResult>) -> Self {
        MalType::Function(Functions::NonNative(f))
    }
}

impl From<String> for MalType {
    fn from(s: String) -> Self {
        MalType::String(s.chars().collect())
    }
}

impl From<Vector<MalType>> for MalType {
    fn from(l: Vector<MalType>) -> Self {
        MalType::List(l)
    }
}

impl MalType {
    pub fn as_key(self) -> EvalResult {
        match &self {
            MalType::HashMap(_) => Err(EvalError::InvalidHashKey(self)),
            _ => Ok(self),
        }
    }
}
impl Into<ParseResult> for MalType {
    fn into(self) -> ParseResult {
        Ok(self)
    }
}

#[derive(Debug, Clone)]
pub enum ParseError {
    NoClosingParen(char),
    InvalidNum(String),
    MissingValue(MalType),
}

#[derive(Debug, Clone)]
pub enum EvalError {
    InvalidHashKey(MalType),
    InvalidType(String, String),
    WrongArgAmount,
    SymbolNotFound(String),
    ParseError(ParseError),
    InvalidFile(String)
}

impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::InvalidHashKey(m) => write!(f, "Invalid hash key: {}", m),
            InvalidType(expected, actual) => write!(f, "Expected {}, got {}", expected, actual),
            EvalError::WrongArgAmount => {
                write!(f, "Wrong number of arguments for the function")
            }
            EvalError::SymbolNotFound(s) => {
                write!(f, "Symbol not found: {}", s)
            }
            EvalError::ParseError(p) => {
                write!(f, "failed at parsing {}", p)
            }
            EvalError::InvalidFile(s) => {
                write!(f, "failed at loading file: {}", s)
            }
        }
    }
}

impl From<ParseError> for EvalError {
    fn from(p: ParseError) -> Self {
        EvalError::ParseError(p)
    }
}
impl error::Error for EvalError {}

impl error::Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let intro_string = "you had the following ParseError: ";
        let error_string = match self {
            ParseError::NoClosingParen(c) => {
                format!("No closing paren {}", c)
            }
            ParseError::InvalidNum(n) => {
                format!("{} is not a valid number", n)
            }
            ParseError::MissingValue(m) => {
                format!("Missing value for {}", m)
            }
        };
        write!(f, "{}{}", intro_string, error_string)
    }
}

impl MalType {
    pub fn type_string(&self) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(_) => "bool".to_string(),
            MalType::Integer(_) => "int".to_string(),
            MalType::Float(_) => "float".to_string(),
            MalType::List(_) => "list".to_string(),
            MalType::Symbol(_) => "symbol".to_string(),
            MalType::String(_) => "string".to_string(),
            MalType::Vector(_) => "vector".to_string(),
            MalType::HashMap(_) => "hash-map".to_string(),
            MalType::Function(_) => "function".to_string(),
            MalType::NonNativeFunction(_) => "function".to_string(),
            MalType::Atom(_) => {"atom".to_string()}
        }
    }
    pub fn to_atom_value(self) -> Atom{
        Atom(Rc::new(RefCell::new(self)))
    }
    pub fn to_atom_mal_type(self) -> Self{
        MalType::Atom(self.to_atom_value())
    }
    pub fn to_symbol(self) -> MidResult<String> {
        match self {
            MalType::Symbol(s) => Ok(s),
            _ => Err(EvalError::InvalidType(
                "symbol".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_integer(&self) -> MidResult<i64> {
        match self {
            MalType::Integer(i) => Ok(*i),
            _ => Err(EvalError::InvalidType(
                "integer".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn coerce_to_integer(&self) -> MidResult<i64> {
        match self {
            MalType::Integer(i) => Ok(*i),
            MalType::Float(f) => Ok(f.0 as i64),
            _ => Err(EvalError::InvalidType(
                "integer".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_float(&self) -> MidResult<f64> {
        match self {
            MalType::Float(f) => Ok(f.0),
            MalType::Integer(i) => Ok(*i as f64),
            _ => Err(EvalError::InvalidType(
                "float".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_bool(&self) -> bool {
        match self {
            MalType::Bool(b) => *b,
            MalType::Nil => false,
            _ => true,
        }
    }
    pub fn to_list(self) -> MidResult<Vector<MalType>> {
        match self {
            MalType::List(l) => Ok(l),
            MalType::Vector(l) => Ok(l),
            _ => Err(EvalError::InvalidType(
                "list".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_vector(self) -> MidResult<Vector<MalType>> {
        match self {
            MalType::Vector(v) => Ok(v),
            _ => Err(EvalError::InvalidType(
                "vector".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_hash_map(self) -> MidResult<HashMap<MalType, MalType>> {
        match self {
            MalType::HashMap(h) => Ok(h),
            _ => Err(EvalError::InvalidType(
                "hash-map".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_mal_string(self) -> MidResult<Vector<char>> {
        match self {
            MalType::String(s) => Ok(s),
            _ => Err(EvalError::InvalidType(
                "string".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn to_function(self) -> MidResult<Functions> {
        match self {
            MalType::Function(f) => Ok(f),
            _ => Err(EvalError::InvalidType(
                "function".to_string(),
                self.type_string(),
            )),
        }
    }
    pub fn is_hashable(&self) -> bool {
        match self {
            MalType::Float(_) => false,
            MalType::HashMap(_) => false,
            _ => true,
        }
    }
    pub fn to_real_str(self) -> MidResult<String>{
        Ok(self.to_mal_string()?.into_iter().collect::<String>())
    }

    pub fn to_atom_inner(self) -> EvalResult{
        match self {
            MalType::Atom(m) => Ok(m.get_value()),
            _ => Err(EvalError::InvalidType("Atom".to_string(), self.type_string()))
        }
    }
    pub fn to_atom(self) -> MidResult<Atom>{
        match self {
            MalType::Atom(m) => Ok(m),
            _ => Err(EvalError::InvalidType("Atom".to_string(), self.type_string()))
        }
    }
}

