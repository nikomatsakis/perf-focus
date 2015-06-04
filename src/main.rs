use std::env;
use std::io::stdin;

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

    let stdin = stdin();
    let stdin = stdin.lock();
    trace::each_trace(stdin, |frames| {
        if matcher.search_trace(frames).is_some() {
            matches += 1;
        } else {
            not_matches += 1;
        }
    });

    println!("Matcher    : {:?}", matcher);
    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);

    let matchesf = matches as f64;
    let not_matchesf = not_matches as f64;
    let totalf = matchesf + not_matchesf;
    let percentage = matchesf / totalf * 100.0;
    println!("Percentage : {:3.0}%", percentage);
}
