use std::fmt::{Display, Formatter};
use types::MalType;
use types::MalType::Float;

impl Display for MalType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MalType::Nil => write!(f, "nil"),
            MalType::Bool(bool) => write!(f, "{}", bool),
            MalType::Integer(n) => write!(f, "{}", n),
            Float(n) => write!(f, "{}", n.0),
            MalType::List(l) => {
                let inner = &l.iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                write!(f, "({})", inner)
            }
            MalType::Vector(v) => {
                let inner = &v.iter()
                        .map(|m| m.to_string())
                        .collect::<Vec<String>>()
                        .join(" ");
                write!(f, "[{}]", inner)
            }
            MalType::HashMap(h) => {
                let inner = &h.iter()
                        .map(|(k, v)| format!("{} {}", k.to_string(), v.to_string()))
                        .collect::<Vec<String>>()
                        .join(" ");
                write!(f, "{{{}}}", inner)
            }
            MalType::Symbol(s) => write!(f, "{}", s),
            MalType::String(s) => {
                let string = s.iter().collect::<String>();
                let string = string.replace("\\n", "\n");
                let string = string.replace("\\\\", "\\");
                let string = string.replace("\\\"", "\"");
                write!(f, "{}", string)
            }
        }
    }
}
