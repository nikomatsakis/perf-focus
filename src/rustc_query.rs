//! Code to filter a trace down to queries.
//!
//! We are looking for symbols like this:
//!
//!     rustc::ty::maps::<impl rustc::ty::maps::queries::borrowck<'tcx>>::force
//!
//! which we want to transform to just the name of the query `borrowck`.

const QUERY_PREFIX: &str = "rustc::ty::maps::<impl rustc::ty::maps::queries::";
const QUERY_SUFFIX: &str = ">::force";

use trace::TraceArgs;

pub fn to_query_stack(trace_args: &mut TraceArgs) {
    let stack: Vec<String> = ::std::iter::once("main()")
        .chain(trace_args.stack.iter().filter_map(|s| match_query(s)))
        .map(|s| s.to_string())
        .collect();

    trace_args.stack = stack;
}

fn match_query(frame: &str) -> Option<&str> {
    if !frame.starts_with(QUERY_PREFIX) || !frame.ends_with(QUERY_SUFFIX) {
        return None;
    }

    let query_name = &frame[QUERY_PREFIX.len()..frame.len() - QUERY_SUFFIX.len()];
    Some(query_name)
}
