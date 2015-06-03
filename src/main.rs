use std::env;

extern crate regex;

#[macro_use]
extern crate rusty_peg;

mod matcher;
mod trace;

use matcher::{parse_matcher};

fn main() {
    let mut args = env::args();
    let arg0 = args.nth(1).unwrap();
    let matcher = match parse_matcher(&arg0) {
        Ok(r) => r,
        Err(err) => {
            println!("Error: {}", err.expected);
            println!("    {}", arg0);
            println!("    {0:1$}^", "", err.offset);
            return;
        }
    };

    let mut matches = 0;
    let mut not_matches = 0;

    trace::each_trace(|trace| {
        if matcher.search_trace(&trace.frames).is_some() {
            matches += 1;
        } else {
            println!("{:?}", trace);
            not_matches += 1;
        }
    });

    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);
    println!("Percentage : {:3.0}%", (matches as f64) * 100.0 / (not_matches as f64));
}
