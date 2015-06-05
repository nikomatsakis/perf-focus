/// Matchers match against stack traces. This is basically a simple
/// parser combinator.

#[cfg(test)]
mod test;

use rusty_peg::{self, Symbol};
use regex::Regex;
use std::fmt::{Debug, Error, Formatter};

type StackTrace<'stack> = &'stack [StackFrame];
type StackFrame = String;
type MatchResult<'stack> = Result<StackTrace<'stack>, StackTrace<'stack>>;

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
    pub fn search_trace(&self, input: StackTrace) -> Option<SearchResult> {
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
                Err(_) => {
                    dropped += 1;
                    stack = &stack[1..];
                }
            }
        }
        None
    }

    fn match_trace<'stack>(&self, input: StackTrace<'stack>) -> MatchResult<'stack> {
        self.object.match_trace(input)
    }
}

///////////////////////////////////////////////////////////////////////////

trait MatcherTrait: Debug {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack>;

    fn clone_object(&self) -> Box<MatcherTrait>;
}

///////////////////////////////////////////////////////////////////////////

#[allow(dead_code)]
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
            Err(s)
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
            Err(s)
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
        match self.matcher.match_trace(s) {
            Ok(t) => Err(t),
            Err(_) => Ok(s),
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

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(SkipMatcher { matcher: self.matcher.clone() })
    }
}

impl Debug for SkipMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "..{:?}", self.matcher)
    }
}
