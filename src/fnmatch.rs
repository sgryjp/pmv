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
/// ```
pub fn fnmatch(pattern: &str, name: &str) -> Option<Vec<String>> {
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut matches: Vec<String> = Vec::new();
    loop {
        if pattern[i] == b'?' {
            if name.len() <= j {
                return None; // no more chars available for this '?'
            }

            // Match one character
            matches.push(String::from_utf8(name[j..=j].to_vec()).unwrap());
            i += 1;
            j += 1;
        } else if pattern[i] == b'*' {
            if pattern.len() <= i + 1 {
                // Match all the remainings
                matches.push(String::from_utf8(name[j..].to_vec()).unwrap());
                i += 1;
                j = name.len();
            } else if pattern[i + 1] == b'*' {
                // Match an empty string (consume nothing)
                i += 1;
                matches.push(String::new());
            } else if pattern[i + 1] == b'?' {
                // Count how many question marks are there
                let num_questions = 1 + strspn(pattern, i + 2, b'?');
                let ii = i + 1 + num_questions;
                let matched_len = if ii < pattern.len() {
                    let term = pattern[ii];
                    if term == b'*' {
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
                matches.push(String::from_utf8(substr_for_star.to_vec()).unwrap());
                for jj in j + substr_for_star.len()..j + matched_len {
                    matches.push(String::from_utf8(name[jj..=jj].to_vec()).unwrap());
                }
                i = ii;
                j += matched_len;
            } else {
                debug_assert!(i + 1 < pattern.len());
                let jj = j + strcspn(name, j, pattern[i + 1]);
                matches.push(String::from_utf8(name[j..jj].to_vec()).unwrap());
                i += 1;
                j = jj;
            }
        } else if j < name.len() && pattern[i] == name[j] {
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

fn strspn(s: &[u8], i: usize, accept: u8) -> usize {
    let mut j = i;
    while j < s.len() {
        if accept != s[j] {
            return j - i;
        }
        j += 1;
    }
    s.len() - i
}

fn strcspn(s: &[u8], i: usize, reject: u8) -> usize {
    let mut j = i;
    while j < s.len() {
        if reject == s[j] {
            return j - i;
        }
        j += 1;
    }
    s.len() - i
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strspn() {
        assert_eq!(strspn(b"foobar", 0, b'o'), 0);
        assert_eq!(strspn(b"foobar", 1, b'o'), 2);
        assert_eq!(strspn(b"foobar", 5, b'r'), 1);
    }

    #[test]
    fn test_strcspn() {
        assert_eq!(strcspn(b"foobar", 0, b'f'), 0);
        assert_eq!(strcspn(b"foobar", 1, b'b'), 2);
        assert_eq!(strcspn(b"foobar", 2, b'x'), 4);
    }
}
