use pmv::fnmatch;

#[test]
fn test_fnmatch_no_special() {
    assert_eq!(fnmatch("fooba", "foobar"), None);
    assert_eq!(fnmatch("foobar", "foobar"), Some(vec![]));
    assert_eq!(fnmatch("foobar!", "foobar"), None);
}

#[test]
fn test_fnmatch_question_single() {
    assert_eq!(fnmatch("?oobar", "foobar"), Some(vec![String::from("f")]));
    assert_eq!(fnmatch("fooba?", "foobar"), Some(vec![String::from("r")]));
    assert_eq!(fnmatch("foobar?", "foobar"), None);
    assert_eq!(fnmatch("?", ""), None);
}

#[test]
fn test_fnmatch_question_multiple() {
    assert_eq!(
        fnmatch("?oo?ar", "foobar"),
        Some(vec![String::from("f"), String::from("b")])
    );
    assert_eq!(
        fnmatch("foob??", "foobar"),
        Some(vec![String::from("a"), String::from("r")])
    );
    assert_eq!(fnmatch("fooba??", "foobar"), None);
}

#[test]
fn test_fnmatch_star() {
    assert_eq!(fnmatch("f*r", "foobar"), Some(vec![String::from("ooba")]));
    assert_eq!(fnmatch("foo*", "foobar"), Some(vec![String::from("bar")]));
    assert_eq!(fnmatch("*bar", "foobar"), Some(vec![String::from("foo")]));
    assert_eq!(fnmatch("*", "foobar"), Some(vec![String::from("foobar")]));
    assert_eq!(fnmatch("*", ""), Some(vec![String::from("")]));
    assert_eq!(fnmatch("foo*", "foo"), Some(vec![String::from("")]));
}

#[test]
fn test_fnmatch_star_star() {
    assert_eq!(
        fnmatch("f**r", "foobar"),
        Some(vec![String::from(""), String::from("ooba")])
    );
}

#[test]
fn test_fnmatch_star_questions() {
    assert_eq!(
        fnmatch("fo*??r", "foobar"),
        Some(vec![
            String::from("o"),
            String::from("b"),
            String::from("a")
        ])
    );
    assert_eq!(
        fnmatch("foo*??r", "foobar"),
        Some(vec![String::from(""), String::from("b"), String::from("a")])
    );
    assert_eq!(fnmatch("foob*??r", "foobar"), None);

    assert_eq!(
        fnmatch("foo*??", "foobar"),
        Some(vec![
            String::from("b"),
            String::from("a"),
            String::from("r")
        ])
    );
}

#[test]
fn test_fnmatch_star_question_star() {
    assert_eq!(fnmatch("f*?*r", "foobar"), None);
}