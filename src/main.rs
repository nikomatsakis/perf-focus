use std::env;
use std::fmt::Display;
use std::fs::File;
use std::io::{self, stdin, BufWriter};
use std::process::exit;
use std::str::FromStr;

extern crate itertools;
extern crate regex;

#[macro_use]
extern crate rusty_peg;

mod histogram;
mod graph;
mod matcher;
mod rustc_query;
mod trace;
mod tree;
mod util;

use histogram::Histogram;
use graph::CallGraph;
use matcher::{parse_matcher, Matcher, SearchResult};
use regex::Regex;
use tree::Tree;
use util::percent;

trait AddFrames {
    fn add_frames<I>(&mut self, frames: I)
    where
        I: Iterator<Item = String>;
}

struct Options {
    process_name_filter: Option<regex::Regex>,
    rustc_query: bool,
    matcher: Option<Matcher>,
    print_match: bool,
    script_match: bool,
    script_miss: bool,
    graph_file: Option<String>,
    graph_mode: Option<GraphMode>,
    hist_mode: Option<GraphMode>,
    top_n: usize,
    tree_mode: Option<GraphMode>,
    tree_max_depth: usize,
    tree_min_percent: usize,
    rename: Vec<(regex::Regex, String)>,
}

fn usage(msg: &str) -> ! {
    println!("Usage: perf-focus [options] <matcher>");
    println!("");
    println!("Options:");
    println!(" --process-name <regex>   filter samples by process name");
    println!(" --rustc-query            convert from raw stacks to rustc query stacks");
    println!(" --print-match            dump samples that match and show why they matched");
    println!(" --print-miss             dump samples that do not match");
    println!(" --script-match           dump samples that match in `perf script` format");
    println!(" --script-miss            dump samples that do not match in `perf script` format");
    println!(" --top-n <n>              limit graph or histograms to the top <n> fns");
    println!(" --graph <file>           dumps a callgraph of matching samples into <file>");
    println!(" --graph-callers <file>   as above, but only dumps callers of the matcher");
    println!(" --graph-callees <file>   as above, but only dumps callees of the matcher");
    println!(" --hist                   prints out the most common fns");
    println!(" --hist-callers           prints out the most common fns amongst the callers");
    println!(" --hist-callees           prints out the most common fns amongst the callees");
    println!(" --tree                   prints out a tree of the samples");
    println!(" --tree-callers           prints out an (inverted) tree of the callers");
    println!(" --tree-callees           prints out a tree of the callees");
    println!(" --tree-max-depth <n>     limit tree to the outermost N functions");
    println!(" --tree-min-percent <n>   limit tree to fns whose total time exceeds N%");
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
        None => usage("Error: missing argument"),
    }
}

#[derive(Copy, Clone)]
enum GraphMode {
    All,
    Caller,
    Callee,
}

fn parse_options() -> Options {
    let mut args = env::args().skip(1);

    let mut options = Options {
        process_name_filter: None,
        rustc_query: false,
        matcher: None,
        script_match: false,
        print_match: false,
        script_miss: false,
        graph_file: None,
        graph_mode: None,
        hist_mode: None,
        tree_mode: None,
        top_n: 22,
        tree_max_depth: ::std::usize::MAX,
        tree_min_percent: 0,
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
                    usage(&format!(
                        "Error: invalid process name regular expression: {}",
                        e
                    ));
                }
            }
        } else if arg == "--print-match" {
            options.print_match = true;
        } else if arg == "--script-match" {
            options.script_match = true;
        } else if arg == "--rustc-query" {
            options.rustc_query = true;
        } else if arg == "--print-miss" || arg == "--script-miss" {
            options.script_miss = true;
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
        } else if arg == "--tree" {
            set_tree(&mut options, GraphMode::All);
        } else if arg == "--tree-callers" {
            set_tree(&mut options, GraphMode::Caller);
        } else if arg == "--tree-callees" {
            set_tree(&mut options, GraphMode::Callee);
        } else if arg == "--top-n" {
            let n = expect(usize::from_str(&*expect(args.next())).ok());
            options.top_n = n;
        } else if arg == "--tree-max-depth" {
            let n = expect(usize::from_str(&*expect(args.next())).ok());
            options.tree_max_depth = n;
        } else if arg == "--tree-min-percent" {
            let n = expect(usize::from_str(&*expect(args.next())).ok());
            options.tree_min_percent = n;
        } else if arg == "--rename" {
            let m = check_err(
                "invalid regular expression",
                Regex::new(&*expect(args.next())),
            );
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
                    usage(&format!(
                        "Error: invalid matcher: {} (*) {}",
                        &arg[..err.offset],
                        &arg[err.offset..]
                    ));
                }
            }
        }
    }

    if options.matcher.is_none() {
        usage("Error: no matcher supplied");
    }

    return options;

    fn set_graph(options: &mut Options, file_name: Option<String>, mode: GraphMode) {
        check_graph_hist_etc(options);
        options.graph_file = Some(expect(file_name));
        options.graph_mode = Some(mode);
    }

    fn set_hist(options: &mut Options, mode: GraphMode) {
        check_graph_hist_etc(options);
        options.hist_mode = Some(mode);
    }

    fn set_tree(options: &mut Options, mode: GraphMode) {
        check_graph_hist_etc(options);
        options.tree_mode = Some(mode);
    }

    fn check_graph_hist_etc(options: &Options) {
        if options.graph_mode.is_some() || options.hist_mode.is_some()
            || options.tree_mode.is_some()
        {
            usage("Error: graph, histogram, or tree already specified");
        }
    }
}

fn main() {
    let options = parse_options();
    let matcher = options.matcher.as_ref().unwrap();

    let mut graph = CallGraph::new();
    let mut hist = Histogram::new();
    let mut tree = Tree::new();
    let mut matches = 0;
    let mut not_matches = 0;
    let stdin = stdin();
    let stdin = stdin.lock();
    trace::each_trace(stdin, |mut args| {
        if let Some(ref regex) = options.process_name_filter {
            if !regex.is_match(args.process_name) {
                return;
            }
        }

        if options.rustc_query {
            rustc_query::to_query_stack(&mut args);
        }

        if let Some(result) = matcher.search_trace(&args.stack) {
            matches += 1;

            if options.print_match {
                print_trace(&args.header, Some(result));
            } else if options.script_match {
                print_trace(&args.header, None);
            }

            if let Some(mode) = options.hist_mode {
                add_frames(&matcher, mode, args.stack, result, &options, &mut hist);
            } else if let Some(mode) = options.graph_mode {
                add_frames(&matcher, mode, args.stack, result, &options, &mut graph);
            } else if let Some(mode) = options.tree_mode {
                add_frames(&matcher, mode, args.stack, result, &options, &mut tree);
            }
        } else {
            not_matches += 1;

            if options.script_miss {
                print_trace(&args.header, None);
            }
        }
    });

    let total = matches + not_matches;
    graph.set_total(total, options.top_n);

    if let Some(ref graph_file) = options.graph_file {
        check_err(
            &format!("Error printing graph to `{}`", graph_file),
            dump_graph(&graph, graph_file),
        );
    }

    println!("Matcher    : {:?}", matcher);
    println!("Matches    : {}", matches);
    println!("Not Matches: {}", not_matches);
    println!("Percentage : {}%", percent(matches, total));

    if options.hist_mode.is_some() {
        println!("");
        println!("Histogram");
        hist.dump(total, options.top_n);
    }

    if options.tree_mode.is_some() {
        println!("");
        println!("Tree");
        tree.sort();
        tree.dump(total, options.tree_max_depth, options.tree_min_percent);
    }
}

fn add_frames<F>(
    matcher: &Matcher,
    mode: GraphMode,
    frames: Vec<String>,
    result: SearchResult,
    options: &Options,
    acc: &mut F,
) where
    F: AddFrames,
{
    match mode {
        GraphMode::All => {
            acc.add_frames(frames.into_iter().map(|s| rename_frame(options, s)));
        }
        GraphMode::Caller => {
            let caller_frames: Vec<_> = frames
                .into_iter()
                .take(result.first_matching_frame)
                .map(|s| rename_frame(options, s))
                .chain(vec![format!("matched `{:?}`", matcher)].into_iter())
                .collect();
            acc.add_frames(caller_frames.into_iter().rev());
        }
        GraphMode::Callee => {
            acc.add_frames(
                vec![format!("matched `{:?}`", matcher)].into_iter().chain(
                    frames
                        .into_iter()
                        .skip(result.first_callee_frame)
                        .map(|s| rename_frame(options, s)),
                ),
            );
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
    if let Some(SearchResult {
        first_matching_frame,
        first_callee_frame,
    }) = selected
    {
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

        println!("{}", header[0]);
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
        for string in header {
            println!("{}", string);
        }
    }
    println!("");
}

fn dump_graph(graph: &CallGraph, graph_file: &str) -> io::Result<()> {
    let mut file = BufWriter::new(try!(File::create(graph_file)));
    graph.dump(&mut file)
}

fn check_err<O, E: Display>(prefix: &str, r: Result<O, E>) -> O {
    match r {
        Ok(o) => o,
        Err(e) => {
            println!("{}: {}", prefix, e);
            exit(1);
        }
    }
}
