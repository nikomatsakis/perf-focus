use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, BufWriter, stdin};
use std::process::exit;
use std::str::FromStr;

extern crate regex;
extern crate itertools;

#[macro_use]
extern crate rusty_peg;

mod histogram;
mod graph;
mod matcher;
mod trace;
mod util;

use histogram::Histogram;
use graph::CallGraph;
use matcher::{parse_matcher, Matcher, SearchResult};
use regex::Regex;
use util::percent;

trait AddFrames {
    fn add_frames<I>(&mut self, frames: I)
        where I: Iterator<Item=String>;
}

struct Options {
    process_name_filter: Option<regex::Regex>,
    matcher: Option<Matcher>,
    print_match: bool,
    print_miss: bool,
    graph_file: Option<String>,
    graph_mode: Option<GraphMode>,
    hist_mode: Option<GraphMode>,
    threshold: usize,
    rename: Vec<(regex::Regex, String)>,
}

fn usage(msg: &str) -> ! {
    println!("Usage: perf-focus [options] <matcher>");
    println!("");
    println!("Options:");
    println!(" --process-name <regex>   filter samples by process name");
    println!(" --print-match            dump samples that match");
    println!(" --print-miss             dump samples that do not match");
    println!(" --threshold <n>          limit graph or histograms to the top <n> fns");
    println!(" --graph <file>           dumps a callgraph of matching samples into <file>");
    println!(" --graph-callers <file>   as above, but only dumps callers of the matcher");
    println!(" --graph-callees <file>   as above, but only dumps callees of the matcher");
    println!(" --hist                   prints out the most common fns");
    println!(" --hist-callers           prints out the most common fns amongst the callers");
    println!(" --hist-callees           prints out the most common fns amongst the callees");
    println!(" --rename <match> <repl>  post-process names for graphs/histograms;");
    println!("                          see `replace_all` in Regex doc [1] for instructions.");
    println!("                          May be specified more than once.");
    println!("                          [1]: http://doc.rust-lang.org/regex/regex/index.html");
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

#[derive(Copy, Clone)]
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
        hist_mode: None,
        threshold: 22,
        rename: vec![],
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
        } else if arg == "--hist" {
            set_hist(&mut options, GraphMode::All);
        } else if arg == "--hist-callers" {
            set_hist(&mut options, GraphMode::Caller);
        } else if arg == "--hist-callees" {
            set_hist(&mut options, GraphMode::Callee);
        } else if arg == "--threshold" {
            let n = expect(usize::from_str(&*expect(args.next())).ok());
            options.threshold = n;
        } else if arg == "--rename" {
            let m = check_err("invalid regular expression", Regex::new(&*expect(args.next())));
            let r = expect(args.next());
            options.rename.push((m, r));
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
        if options.graph_mode.is_some() || options.hist_mode.is_some() {
            usage("Error: graph or histogram already specified");
        }
        options.graph_file = Some(expect(file_name));
        options.graph_mode = Some(mode);
    }

    fn set_hist(options:  &mut Options,
                mode: GraphMode)
    {
        if options.graph_mode.is_some() || options.hist_mode.is_some() {
            usage("Error: graph or histogram already specified");
        }
        options.hist_mode = Some(mode);
    }
}

fn main() {
    let options = parse_options();
    let matcher = options.matcher.as_ref().unwrap();

    let mut graph = CallGraph::new();
    let mut hist = Histogram::new();
    let mut matches = 0;
    let mut not_matches = 0;
    let stdin = stdin();
    let stdin = stdin.lock();
    trace::each_trace(stdin, |args| {
        if let Some(ref regex) = options.process_name_filter {
            if !regex.is_match(args.process_name) {
                return;
            }
        }

        if let Some(result) = matcher.search_trace(&args.stack) {
            matches += 1;

            if options.print_match {
                print_trace(&args.header, Some(result));
            }

            match (options.hist_mode, options.graph_mode) {
                (Some(mode), _) => {
                    add_frames(&matcher, mode, args.stack, result, &options, &mut hist);
                }
                (_, Some(mode)) => {
                    add_frames(&matcher, mode, args.stack, result, &options, &mut graph);
                }
                (None, None) => {
                }
            }
        } else {
            not_matches += 1;

            if options.print_miss {
                print_trace(&args.header, None);
            }
        }
    });

    let total = matches + not_matches;
    graph.set_total(total, options.threshold);

    if let Some(ref graph_file) = options.graph_file {
        check_err(&format!("Error printing graph to `{}`", graph_file),
                  dump_graph(&graph, graph_file));
    }

    println!("Matcher    : {:?}", matcher);
    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);
    println!("Percentage : {}%", percent(matches, total));

    if options.hist_mode.is_some() {
        println!("");
        println!("Histogram");
        hist.dump(total, options.threshold);
    }
}

fn add_frames<F>(matcher: &Matcher,
                 mode: GraphMode,
                 frames: Vec<String>,
                 result: SearchResult,
                 options: &Options,
                 acc: &mut F)
    where F: AddFrames
{
    match mode {
        GraphMode::All => {
            acc.add_frames(
                frames.into_iter()
                      .map(|s| rename_frame(options, s)));
        }
        GraphMode::Caller => {
            acc.add_frames(
                frames
                    .into_iter()
                    .take(result.first_matching_frame)
                    .map(|s| rename_frame(options, s))
                    .chain(vec![format!("matched `{:?}`", matcher)].into_iter()));
        }
        GraphMode::Callee => {
            acc.add_frames(
                vec![format!("matched `{:?}`", matcher)]
                    .into_iter()
                    .chain(
                        frames.into_iter()
                              .skip(result.first_callee_frame)
                              .map(|s| rename_frame(options, s))));
        }
    }
}

fn rename_frame(options: &Options, frame: String) -> String {
    let mut frame = frame;
    for &(ref regex, ref repl) in &options.rename {
        let tmp = regex.replace_all(&frame, &repl[..]);
        frame = tmp;
    }
    frame
}

fn print_trace(header: &[String], selected: Option<SearchResult>) {
    println!("{}", header[0]);

    if let Some(SearchResult { first_matching_frame, first_callee_frame }) = selected {
        // The search result is expressed counting backwards from
        // **top** element in the stack, which is last in this list.
        //
        // Matcher: {a}..{b}
        // against Frames:
        //     z
        //     b
        //     y
        //     a
        //     x
        // yields `{ first_callee_frame: 4, first_matching_frame: 1 }`
        // we want to select from index `1..5`.
        let selection_start = header.len() - first_callee_frame;
        let selection_end = header.len() - first_matching_frame;

        for string in &header[1..selection_start] {
            println!("  {}", string);
        }
        for string in &header[selection_start..selection_end] {
            println!("| {}", string);
        }
        for string in &header[selection_end..] {
            println!("  {}", string);
        }
    } else {
        for string in &header[1..] {
            println!("  {}", string);
        }
    }
    println!("");
}

fn dump_graph(graph: &CallGraph, graph_file: &str) -> io::Result<()> {
    let mut file = BufWriter::new(try!(File::create(graph_file)));
    graph.dump(&mut file)
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
