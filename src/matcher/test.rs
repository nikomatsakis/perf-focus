use super::*;

#[test]
fn matcher() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m = Matcher::new(RegexMatcher::new("b"));
    let r = m.search_trace(&x).unwrap();
    assert_eq!(r.prefix, &[format!("a")]);
    assert_eq!(r.suffix, &[format!("c")]);
}

#[test]
fn matcher_2() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m =
        Matcher::new(
            ThenMatcher::new(
                Matcher::new(RegexMatcher::new("b")),
                Matcher::new(RegexMatcher::new("c"))));
    let r = m.search_trace(&x).unwrap();
    assert_eq!(r.prefix, &[format!("a")]);
    assert!(r.suffix.is_empty());
}

#[test]
fn matcher_3() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m =
        Matcher::new(
            ThenMatcher::new(
                Matcher::new(RegexMatcher::new("a")),
                Matcher::new(RegexMatcher::new("c"))));
    assert!(m.search_trace(&x).is_none());
}

#[test]
fn matcher_4() {
    let m =
        Matcher::new(
            ThenMatcher::new(
                Matcher::new(RegexMatcher::new("a")),
                Matcher::new(SkipMatcher::new(
                    Matcher::new(RegexMatcher::new("c"))))));

    assert!(m.search_trace(&[format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 0);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);
}

#[test]
fn matcher_parse_a_dotdot_c() {
    let m = parse_matcher("{a}..{c}").unwrap();

    assert!(m.search_trace(&[format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 0);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);
}

#[test]
fn matcher_parse_a_then_c() {
    let m = parse_matcher("{a},{c}").unwrap();

    assert!(m.search_trace(&[format!("a"), format!("b"), format!("c")])
             .is_none());

    assert!(m.search_trace(&[format!("a"), format!("c")])
             .unwrap().prefix.len() == 0);
}
