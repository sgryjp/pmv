use std::fs::{self, DirEntry};
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

/// Returns the directory entries which matched the given pattern.
///
/// This function recursively search directory tree for entries matching the
/// given pattern. While this function walks the directory tree, it remembers
/// which part of the path corresponds to which wildcard in the pattern.
/// For every matched entry this function finds, it creates a pair of an
/// `std::fs::DirEntry` for it and a vector of the substrings, collect them as
/// a vector, and return the vector.
pub fn walk(dir: &Path, pattern: &str) -> Result<Vec<(DirEntry, Vec<String>)>, String> {
    let mut matches: Vec<(DirEntry, Vec<String>)> = Vec::new();
    let mut matched_parts: Vec<String> = Vec::new();
    let patterns: Vec<Component> = Path::new(pattern).components().collect();
    walk1(dir, &patterns[..], &mut matches, &mut matched_parts)?;
    Ok(matches)
}

pub fn walk1(
    dir: &Path,
    patterns: &[Component],
    matches: &mut Vec<(DirEntry, Vec<String>)>,
    matched_parts: &mut Vec<String>,
) -> Result<(), String> {
    if patterns.is_empty() {
        return Ok(());
    }

    // Match directories
    match patterns[0] {
        Component::Prefix(p) => {
            // Reset the curdir to the path
            let curdir = p.as_os_str();
            let curdir = PathBuf::from(curdir);
            walk1(&curdir, &patterns[1..], matches, matched_parts)
        }
        Component::RootDir => {
            // Move to the root
            let root = MAIN_SEPARATOR.to_string();
            let root = PathBuf::from(root);
            walk1(root.as_path(), &patterns[1..], matches, matched_parts)
        }
        Component::ParentDir => {
            // Move to the parent
            let parent = dir.parent().unwrap(); //TODO: Handle error
            walk1(parent, &patterns[1..], matches, matched_parts)
        }
        Component::CurDir => {
            // Ignore the path component
            walk1(dir, &patterns[1..], matches, matched_parts)
        }
        Component::Normal(pattern) => {
            // Move into the matched sub-directories
            let entry_iter = match fs::read_dir(dir) {
                Err(err) => {
                    return Err(format!(
                        "fs::read_dir() failed: dir=\"{}\", error=\"{}\"",
                        dir.to_str().unwrap(),
                        err
                    ))
                }
                Ok(iter) => iter,
            };

            // Search entries of which name matches the pattern
            for maybe_entry in entry_iter {
                let entry = match maybe_entry {
                    Err(err) => return Err(format!("failed to get a directory entry: {}", err)), //TODO: Test this
                    Ok(entry) => entry,
                };
                let fname = entry.file_name();
                let pattern = pattern.to_str().unwrap();
                if let Some(mut m) = fnmatch(pattern, fname.to_str().unwrap()) {
                    // Call self for the remaining path-components, or store
                    // the matching result if it's a leaf
                    let mut matched_parts = matched_parts.clone();
                    matched_parts.append(&mut m);
                    let dir = dir.join(fname);
                    if 1 < patterns.len() {
                        let patterns_ = &patterns[1..];
                        walk1(dir.as_path(), patterns_, matches, &mut matched_parts)?;
                    } else {
                        matches.push((entry, matched_parts));
                    }
                }
            }
            Ok(())
        }
    }
}

/// Matches a file name with a pattern and returns matched parts.
///
/// # Examples
///
/// ```rust
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
        use super::fnmatch;

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

    mod walk {
        use std::fs;
        use std::path::Path;

        use super::walk;

        fn setup(id: &str) {
            let _ = fs::create_dir(Path::new("temp"));
            let _ = fs::remove_dir_all(Path::new(&format!("temp/{}", id)));
            for dir1 in ["foo", "bar", "baz"].iter() {
                for dir2 in ["foo", "bar", "baz"].iter() {
                    let _ =
                        fs::create_dir_all(Path::new(&format!("temp/{}/{}/{}", id, dir1, dir2)));
                    for fname in ["foo", "bar", "baz"].iter() {
                        let path: String = format!("temp/{}/{}/{}/{}", id, dir1, dir2, fname);
                        fs::write(Path::new(&path), path.as_bytes()).unwrap();
                    }
                }
            }
        }

        #[test]
        fn no_specials() {
            setup("no_specials");
            let matches = walk(Path::new("temp/no_specials"), "foo/bar/baz").unwrap();
            assert_eq!(matches.len(), 1);
            assert_eq!(
                matches[0].0.path(),
                Path::new("temp/no_specials/foo/bar/baz")
            );
            assert_eq!(matches[0].1, Vec::<String>::new());
        }

        #[test]
        fn question() {
            setup("question");
            let mut matches = walk(Path::new("temp/question"), "ba?/ba?/ba?").unwrap();
            assert_eq!(matches.len(), 8);
            matches.sort_by(|a, b| a.0.path().cmp(&b.0.path()));

            let paths: Vec<_> = matches.iter().map(|m| m.0.path()).collect();
            assert_eq!(
                paths,
                vec![
                    Path::new("temp/question/bar/bar/bar"),
                    Path::new("temp/question/bar/bar/baz"),
                    Path::new("temp/question/bar/baz/bar"),
                    Path::new("temp/question/bar/baz/baz"),
                    Path::new("temp/question/baz/bar/bar"),
                    Path::new("temp/question/baz/bar/baz"),
                    Path::new("temp/question/baz/baz/bar"),
                    Path::new("temp/question/baz/baz/baz"),
                ]
            );

            let patterns: Vec<_> = matches
                .iter()
                .map(|x| {
                    let s = &x.1;
                    s.iter().fold("".to_string(), |acc, x| acc + "." + x)
                })
                .collect();
            assert_eq!(
                patterns,
                vec![
                    String::from(".r.r.r"),
                    String::from(".r.r.z"),
                    String::from(".r.z.r"),
                    String::from(".r.z.z"),
                    String::from(".z.r.r"),
                    String::from(".z.r.z"),
                    String::from(".z.z.r"),
                    String::from(".z.z.z"),
                ]
            );
        }

        #[test]
        fn star() {
            setup("star");
            let mut matches = walk(Path::new("temp/star"), "b*/b*/b*").unwrap();
            assert_eq!(matches.len(), 8);
            matches.sort_by(|a, b| a.0.path().cmp(&b.0.path()));

            let paths: Vec<_> = matches.iter().map(|x| x.0.path()).collect();
            assert_eq!(
                paths,
                vec![
                    Path::new("temp/star/bar/bar/bar"),
                    Path::new("temp/star/bar/bar/baz"),
                    Path::new("temp/star/bar/baz/bar"),
                    Path::new("temp/star/bar/baz/baz"),
                    Path::new("temp/star/baz/bar/bar"),
                    Path::new("temp/star/baz/bar/baz"),
                    Path::new("temp/star/baz/baz/bar"),
                    Path::new("temp/star/baz/baz/baz"),
                ]
            );

            let patterns: Vec<_> = matches
                .iter()
                .map(|x| {
                    let s = &x.1;
                    s.iter().fold("".to_string(), |acc, x| acc + "." + x)
                })
                .collect();
            assert_eq!(
                patterns,
                vec![
                    String::from(".ar.ar.ar"),
                    String::from(".ar.ar.az"),
                    String::from(".ar.az.ar"),
                    String::from(".ar.az.az"),
                    String::from(".az.ar.ar"),
                    String::from(".az.ar.az"),
                    String::from(".az.az.ar"),
                    String::from(".az.az.az"),
                ]
            );
        }
    }
}
