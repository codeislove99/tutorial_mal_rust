use std::fmt::{Display, Formatter};
use types::MalType;
use types::MalType::Float;

impl Display for MalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().pr_str(true))
    }
}


impl MalType {

    pub fn pr_str(self, print_readably: bool) -> String {
        match self {
            MalType::Nil => "nil".to_string(),
            MalType::Bool(bool) => bool.to_string(),
            MalType::Integer(n) => n.to_string(),
            Float(n) => n.0.to_string(),
            MalType::List(l) => {
                let inner = l
                    .into_iter()
                    .map(|m| m.pr_str(print_readably))
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("({})", inner)
            }
            MalType::Vector(l)=> {
                let inner = l
                    .into_iter()
                    .map(|m| m.pr_str(print_readably))
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("[{}]", inner)
            }
            MalType::HashMap(h) => {
                let inner = h
                    .into_iter()
                    .map(|(k, v)| format!("{} {}", k.pr_str(print_readably), v.pr_str(print_readably)))
                    .collect::<Vec<String>>()
                    .join(" ");
                format!("{{{}}}", inner)
            }
            MalType::Symbol(s) => s.to_string(),
            MalType::String(s) => {
                if print_readably {
                    let string = s.iter().collect::<String>();
                    let string = string.replace("\n", "\\n");
                    let string = string.replace("\\", "\\\\");
                    let mut string = string.replace("\"", "\\\"");
                    string.push('"');
                    string.insert(0, '"');
                    string
                } else {
                    let mut string = s.iter().collect::<String>();
                    string.push('"');
                    string.insert(0, '"');
                    string
                }
            }
            MalType::Function(_) => {
                "#<function>".to_string()
            }
        }
    }
}