use im_rc::{HashMap, Vector};
use lazy_static::lazy_static;
use regex::Regex;
use std::iter::Peekable;
use std::vec::IntoIter;
use types::MalType::{Bool, Float, Integer, Nil, Symbol};
use types::ParseError::NoClosingParen;
use types::{MalType, ParseError, ParseResult};

type Reader = Peekable<IntoIter<String>>;

const REG_STRING: &str = r#"[\s,]*(~@|[\[\]{}()'`~^@]|"(?:\\.|[^\\"])*"?|;.*|[^\s\[\]{}('"`,;)]+)"#;

lazy_static! {
    static ref REGEX: Regex = Regex::new(REG_STRING).expect("regex failed");
}
pub fn read_str(text: String) -> ParseResult {
    let mut tokenized_result = tokenize(text).into_iter().peekable();
    read_form(&mut tokenized_result)
}

fn read_form(reader: &mut Reader) -> ParseResult {
    // println!("{:?} form", reader);
    let head = match reader.peek() {
        None => return Nil.into(),
        Some(v) => v,
    };

    if head == "(" {
        MalType::List(read_vector(reader, ')')?).into()
    } else if head == "[" {
        MalType::Vector(read_vector(reader, ']')?).into()
    } else if head == "{" {
        read_hash_map(reader)
    } else if head == "'" {
        quote_name(reader, "quote")
    } else if head == "`" {
        quote_name(reader, "quasiquote")
    } else if head == "~" {
        quote_name(reader, "unquote")
    } else if head == "~@" {
        quote_name(reader, "splice-unquote")
    } else if head == "@" {
        quote_name(reader, "deref")
    } else {
        read_atom(reader)
    }
}
fn quote_name(reader: &mut Reader, name: &str) -> ParseResult {
    reader.next();
    let mut v = Vector::new();
    v.push_back(Symbol(name.to_string()));
    v.push_back(read_form(reader)?);
    Ok(v.into())
}

fn read_atom(reader: &mut Reader) -> ParseResult {
    // println!("{:?} atom", reader);
    let head = reader.next().expect("should always have a value here");
    if head
        .chars()
        .next()
        .expect("should be greater then 0 elements")
        .is_numeric()
        || (head.len() > 1 && (head.starts_with("-") || head.starts_with("+")))
    {
        match head.parse::<i64>() {
            Ok(n) => Integer(n).into(),
            Err(_) => match head.parse::<f64>() {
                Ok(n) => Float(n.into()).into(),
                Err(_) => return Err(ParseError::InvalidNum(head)),
            },
        }
    } else if head == "nil" {
        Nil.into()
    } else if head == "true" {
        Bool(true).into()
    } else if head == "false" {
        Bool(false).into()
    } else if head.chars().nth(0).expect("should have at least one value") == '"' {
        if head.chars().last().unwrap() != '"' {
            return Err(ParseError::NoClosingParen('"'));
        }
        Ok(MalType::String(parse_str(head)))
    } else {
        Symbol(head.clone()).into()
    }
}

fn parse_str(s: String) -> Vector<char> {
    let mut v = Vector::new();
    let mut chars = s.chars();
    chars.next().unwrap();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next().unwrap() {
                'n' => v.push_back('\n'),
                next => v.push_back(next),
            }
        } else {
            v.push_back(c);
        }
    }
    v.pop_back().unwrap();
    v
}

fn read_vector(reader: &mut Reader, end: char) -> Result<Vector<MalType>, ParseError> {
    // println!("{:?} listhelper", reader);
    reader.next().unwrap();
    let mut result = Vector::new();
    loop {
        if reader.peek().ok_or(NoClosingParen(end))? == end.to_string().as_str() {
            reader.next().unwrap();
            break;
        } else {
            result.push_back(read_form(reader)?)
        }
    }
    Ok(result)
}

fn read_hash_map(reader: &mut Reader) -> ParseResult {
    // println!("{:?} hashmap", reader);
    if reader.next().expect("should always have a value here") != "{" {
        panic!()
    }
    let mut v = HashMap::new();
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
        v.insert(key, value);
    }
    Ok(MalType::HashMap(v))
}

fn tokenize(text: String) -> Vec<String> {
    let captures = REGEX.captures_iter(text.as_str());
    let mut result: Vec<String> = vec![];
    for capture in captures {
        let cap =
            capture
                .get(1)
                .expect("should have returned a capture")
                .as_str()
                .to_string();
        if cap.starts_with(";"){
            continue
        } else {
            result.push(cap);
        }
    }
    // println!("{:?}", result);
    result
}
