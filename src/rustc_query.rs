//! Code to filter a trace down to queries.
//!
//! We are looking for symbols like this:
//!
//!     rustc::ty::maps::<impl rustc::ty::maps::queries::borrowck<'tcx>>::force
//!
//! which we want to transform to just the name of the query `borrowck`.

const QUERY_PREFIX_0: &str = "rustc::ty::maps::<impl rustc::ty::maps::queries::";
const QUERY_SUFFIX_0: &str = ">::force";
const QUERY_PREFIX_1: &str = "rustc::ty::maps::__query_compute::";
const QUERY_PREFIX_2: &str = "_ZN5rustc2ty4maps15__query_compute";

use std::str::FromStr;
use trace::TraceArgs;

pub fn to_query_stack(trace_args: &mut TraceArgs) {
    let stack: Vec<String> = ::std::iter::once("main()")
        .chain(trace_args.stack.iter().filter_map(|s| match_query(s)))
        .map(|s| s.to_string())
        .collect();

    trace_args.stack = stack;
}

fn match_query(frame: &str) -> Option<&str> {
    // Try multiple formats a query symbol can have in different versions of
    // the compiler.
    let query_name = if frame.starts_with(QUERY_PREFIX_0) && frame.ends_with(QUERY_SUFFIX_0) {
        &frame[QUERY_PREFIX_0.len()..frame.len() - QUERY_SUFFIX_0.len()]
    } else if frame.starts_with(QUERY_PREFIX_1) {
        &frame[QUERY_PREFIX_1.len()..]
    } else if frame.starts_with(QUERY_PREFIX_2) {
        // This is a mangled symbol, we have to parse out how many characters
        // to read for the query name.
        let num_chars_start = QUERY_PREFIX_2.len();
        let mut num_chars_end = num_chars_start + 1;
        while frame.as_bytes()[num_chars_end].is_ascii_digit() {
            num_chars_end += 1;
        }

        let num_chars = usize::from_str(&frame[num_chars_start..num_chars_end]).unwrap();

        &frame[num_chars_end .. num_chars_end + num_chars]
    } else {
        return None
    };

    Some(query_name)
}
