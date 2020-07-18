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
}
