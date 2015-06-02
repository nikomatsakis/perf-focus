use super::*;

#[test]
fn matcher() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m = RegexMatcher::new(format!("b"));
    let r = m.search_trace(&x).unwrap();
    assert_eq!(r.prefix, &[format!("a")]);
    assert_eq!(r.suffix, &[format!("c")]);
}

#[test]
fn matcher_2() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m =
        ThenMatcher::new(
            Box::new(RegexMatcher::new(format!("b"))),
            Box::new(RegexMatcher::new(format!("c"))));
    let r = m.search_trace(&x).unwrap();
    assert_eq!(r.prefix, &[format!("a")]);
    assert!(r.suffix.is_empty());
}

#[test]
fn matcher_3() {
    let x = [format!("a"), format!("b"), format!("c")];
    let m =
        ThenMatcher::new(
            Box::new(RegexMatcher::new(format!("a"))),
            Box::new(RegexMatcher::new(format!("c"))));
    assert!(m.search_trace(&x).is_none());
}

#[test]
fn matcher_4() {
    let m =
        ThenMatcher::new(
            Box::new(RegexMatcher::new(format!("a"))),
            Box::new(SkipMatcher::new(
                Box::new(RegexMatcher::new(format!("c"))))));

    assert!(m.search_trace(&[format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 0);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);

    assert!(m.search_trace(&[format!("x"), format!("a"), format!("b"), format!("b"), format!("c")])
             .unwrap().prefix.len() == 1);
}
