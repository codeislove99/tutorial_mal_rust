use regex::{Regex};
use rpds::{HashTrieMap, List, Vector};
use std::iter::{FromIterator, Peekable};
use std::vec::IntoIter;
use lazy_static::lazy_static;
use types::{MalType, ParseError, ParseResult};
use types::MalType::{Bool, Nil, Integer, Symbol, Float};
use types::ParseError::NoClosingParen;

type Reader = Peekable<IntoIter<String>>;

const REG_STRING: &str = r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#;

lazy_static!{
    static ref REGEX: Regex = Regex::new(REG_STRING).expect("regex failed");
}
pub fn read_str(text: String) -> ParseResult {
    let mut tokenized_result = tokenize(text).into_iter().peekable();
    read_form(& mut tokenized_result)
}

fn read_form(reader: & mut Reader) -> ParseResult {
    // println!("{:?} form", reader);
    let head = match reader.peek() {
        None => {return Nil.into()}
        Some(v) => {v}
    };

    if head == "(" {
        read_list(reader)
    } else if head == "[" {
        read_vector(reader)
    } else if head == "{" {
        read_hash_map(reader)
    } else if head == "'"{
        quote_name(reader, "quote")
    } else if head == "`" {
        quote_name(reader, "quasiquote")
    } else if head == "~" {
        quote_name(reader, "unquote")
    } else if head == "~@" {
        quote_name(reader, "splice-unquote")
    } else if head == "@" {
        quote_name(reader, "deref")
    } else if head == "^" {
        read_meta(reader)
    } else{
        read_atom(reader)
    }
}
fn quote_name(reader: & mut Reader, name: &str) -> ParseResult{
    reader.next();
    push_in_front(Symbol(name.to_string()), read_form(reader)?).into()
}
fn push_in_front(front: MalType, back: MalType) -> MalType{
    List::new().push_front(back).push_front(front).into()
}

fn read_atom(reader: & mut Reader) -> ParseResult {
    // println!("{:?} atom", reader);
    let head = reader.next().expect("should always have a value here");
    if head
        .chars()
        .next()
        .expect("should be greater then 0 elements")
        .is_numeric()
    {
        match head.parse::<i64>() {
            Ok(n) => {Integer(n).into()}
            Err(_) => {
                match head.parse::<f64>() {
                    Ok(n) => {Float(n.into()).into()}
                    Err(_) => {return Err(ParseError::InvalidNum(head))}
                }
            }
        }
    } else if head == "nil" {
        Nil.into()
    } else if head == "true" {
        Bool(true).into()
    } else if head == "false" {
        Bool(false).into()
    } else if head.chars().nth(0).expect("should have at least one value") == '"'{
        if head.chars().last().unwrap() != '"' {
            return Err(ParseError::NoClosingParen('"'));
        }
        MalType::String(head.chars().collect::<List<char>>()).into()
    } else {
        Symbol(head.clone()).into()
    }
}

fn read_list(reader: & mut Reader) -> ParseResult {
    // println!("{:?} list", reader);
    if reader.next().expect("should always have a value here") != "(" {
        panic!()
    }
    MalType::List(read_list_helper(reader)?).into()
}

fn read_list_helper(reader: & mut Reader) -> Result<List<MalType>, ParseError> {
    // println!("{:?} listhelper", reader);
    if reader.peek().ok_or(NoClosingParen(')'))? == ")" {
        reader.next();
        Ok(List::new())
    } else {
        let beggining = read_form(reader);
        let mut end = read_list_helper(reader)?;
        end.push_front_mut(beggining?);
        Ok(end)
    }
}
fn read_vector(reader: & mut Reader) -> ParseResult {
    // println!("{:?} vector", reader);
    if reader.next().expect("should always have a value here") != "[" {
        panic!()
    }
    let mut v: Vec<MalType> = vec![];
    while reader.peek().ok_or(NoClosingParen(']'))? != "]" {
        v.push(read_form(reader)?);
    }
    MalType::Vector(Vector::from_iter(v)).into()
}

fn read_hash_map(reader: & mut Reader) -> ParseResult {
    // println!("{:?} hashmap", reader);
    if reader.next().expect("should always have a value here") != "{" {
        panic!()
    }
    let mut v: Vec<(MalType, MalType)> = vec![];
    loop {
        if reader.peek().ok_or(NoClosingParen('}'))? == "}" {
            reader.next();
            break;
        }
        let key = read_form(reader)?;
        if reader.peek().ok_or(NoClosingParen('}'))? == "}" {
            return Err(ParseError::MissingValue(key));
        }
        let value = read_form(reader)?;
        v.push((key, value));
    }
    MalType::HashMap(HashTrieMap::from_iter(v).into()).into()
}


fn tokenize(text: String) -> Vec<String> {
    let captures = REGEX.captures_iter(text.as_str());
    let mut result: Vec<String> = vec![];
    for capture in captures {
        result.push(
            capture
                .get(1)
                .expect("should have returned a capture")
                .as_str()
                .to_string(),
        )
    }
    // println!("{:?}", result);
    result
}


mod tests {
    use super::*;

    #[test]
    fn test_tokenize() {
        let text = "(+ 1 2.11      hello )".to_string();
        let result = tokenize(text);
        print!("{:?}", result);
        assert_eq!(result.tokens, vec!["(", "+", "1", "2.11", "hello", ")"])
    }
}
