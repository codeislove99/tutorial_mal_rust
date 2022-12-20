extern crate mal_rust;

use std::io;
use std::io::{stdout, Write};

fn read(text: String) -> String{
    return text
}
fn eval(text: String) -> String{
    return text
}

fn print(text: String) -> String {
    return text
}

fn rep(text: String) -> String {
    let text= read(text);
    let text= eval(text);
    let text = print(text);
    return text
}
fn main() {

    let standard_input = io::stdin();
    while true {
        let mut buffer = String::new();
        print!("user> ");
        stdout().flush();
        standard_input.read_line(&mut buffer);
        let result = rep(buffer);
        print!("{}", result)
    }
}
