/// Matches a file name with a pattern and returns matched parts.
///
/// # Examples
///
/// ```no run
/// use pmv::fnmatch;
///
/// assert_eq!(fnmatch("f*??r", "foobar"), Some(vec![
///     String::from("oo"),
///     String::from("b"),
///     String::from("a"),
/// ]));
/// assert_eq!(fnmatch("f*??r", "blah"), None);
/// ```
pub fn fnmatch(pattern: &str, name: &str) -> Option<Vec<String>> {
    let pattern: Vec<char> = pattern.chars().collect();
    let pattern: &[char] = &pattern[..];
    let name: Vec<char> = name.chars().collect();
    let name: &[char] = &name[..];
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut matches: Vec<String> = Vec::new();
    loop {
        if pattern[i] == '?' {
            if name.len() <= j {
                return None; // no more chars available for this '?'
            }

            // Match one character
            matches.push(name[j..=j].iter().collect());
            i += 1;
            j += 1;
        } else if pattern[i] == '*' {
            if pattern.len() <= i + 1 {
                // Match all the remainings
                matches.push(name[j..].iter().collect());
                i += 1;
                j = name.len();
            } else if pattern[i + 1] == '*' {
                // Match an empty string (consume nothing)
                i += 1;
                matches.push(String::new());
            } else if pattern[i + 1] == '?' {
                // Count how many question marks are there
                let num_questions = 1 + strspn(pattern, i + 2, '?');
                let ii = i + 1 + num_questions;
                let matched_len = if ii < pattern.len() {
                    let term = pattern[ii];
                    if term == '*' {
                        return None; // Patterns like `*?*` are ambiguous
                    }
                    strcspn(name, j, term)
                } else {
                    name.len() - j
                };
                if matched_len < num_questions {
                    return None; // Too short for the question marks
                }

                // Keep matched parts
                let substr_for_star = &name[j..(j + matched_len - num_questions)];
                matches.push(substr_for_star.iter().collect());
                for jj in j + substr_for_star.len()..j + matched_len {
                    matches.push(name[jj..=jj].iter().collect());
                }
                i = ii;
                j += matched_len;
            } else {
                debug_assert!(i + 1 < pattern.len());
                let jj = j + strcspn(name, j, pattern[i + 1]);
                matches.push(name[j..jj].iter().collect());
                i += 1;
                j = jj;
            }
        } else if j < name.len() && match_chars(pattern[i], name[j]) {
            i += 1;
            j += 1;
        } else {
            return None;
        }

        if pattern.len() <= i {
            if name.len() == j {
                return Some(matches);
            } else {
                return None;
            }
        }
    }
}

fn strspn(s: &[char], i: usize, accept: char) -> usize {
    let mut j = i;
    while j < s.len() {
        if accept != s[j] {
            return j - i;
        }
        j += 1;
    }
    s.len() - i
}

fn strcspn(s: &[char], i: usize, reject: char) -> usize {
    let mut j = i;
    while j < s.len() {
        if reject == s[j] {
            return j - i;
        }
        j += 1;
    }
    s.len() - i
}

fn match_chars(a: char, b: char) -> bool {
    if cfg!(windows) {
        let offset = 'a' as u32 - 'A' as u32;

        let a = match a {
            'A'..='Z' => std::char::from_u32(a as u32 + offset).unwrap(),
            _ => a,
        };

        let b = match b {
            'A'..='Z' => std::char::from_u32(b as u32 + offset).unwrap(),
            _ => b,
        };

        a == b
    } else {
        a == b
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strspn() {
        let s: Vec<char> = "foobar".chars().collect();
        assert_eq!(strspn(&s[..], 0, 'o'), 0);
        assert_eq!(strspn(&s[..], 1, 'o'), 2);
        assert_eq!(strspn(&s[..], 5, 'r'), 1);
    }

    #[test]
    fn test_strcspn() {
        let s: Vec<char> = "foobar".chars().collect();
        assert_eq!(strcspn(&s[..], 0, 'f'), 0);
        assert_eq!(strcspn(&s[..], 1, 'b'), 2);
        assert_eq!(strcspn(&s[..], 2, 'x'), 4);
    }

    mod fnmatch {
        use super::*;

        #[test]
        fn no_special() {
            assert_eq!(fnmatch("fooba", "foobar"), None);
            assert_eq!(fnmatch("foobar", "foobar"), Some(vec![]));
            assert_eq!(fnmatch("foobar!", "foobar"), None);
        }

        #[test]
        fn case_sensitivity() {
            let actual = fnmatch("Abc", "abC");
            let expected = if cfg!(windows) {
                Some(Vec::new())
            } else {
                None
            };
            assert_eq!(actual, expected);
        }

        #[test]
        fn question_single() {
            assert_eq!(fnmatch("?oobar", "foobar"), Some(vec![String::from("f")]));
            assert_eq!(fnmatch("fooba?", "foobar"), Some(vec![String::from("r")]));
            assert_eq!(fnmatch("foobar?", "foobar"), None);
            assert_eq!(fnmatch("?", ""), None);
        }

        #[test]
        fn question_multiple() {
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
        fn question_non_ascii() {
            assert_eq!(fnmatch("I ? NY", "I ♡ NY"), Some(vec![String::from("♡")]));
        }

        #[test]
        fn star() {
            assert_eq!(fnmatch("f*r", "foobar"), Some(vec![String::from("ooba")]));
            assert_eq!(fnmatch("foo*", "foobar"), Some(vec![String::from("bar")]));
            assert_eq!(fnmatch("*bar", "foobar"), Some(vec![String::from("foo")]));
            assert_eq!(fnmatch("*", "foobar"), Some(vec![String::from("foobar")]));
            assert_eq!(fnmatch("*", ""), Some(vec![String::from("")]));
            assert_eq!(fnmatch("foo*", "foo"), Some(vec![String::from("")]));
        }

        #[test]
        fn star_star() {
            assert_eq!(
                fnmatch("f**r", "foobar"),
                Some(vec![String::from(""), String::from("ooba")])
            );
        }

        #[test]
        fn star_questions() {
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
        fn star_question_star() {
            assert_eq!(fnmatch("f*?*r", "foobar"), None);
        }
    }
}
