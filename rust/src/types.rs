use std::error;
use rpds::{HashTrieMap, List, Vector};
use std::fmt::{Debug, Display, Formatter, write};
use std::hash::{Hash};
use std::ops::{Deref, DerefMut};
use functions::Functions;
use types::EvalError::InvalidType;

type Sym = String;

pub type ParseResult = Result<MalType, ParseError>;
pub type EvalResult = Result<MalType, EvalError>;
pub type MidResult<T> = Result<T, EvalError>;

#[derive(Debug, Clone, Copy)]
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

impl Eq for MalFloat {

}

impl From<f64> for MalFloat {
    fn from(f: f64) -> Self {
        MalFloat(f)
    }
}
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct HashTrieMapWrapper(pub HashTrieMap<MalType, MalType>);

impl Hash for HashTrieMapWrapper {
    fn hash<H: std::hash::Hasher>(&self, _state: &mut H) {
        panic!("should not be able to use Hash Map as a key since it is not hashable")
    }
}
impl Deref for HashTrieMapWrapper {
    type Target = HashTrieMap<MalType, MalType>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<HashTrieMap<MalType, MalType>> for HashTrieMapWrapper {
    fn from(h: HashTrieMap<MalType, MalType>) -> Self {
        HashTrieMapWrapper(h)
    }
}


impl DerefMut for HashTrieMapWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Eq, PartialEq, Hash, Debug, Clone)]
pub enum MalType {
    Nil,
    Bool(bool),
    Integer(i64),
    Float(MalFloat),
    List(List<MalType>),
    Symbol(Sym),
    String(List<char>),
    Vector(Vector<MalType>),
    HashMap(HashTrieMapWrapper),
    Function(Functions),
}

pub fn map_list<A: Clone, B, E>(mut list: List<A>, f: impl Fn(A) -> Result<B, E>) -> Result<List<B>, E> {
    match list.first() {
        None => {Ok(List::new())}
        Some(head) => {
            let tail = list.drop_first().unwrap();
            let new_head = f(head.clone())?;
            let new_tail = map_list(tail, f)?;
            Ok(new_tail.push_front(new_head))
        }
    }
}


impl From<Vector<MalType>> for MalType {
    fn from(v: Vector<MalType>) -> Self {
        MalType::Vector(v)
    }
}

impl From<List<MalType>> for MalType {
    fn from(l: List<MalType>) -> Self {
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
    MissingValue(MalType)
}

#[derive(Debug, Clone)]
pub enum EvalError {
    InvalidHashKey(MalType),
    InvalidType(String, String),
    WrongArgAmount,
    SymbolNotFound(String),
}


impl Display for EvalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::InvalidHashKey(m) => write!(f, "Invalid hash key: {}", m),
            InvalidType(expected, actual) => write!(f, "Expected {}, got {}", expected, actual),
            EvalError::WrongArgAmount => {write!(f, "Wrong number of arguments for the function")}
            EvalError::SymbolNotFound(s) => {write!(f, "Symbol not found: {}", s)}
        }
    }
}

impl error::Error for EvalError {

}

impl error::Error for ParseError{

}

impl Display for ParseError{
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
        }
    }
    pub fn to_symbol(self) -> MidResult<String>{
        match self {
            MalType::Symbol(s) => Ok(s),
            _ => Err(EvalError::InvalidType("symbol".to_string(), self.type_string()))
        }
    }
    pub fn to_integer(&self) -> MidResult<i64>{
        match self {
            MalType::Integer(i) => Ok(*i),
            _ => Err(EvalError::InvalidType("integer".to_string(), self.type_string()))
        }
    }
    pub fn coerce_to_integer(&self) -> MidResult<i64>{
        match self {
            MalType::Integer(i) => Ok(*i),
            MalType::Float(f) => Ok(f.0 as i64),
            _ => Err(EvalError::InvalidType("integer".to_string(), self.type_string()))
        }
    }
    pub fn to_float(&self) -> MidResult<f64>{
        match self {
            MalType::Float(f) => Ok(f.0),
            MalType::Integer(i) => Ok(*i as f64),
            _ => Err(EvalError::InvalidType("float".to_string(), self.type_string()))
        }
    }
    pub fn to_bool(&self) -> MidResult<bool>{
        match self {
            MalType::Bool(b) => Ok(*b),
            _ => Err(EvalError::InvalidType("bool".to_string(), self.type_string()))
        }
    }
    pub fn to_list(self) -> MidResult<List<MalType>>{
        match self {
            MalType::List(l) => Ok(l),
            _ => Err(EvalError::InvalidType("list".to_string(), self.type_string()))
        }
    }
    pub fn to_vector(self) -> MidResult<Vector<MalType>>{
        match self {
            MalType::Vector(v) => Ok(v),
            _ => Err(EvalError::InvalidType("vector".to_string(), self.type_string()))
        }
    }
    pub fn to_hash_map(self) -> MidResult<HashTrieMapWrapper>{
        match self {
            MalType::HashMap(h) => Ok(h),
            _ => Err(EvalError::InvalidType("hash-map".to_string(), self.type_string()))
        }
    }
    pub fn to_mal_string(self) -> MidResult<List<char>>{
        match self {
            MalType::String(s) => Ok(s),
            _ => Err(EvalError::InvalidType("string".to_string(), self.type_string()))
        }
    }
    pub fn to_function(self) -> MidResult<Functions>{
        match self {
            MalType::Function(f) => Ok(f),
            _ => Err(EvalError::InvalidType("function".to_string(), self.type_string()))
        }
    }
}
