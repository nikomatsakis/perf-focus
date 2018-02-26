#![allow(dead_code)]

use super::*;
use rusty_peg::{Error, Symbol, Input, ParseResult};

rusty_peg! {
    parser Parser<'input> {
        MATCHER: Matcher = (
            MATCHER_COMMA_MATCHER /
                MATCHER_NOT_THEN_MATCHER /
                MATCHER_THEN_NOT_MATCHER /
                MATCHER_SKIP_MATCHER /
                MATCHER0
        );

        MATCHER_COMMA_MATCHER: Matcher =
            (<lhs:MATCHER0>, ",", <rhs:MATCHER>) => {
                ThenMatcher::new(lhs, rhs)
            };

        MATCHER_THEN_NOT_MATCHER: Matcher =
            (<lhs:MATCHER0>, "..", "!", <rhs:MATCHER0>) => {
                ThenMatcher::new(lhs, NotMatcher::new(SkipMatcher::new(rhs)))
            };

        MATCHER_SKIP_MATCHER: Matcher =
            (<lhs:MATCHER0>, "..", <rhs:MATCHER>) => {
                ThenMatcher::new(lhs, SkipMatcher::new(rhs))
            };

        MATCHER_NOT_THEN_MATCHER: Matcher =
            ("!", <lhs:MATCHER0>, "..", <rhs:MATCHER0>) => {
                SkipMatcher::with_condition(rhs, NotMatcher::new(lhs))
            };

        MATCHER1: Matcher =
            (MATCHER_OR / MATCHER0);

        MATCHER_OR: Matcher =
            (<lhs:MATCHER0>, "/", <rhs:MATCHER1>) => OrMatcher::new(lhs, rhs);

        MATCHER0: Matcher =
            (MATCHER_RE / MATCHER_SKIP / MATCHER_PAREN / MATCHER_ANY);

        MATCHER_SKIP: Matcher =
            ("..", <rhs:MATCHER0>) => SkipMatcher::new(rhs);

        MATCHER_PAREN: Matcher =
            ("(", <rhs:MATCHER>, ")") => ParenMatcher::new(rhs);

        MATCHER_ANY: Matcher =
            (".") => WildcardMatcher::new();
    }
}

#[allow(non_camel_case_types)]
pub struct MATCHER_RE;

impl<'input> Symbol<'input, Parser<'input>> for MATCHER_RE {
    type Output = Matcher;

    fn pretty_print(&self) -> String {
        format!("MATCHER_RE")
    }

    fn parse(&self, _: &mut Parser<'input>, input: Input<'input>)
             -> ParseResult<'input,Matcher>
    {
        let bytes = input.text.as_bytes();
        let mut offset = input.offset;

        if offset >= input.text.len() || bytes[offset] != ('{' as u8) {
            return Err(Error { expected: "'{' character",
                               offset: input.offset });
        }

        let mut balance = 1;
        while balance != 0 {
            offset += 1;

            if offset >= input.text.len() {
                return Err(Error { expected: "matching '}' character",
                                   offset: offset });
            }

            if bytes[offset] == ('{' as u8) {
                balance += 1;
            } else if bytes[offset] == ('}' as u8) {
                balance -= 1;
            } else if bytes[offset] == ('\\' as u8) {
                offset += 1; // skip next character
            }
        }

        offset += 1; // consume final `}`

        let regex_str = &input.text[input.offset + 1 .. offset - 1];
        let regex: Matcher = RegexMatcher::new(regex_str);
        let output = Input { text: input.text, offset: offset };
        return Ok((output, regex));
    }
}
