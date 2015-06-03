#![allow(dead_code)]

use super::*;
use rusty_peg::{Error, Symbol, Input, ParseResult};

rusty_peg! {
    parser Parser<'input> {
        MATCHER: Matcher =
            fold(<lhs:MATCHER0>,
                 (",", <rhs:MATCHER0>) => {
                     Matcher::new(ThenMatcher::new(lhs, rhs))
                 },
                 ("..", <rhs:MATCHER0>) => {
                     Matcher::new(ThenMatcher::new(lhs, Matcher::new(SkipMatcher::new(rhs))))
                 });

        MATCHER0: Matcher =
            (MATCHER_RE / MATCHER_SKIP / MATCHER_PAREN /
             MATCHER_NOT / MATCHER_ALL / MATCHER_ANY);

        MATCHER_SKIP: Matcher =
            ("..", <rhs:MATCHER0>) => Matcher::new(SkipMatcher::new(rhs));

        MATCHER_PAREN: Matcher =
            ("(", <rhs:MATCHER>, ")") => Matcher::new(ParenMatcher::new(rhs));

        MATCHER_NOT: Matcher =
            ("!", <rhs:MATCHER0>) => Matcher::new(NotMatcher::new(rhs));

        MATCHER_ALL: Matcher =
            (ALL, <rhs:MATCHER0>) => Matcher::new(AllMatcher::new(rhs));

        MATCHER_ANY: Matcher =
            (".") => Matcher::new(WildcardMatcher::new());

        ALL: &'input str =
            regex(r"all\b");
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
            }
        }

        offset += 1; // consume final `}`

        let regex_str = &input.text[input.offset + 1 .. offset - 1];
        let regex: Matcher = Matcher::new(RegexMatcher::new(regex_str));
        let output = Input { text: input.text, offset: offset };
        return Ok((output, regex));
    }
}
