/// Matchers match against stack traces. This is basically a simple
/// parser combinator.

#[cfg(test)]
mod test;

use regex::Regex;
use std::fmt::{Debug, Error, Formatter};

type StackTrace<'stack> = &'stack [StackFrame];
type StackFrame = String;
type MatchResult<'stack> = Result<StackTrace<'stack>, StackTrace<'stack>>;

pub trait Matcher {
    fn search_trace<'stack>(&self, input: StackTrace<'stack>) -> Option<SearchResult<'stack>> {
        // Drop off frames from the top until we find a match. Return
        // the frames we dropped, and those that followed the match.
        let mut stack = input;
        let mut dropped = 0;
        while !stack.is_empty() {
            match self.match_trace(stack) {
                Ok(suffix) => {
                    return Some(SearchResult {
                        input: input,
                        prefix: &input[0..dropped],
                        suffix: suffix
                    });
                }
                Err(_) => {
                    dropped += 1;
                    stack = &stack[1..];
                }
            }
        }
        None
    }

    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack>;
    fn debug(&self) -> String;
}

///////////////////////////////////////////////////////////////////////////

pub struct SearchResult<'stack> {
    // all frames that were provided as input
    input: StackTrace<'stack>,

    // those we had to drop
    prefix: StackTrace<'stack>,

    // the suffix
    suffix: StackTrace<'stack>,
}

///////////////////////////////////////////////////////////////////////////

pub struct RegexMatcher {
    text: String,
    regex: Regex
}

impl RegexMatcher {
    pub fn new(r: String) -> RegexMatcher {
        let regex = Regex::new(&r).unwrap();
        RegexMatcher { text: r, regex: regex }
    }
}

impl Matcher for RegexMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        if !s.is_empty() && self.regex.is_match(&s[0]) {
            Ok(&s[1..])
        } else {
            Err(s)
        }
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for RegexMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "[{}]", self.text)
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct WildcardMatcher {
    dummy: ()
}

impl WildcardMatcher {
    pub fn new() -> WildcardMatcher {
        WildcardMatcher { dummy: () }
    }
}

impl Matcher for WildcardMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        if !s.is_empty() {
            Ok(&s[1..])
        } else {
            Err(s)
        }
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for WildcardMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, ".")
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct EmptyMatcher {
    dummy: ()
}

impl EmptyMatcher {
    pub fn new() -> EmptyMatcher {
        EmptyMatcher { dummy: () }
    }
}

impl Matcher for EmptyMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        Ok(s)
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for EmptyMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "*")
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct KleeneStarMatcher {
    matcher: Box<Matcher>
}

impl KleeneStarMatcher {
    pub fn new(other: Box<Matcher>) -> KleeneStarMatcher {
        KleeneStarMatcher { matcher: other }
    }
}

impl Matcher for KleeneStarMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        let mut s = s;
        loop {
            match self.matcher.match_trace(s) {
                Ok(t) => { s = t; }
                Err(t) => { return Ok(s); }
            }
        }
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for KleeneStarMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{}*", self.matcher.debug())
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct ThenMatcher {
    left: Box<Matcher>,
    right: Box<Matcher>,
}

impl ThenMatcher {
    pub fn new(left: Box<Matcher>, right: Box<Matcher>) -> ThenMatcher {
        ThenMatcher { left: left, right: right }
    }
}

impl Matcher for ThenMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        let t = try!(self.left.match_trace(s));
        let u = try!(self.right.match_trace(t));
        Ok(u)
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for ThenMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{},{}", self.left.debug(), self.right.debug())
    }
}

///////////////////////////////////////////////////////////////////////////

pub struct SkipMatcher {
    matcher: Box<Matcher>,
}

impl SkipMatcher {
    pub fn new(matcher: Box<Matcher>) -> SkipMatcher {
        SkipMatcher { matcher: matcher }
    }
}

impl Matcher for SkipMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        let mut t = s;
        loop {
            match self.matcher.match_trace(t) {
                Ok(u) => { return Ok(u); }
                Err(_) => {
                    if t.len() == 0 {
                        return Err(s);
                    }

                    t = &t[1..];
                }
            }
        }
    }

    fn debug(&self) -> String {
        format!("{:?}", self)
    }
}

impl Debug for SkipMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "..{}", self.matcher.debug())
    }
}
