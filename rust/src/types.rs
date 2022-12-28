use std::error;
use rpds::{HashTrieMap, List, Vector};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash};
use std::ops::{Deref, DerefMut};

type Sym = String;

pub type ParseResult = Result<MalType, ParseError>;
pub type EvalResult = Result<MalType, EvalError>;

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

pub enum EvalError {
    InvalidHashKey(MalType),
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

