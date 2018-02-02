/// Matchers match against stack traces. This is basically a simple
/// parser combinator.

#[cfg(test)]
mod test;

use rusty_peg::{self, Symbol};
use regex::Regex;
use std::fmt::{Debug, Error, Formatter};

type StackTrace<'stack> = &'stack [StackFrame];
type StackFrame = String;

type MatchResult<'stack> = Result<StackTrace<'stack>, MatchError>;

enum MatchError {
    // Some part of our query failed to find a match, but we should
    // skip the top frame and try again later. For example, if the
    // query is `{a},{b}` and it is matching against
    //
    //     x
    //     a <-- starting here
    //     y
    //     z
    //     a
    //     b
    //
    // then the `{a}` will match but the `{b}` match will yield
    // `RecoverableError`.  This will cause us to start matching again
    // from `y` (and we will eventually find a match later on).
    RecoverableError,

    // Some part of our query failed to find a match, and we should
    // stop trying. This others with skip queries like `{a}..{b}` and
    // `{a}..!{b}`. This is because the results are counterintuitive if we keep
    // searching, particularly in the negative case. Consider this case:
    //
    //     x
    //     a <-- first try here will fail...
    //     y
    //     b
    //     a <-- ..but second try starting here succeeds
    //     z
    //
    // This is basically "prolog cut"; the concept is fine, but apply
    // it to every `..` is sort of a bit strict perhaps. It might be
    // nice to have an operator that *didn't* cut, like `,..,` or
    // something.
    IrrecoverableError,
}

///////////////////////////////////////////////////////////////////////////

mod parser;

pub fn parse_matcher(s: &str) -> Result<Matcher, rusty_peg::Error> {
    let mut parser = parser::Parser::new(());
    match parser::MATCHER.parse_complete(&mut parser, s) {
        Ok(m) => Ok(m),
        Err(err) => Err(err)
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct Matcher {
    object: Box<MatcherTrait>
}

impl Matcher {
    fn new<M:MatcherTrait+'static>(m: M) -> Matcher {
        Matcher { object: Box::new(m) }
    }
}

impl Clone for Matcher {
    fn clone(&self) -> Matcher {
        Matcher { object: self.object.clone_object() }
    }
}

impl Debug for Matcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}", self.object)
    }
}

impl Matcher {
    /// Try to match `self` against `input`; if it fails, drop the
    /// bottom-most frame and match again. Keep doing this. If we ever
    /// find a match, return `Some`, else return `None`.
    pub fn search_trace<'stack>(&self, input: StackTrace<'stack>) -> Option<SearchResult> {
        // Drop off frames from the top until we find a match. Return
        // the frames we dropped, and those that followed the match.
        let mut stack = input;
        let mut dropped = 0;
        while !stack.is_empty() {
            match self.object.match_trace(stack) {
                Ok(suffix) => {
                    return Some(SearchResult {
                        first_matching_frame: dropped,
                        first_callee_frame: input.len() - suffix.len(),
                    });
                }
                Err(MatchError::RecoverableError) => {
                    dropped += 1;
                    stack = &stack[1..];
                }
                Err(MatchError::IrrecoverableError) => {
                    return None;
                }
            }
        }
        None
    }

    /// Try to match `self` against `input` without skipping any frames.
    fn match_trace<'stack>(&self, input: StackTrace<'stack>) -> MatchResult<'stack> {
        self.object.match_trace(input)
    }
}

///////////////////////////////////////////////////////////////////////////

trait MatcherTrait: Debug {
    /// Try to match `self` against `input` without skipping any frames.
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack>;

    /// Clone this matcher.
    fn clone_object(&self) -> Box<MatcherTrait>;
}

///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct SearchResult {
    pub first_matching_frame: usize,
    pub first_callee_frame: usize,
}

///////////////////////////////////////////////////////////////////////////

pub struct RegexMatcher {
    text: String,
    regex: Regex
}

impl RegexMatcher {
    pub fn new(r: &str) -> RegexMatcher {
        let regex = Regex::new(&r).unwrap();
        RegexMatcher { text: r.to_string(), regex: regex }
    }
}

impl MatcherTrait for RegexMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        if !s.is_empty() && self.regex.is_match(&s[0]) {
            Ok(&s[1..])
        } else {
            Err(MatchError::RecoverableError)
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(RegexMatcher { text: self.text.clone(),
                                regex: self.regex.clone() })
    }
}

impl Debug for RegexMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{{{}}}", self.text)
    }
}

///////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
pub struct WildcardMatcher {
    dummy: ()
}

impl WildcardMatcher {
    pub fn new() -> WildcardMatcher {
        WildcardMatcher { dummy: () }
    }
}

impl MatcherTrait for WildcardMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        if !s.is_empty() {
            Ok(&s[1..])
        } else {
            Err(MatchError::RecoverableError)
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(WildcardMatcher { dummy: () })
    }
}

impl Debug for WildcardMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, ".")
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct ParenMatcher {
    matcher: Matcher
}

impl ParenMatcher {
    pub fn new(other: Matcher) -> ParenMatcher {
        ParenMatcher { matcher: other }
    }
}

impl MatcherTrait for ParenMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        self.matcher.match_trace(s)
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(ParenMatcher { matcher: self.matcher.clone() })
    }
}

impl Debug for ParenMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "({:?})", self.matcher)
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct NotMatcher {
    matcher: Matcher
}

impl NotMatcher {
    pub fn new(other: Matcher) -> NotMatcher {
        NotMatcher { matcher: other }
    }
}

impl MatcherTrait for NotMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        // Make sure that `self.matcher` doesn't match *anywhere* in
        // the trace:
        match self.matcher.search_trace(s) {
            Some(_) => Err(MatchError::IrrecoverableError),
            None => Ok(s),
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(NotMatcher { matcher: self.matcher.clone() })
    }
}

impl Debug for NotMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "!{:?}", self.matcher)
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct ThenMatcher {
    left: Matcher,
    right: Matcher,
}

impl ThenMatcher {
    pub fn new(left: Matcher, right: Matcher) -> ThenMatcher {
        ThenMatcher { left: left, right: right }
    }
}

impl MatcherTrait for ThenMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        let t = try!(self.left.match_trace(s));
        let u = try!(self.right.match_trace(t));
        Ok(u)
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(ThenMatcher { left: self.left.clone(),
                              right: self.right.clone() })
    }
}

impl Debug for ThenMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?},{:?}", self.left, self.right)
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct SkipMatcher {
    matcher: Matcher,
}

impl SkipMatcher {
    pub fn new(matcher: Matcher) -> SkipMatcher {
        SkipMatcher { matcher: matcher }
    }
}

impl MatcherTrait for SkipMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        match self.matcher.search_trace(s) {
            Some(SearchResult { first_callee_frame, .. }) => Ok(&s[first_callee_frame..]),
            None => Err(MatchError::IrrecoverableError),
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(SkipMatcher { matcher: self.matcher.clone() })
    }
}

impl Debug for SkipMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "..{:?}", self.matcher)
    }
}
