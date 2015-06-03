use std::env;

extern crate regex;

#[macro_use]
extern crate rusty_peg;

mod matcher;

use matcher::{parse_matcher};

fn main() {
    let mut args = env::args();
    let arg0 = args.nth(1).unwrap();
    let r = match parse_matcher(&arg0) {
        Ok(r) => r,
        Err(err) => {
            println!("Error: {}", err.expected);
            println!("    {}", arg0);
            println!("    {0:1$}^", "", err.offset);
            return;
        }
    };
}
