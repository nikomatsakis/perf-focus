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
        Err(err) => Err(err),
    }
}

pub fn empty_matcher() -> Matcher {
    EmptyMatcher::new()
}

///////////////////////////////////////////////////////////////////////////

pub struct Matcher {
    object: Box<MatcherTrait>,
}

impl Matcher {
    fn new<M: MatcherTrait + 'static>(m: M) -> Matcher {
        Matcher {
            object: Box::new(m),
        }
    }
}

impl Clone for Matcher {
    fn clone(&self) -> Matcher {
        Matcher {
            object: self.object.clone_object(),
        }
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
        self.search_trace_while(input, &empty_matcher())
    }

    /// Like `search_trace`, except that before we drop, we test
    /// `condition` against the frame we are about to drop to make
    /// sure it is true.
    pub fn search_trace_while<'stack>(
        &self,
        input: StackTrace<'stack>,
        condition: &Matcher,
    ) -> Option<SearchResult> {
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
                    if condition.match_trace(stack).is_err() {
                        return None;
                    }
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

    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        self.object.match_trace(s)
    }

    fn is_empty(&self) -> bool {
        self.object.is_empty()
    }
}

///////////////////////////////////////////////////////////////////////////

trait MatcherTrait: Debug + 'static {
    /// Try to match `self` against `input` without skipping any frames.
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack>;

    /// Clone this matcher.
    fn clone_object(&self) -> Box<MatcherTrait>;

    /// True if this is the empty matcher.
    fn is_empty(&self) -> bool { false }
}

///////////////////////////////////////////////////////////////////////////

#[derive(Copy, Clone)]
pub struct SearchResult {
    pub first_matching_frame: usize,
    pub first_callee_frame: usize,
}

///////////////////////////////////////////////////////////////////////////

/// Consume any frame that matches the given regular expression.
pub struct RegexMatcher {
    text: String,
    regex: Regex,
}

impl RegexMatcher {
    pub fn new(r: &str) -> Matcher {
        let regex = Regex::new(&r).unwrap();
        Matcher::new(RegexMatcher {
            text: r.to_string(),
            regex: regex,
        })
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
        Box::new(RegexMatcher {
            text: self.text.clone(),
            regex: self.regex.clone(),
        })
    }
}

impl Debug for RegexMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{{{}}}", self.text)
    }
}

///////////////////////////////////////////////////////////////////////////

/// Consume any one frame.
#[allow(dead_code)]
pub struct WildcardMatcher {
    dummy: (),
}

impl WildcardMatcher {
    pub fn new() -> Matcher {
        Matcher::new(WildcardMatcher { dummy: () })
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

/// Always succeeds, consuming no frames.
#[allow(dead_code)]
pub struct EmptyMatcher {
    dummy: (),
}

impl EmptyMatcher {
    pub fn new() -> Matcher {
        Matcher::new(EmptyMatcher { dummy: () })
    }
}

impl MatcherTrait for EmptyMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        Ok(s)
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(EmptyMatcher { dummy: () })
    }

    fn is_empty(&self) -> bool {
        true
    }
}

impl Debug for EmptyMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "()")
    }
}

///////////////////////////////////////////////////////////////////////////

/// Try `matcher`.
pub struct ParenMatcher {
    matcher: Matcher,
}

impl ParenMatcher {
    pub fn new(other: Matcher) -> Matcher {
        Matcher::new(ParenMatcher { matcher: other })
    }
}

impl MatcherTrait for ParenMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        self.matcher.match_trace(s)
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(ParenMatcher {
            matcher: self.matcher.clone(),
        })
    }
}

impl Debug for ParenMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "({:?})", self.matcher)
    }
}

///////////////////////////////////////////////////////////////////////////

/// Try `matcher`; if it succeeds, fail. Otherwise, succeed, consuming no frames.
pub struct NotMatcher {
    matcher: Matcher,
}

impl NotMatcher {
    pub fn new(other: Matcher) -> Matcher {
        Matcher::new(NotMatcher { matcher: other })
    }
}

impl MatcherTrait for NotMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        // Make sure that `self.matcher` doesn't match *anywhere* in
        // the trace:
        match self.matcher.match_trace(s) {
            Ok(_) => Err(MatchError::RecoverableError),
            Err(_) => Ok(s),
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(NotMatcher {
            matcher: self.matcher.clone(),
        })
    }
}

impl Debug for NotMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "!{:?}", self.matcher)
    }
}

///////////////////////////////////////////////////////////////////////////

/// Try `left` then try `right` on what follows.
pub struct ThenMatcher {
    left: Matcher,
    right: Matcher,
}

impl ThenMatcher {
    pub fn new(left: Matcher, right: Matcher) -> Matcher {
        Matcher::new(ThenMatcher {
            left: left,
            right: right,
        })
    }
}

impl MatcherTrait for ThenMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        let t = self.left.match_trace(s)?;
        let u = self.right.match_trace(t)?;
        Ok(u)
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(ThenMatcher {
            left: self.left.clone(),
            right: self.right.clone(),
        })
    }
}

impl Debug for ThenMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?},{:?}", self.left, self.right)
    }
}

///////////////////////////////////////////////////////////////////////////

/// Try `needle`: if it succeds, we are done. If it fails, test condition.
/// If condition fails, then we fail. Otherwise, drop the frame and continue.
pub struct SkipMatcher {
    needle: Matcher,
    condition: Matcher,
}

impl SkipMatcher {
    pub fn new(needle: Matcher) -> Matcher {
        Self::with_condition(needle, empty_matcher())
    }

    pub fn with_condition(needle: Matcher, condition: Matcher) -> Matcher {
        Matcher::new(SkipMatcher { needle, condition })
    }
}

impl MatcherTrait for SkipMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        match self.needle.search_trace_while(s, &self.condition) {
            Some(SearchResult {
                first_callee_frame, ..
            }) => Ok(&s[first_callee_frame..]),
            None => Err(MatchError::IrrecoverableError),
        }
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(SkipMatcher {
            needle: self.needle.clone(),
            condition: self.condition.clone(),
        })
    }
}

impl Debug for SkipMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        if self.condition.is_empty() {
            write!(fmt, "..{:?}", self.needle)
        } else {
            write!(fmt, "..{:?} while {:?}", self.needle, self.condition)
        }
    }
}

///////////////////////////////////////////////////////////////////////////

/// Try `left` first; if it fails, try `right.
pub struct OrMatcher {
    left: Matcher,
    right: Matcher,
}

impl OrMatcher {
    pub fn new(left: Matcher, right: Matcher) -> Matcher {
        Matcher::new(OrMatcher {
            left: left,
            right: right,
        })
    }
}

impl MatcherTrait for OrMatcher {
    fn match_trace<'stack>(&self, s: StackTrace<'stack>) -> MatchResult<'stack> {
        self.left.match_trace(s).or_else(|_| self.right.match_trace(s))
    }

    fn clone_object(&self) -> Box<MatcherTrait> {
        Box::new(OrMatcher {
            left: self.left.clone(),
            right: self.right.clone(),
        })
    }
}

impl Debug for OrMatcher {
    fn fmt(&self, fmt: &mut Formatter) -> Result<(), Error> {
        write!(fmt, "{:?}/{:?}", self.left, self.right)
    }
}
