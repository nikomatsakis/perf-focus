use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufWriter, stdin};
use std::process::exit;
use std::str::FromStr;

extern crate regex;

#[macro_use]
extern crate rusty_peg;

mod graph;
mod matcher;
mod trace;
mod util;

use graph::CallGraph;
use matcher::{parse_matcher, Matcher};
use regex::Regex;
use util::percent;

struct Options {
    process_name_filter: Option<regex::Regex>,
    matcher: Option<Matcher>,
    print_match: bool,
    print_miss: bool,
    graph_file: Option<String>,
    graph_mode: Option<GraphMode>,
    graph_threshold: u32,
}

fn usage(msg: &str) -> ! {
    println!("Usage: perf-focus [options] <matcher>");
    println!("");
    println!("Options:");
    println!(" --process-name <regex>   filter samples by process name");
    println!(" --print-match            dump samples that match");
    println!(" --print-miss             dump samples that do not match");
    println!(" --graph <file>           dumps a callgraph of matching samples into <file>");
    println!(" --graph-callers <file>   as above, but only dumps callers of the matcher");
    println!(" --graph-callees <file>   as above, but only dumps callees of the matcher");
    println!(" --graph-threshold <n>    limit graph to edges that occur in more than <n>%");
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

enum GraphMode { All, Caller, Callee }

fn parse_options() -> Options {
    let mut args = env::args().skip(1);

    let mut options = Options {
        process_name_filter: None,
        matcher: None,
        print_match: false,
        print_miss: false,
        graph_file: None,
        graph_mode: None,
        graph_threshold: 0,
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
        } else if arg == "--graph" {
            set_graph(&mut options, args.next(), GraphMode::All);
        } else if arg == "--graph-callers" {
            set_graph(&mut options, args.next(), GraphMode::Caller);
        } else if arg == "--graph-callees" {
            set_graph(&mut options, args.next(), GraphMode::Callee);
        } else if arg == "--graph_threshold" {
            let n = expect(u32::from_str(&*expect(args.next())).ok());
            options.graph_threshold = n;
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

    return options;

    fn set_graph(options:  &mut Options,
                 file_name: Option<String>,
                 mode: GraphMode)
    {
        if options.graph_file.is_some() {
            usage("Error: graph already specified");
        }
        options.graph_file = Some(expect(file_name));
        options.graph_mode = Some(mode);
    }
}

fn main() {
    let options = parse_options();
    let matcher = options.matcher.as_ref().unwrap();

    let mut graph = CallGraph::new();
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
            Some(result) => {
                matches += 1;

                if options.print_match {
                    print_trace(&args.header);
                }

                match options.graph_mode {
                    None => { }
                    Some(GraphMode::All) => {
                        graph.add_edges(args.stack.into_iter());
                    }
                    Some(GraphMode::Caller) => {
                        graph.add_edges(
                            args.stack
                                .into_iter()
                                .take(result.first_matching_frame)
                                .chain(vec![format!("matched `{:?}`", matcher)].into_iter()));
                    }
                    Some(GraphMode::Callee) => {
                        graph.add_edges(
                            vec![format!("matched `{:?}`", matcher)]
                                .into_iter()
                                .chain(args.stack.into_iter().skip(result.first_callee_frame)));
                    }
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

    if let Some(ref graph_file) = options.graph_file {
        check_err(&format!("Error printing graph to `{}`", graph_file),
                  dump_graph(&graph, graph_file, options.graph_threshold));
    }

    println!("Matcher    : {:?}", matcher);
    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);
    println!("Percentage : {}%", percent(matches, matches + not_matches));
}

fn print_trace(header: &[String]) {
    for string in header {
        println!("{}", string);
    }
    println!("");
}

fn dump_graph(graph: &CallGraph, graph_file: &str, graph_threshold: u32) -> io::Result<()> {
    let mut file = BufWriter::new(try!(File::create(graph_file)));
    graph.dump(&mut file, graph_threshold)
}

fn check_err<O,E:Display>(prefix: &str, r: Result<O,E>) -> O {
    match r {
        Ok(o) => o,
        Err(e) => {
            println!("{}: {}", prefix, e);
            exit(1);
        }
    }
}
