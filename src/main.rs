use std::env;
use std::io::stdin;
use std::process::exit;

extern crate regex;

#[macro_use]
extern crate rusty_peg;

mod matcher;
mod trace;

use matcher::{parse_matcher, Matcher};
use regex::Regex;

struct Options {
    process_name_filter: Option<regex::Regex>,
    matcher: Option<Matcher>,
    print_match: bool,
    print_miss: bool,
}

fn usage(msg: &str) -> ! {
    println!("Usage: perf-focus [options] <matcher>");
    println!("");
    println!("Options:");
    println!(" --process-name <regex>   filter samples by process name");
    println!(" --print-match            dump samples that match");
    println!(" --print-miss             dump samples that do not match");
    println!("");
    println!("{}", msg);
    exit(1)
}

fn expect<T>(t: Option<T>) -> T {
    match t {
        Some(v) => v,
        None => usage("Error: missing argument")
    }
}

fn parse_options() -> Options {
    let mut args = env::args().skip(1);

    let mut options = Options {
        process_name_filter: None,
        matcher: None,
        print_match: false,
        print_miss: false,
    };

    while let Some(arg) = args.next() {
        if arg == "--process-name" {
            if options.process_name_filter.is_some() {
                usage(&format!("Error: process-name already specified"));
            }

            let process_name_arg = expect(args.next());
            match Regex::new(&process_name_arg) {
                Ok(r) => {
                    options.process_name_filter = Some(r);
                }
                Err(e) => {
                    usage(&format!("Error: invalid process name regular expression: {}", e));
                }
            }
        } else if arg == "--print-match" {
            options.print_match = true;
        } else if arg == "--print-miss" {
            options.print_miss = true;
        } else if arg.starts_with("-") {
            usage(&format!("Error: unknown argument: {}", arg));
        } else if options.matcher.is_some() {
            usage(&format!("Error: matcher already specified"));
        } else {
            match parse_matcher(&arg) {
                Ok(r) => {
                    options.matcher = Some(r);
                }
                Err(err) => {
                    usage(&format!("Error: invalid matcher: {} (*) {}",
                                   &arg[..err.offset],
                                   &arg[err.offset..]));
                }
            }
        }
    }

    if options.matcher.is_none() {
        usage("Error: no matcher supplied");
    }

    options
}

fn main() {
    let options = parse_options();
    let matcher = options.matcher.as_ref().unwrap();

    let mut matches = 0;
    let mut not_matches = 0;
    let stdin = stdin();
    let stdin = stdin.lock();
    trace::each_trace(stdin, |args| {
        match options.process_name_filter {
            Some(ref regex) => {
                if !regex.is_match(args.process_name) {
                    return;
                }
            }
            None => { }
        }

        match matcher.search_trace(&args.stack) {
            Some(_) => {
                matches += 1;

                if options.print_match {
                    print_trace(&args.header);
                }
            }
            None => {
                not_matches += 1;

                if options.print_miss {
                    print_trace(&args.header);
                }
            }
        }
    });

    println!("Matcher    : {:?}", matcher);
    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);

    let matchesf = matches as f64;
    let not_matchesf = not_matches as f64;
    let totalf = matchesf + not_matchesf;
    let percentage = matchesf / totalf * 100.0;
    println!("Percentage : {}%", percentage);
}

fn print_trace(header: &[String]) {
    for string in header {
        println!("{}", string);
    }
    println!("");
}
