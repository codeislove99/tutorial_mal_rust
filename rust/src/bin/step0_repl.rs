extern crate mal_rust;

use std::io;
use std::io::{stdout, Write};

fn read(text: String) -> String {
    return text;
}
fn eval(text: String) -> String {
    return text;
}

fn print(text: String) -> String {
    return text;
}

fn rep(text: String) -> String {
    let text = read(text);
    let text = eval(text);
    let text = print(text);
    return text;
}
fn main() {
    let standard_input = io::stdin();
    loop {
        let mut buffer = String::new();
        print!("user> ");
        stdout().flush().expect("didn't flush properly");
        if standard_input
            .read_line(&mut buffer)
            .expect("didn't get the line properly")
            == 0
        {
            break;
        }
        let result = rep(buffer);
        print!("{}", result)
    }
}
