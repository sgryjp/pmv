use std::path::MAIN_SEPARATOR;

/// Replaces variables in the given destination path string using the given
/// substrings.
pub fn resolve(dest: &str, substrings: &[String]) -> String {
    let dest = dest.as_bytes();
    let mut resolved = String::new();
    let mut i = 0;
    while i < dest.len() {
        if dest[i] == b'#' && i + 1 < dest.len() && b'1' <= dest[i + 1] && dest[i + 1] <= b'9' {
            let index = (dest[i + 1] - b'1') as usize;
            let replacement = match substrings.get(index) {
                Some(s) => s,
                None => {
                    resolved.push('#');
                    resolved.push(dest[i + 1] as char);
                    i += 2;
                    continue;
                }
            };
            resolved.push_str(replacement);
            i += 2;
        } else if dest[i] == b'\\' || dest[i] == b'/' {
            resolved.push(MAIN_SEPARATOR);
            i += 1;
        } else {
            resolved.push(dest[i] as char);
            i += 1;
        }
    }
    resolved
}

#[cfg(test)]
mod tests {
    use super::*;

    mod resolve {
        use super::*;

        // Testing points of view:
        // - dest
        //   - empty
        //   - no variables
        //   - sharp
        //   - sharp 0
        //   - sharp 1
        //   - sharp 9
        //   - sharp colon
        //   - sharp 10
        //   - variable inside directory path
        //   - variable inside file name
        //   - variable same multiple usage
        //   - variable index out of range
        //   - slash_substitution
        // - substrs
        //   - empty

        static SEP: char = MAIN_SEPARATOR;

        fn default_substrs() -> Vec<String> {
            vec!["v1", "v2", "v3", "v4", "v5", "v6", "v7", "v8", "v9", "vX"]
                .iter()
                .map(|x| String::from(*x))
                .collect::<Vec<_>>()
        }

        #[test]
        fn dest_empty() {
            let dest = "";
            let substrs = default_substrs();
            assert_eq!(resolve(dest, &substrs[..]), String::from(""));
        }

        #[test]
        fn dest_no_vars() {
            let dest = "/foo/bar";
            let substrs = default_substrs();
            assert_eq!(resolve(dest, &substrs[..]), format!("{}foo{}bar", SEP, SEP));
        }

        #[test]
        fn dest_sharp() {
            let dest = "/foo/bar/#";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}#", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_0() {
            let dest = "/foo/bar/#0";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}#0", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_1() {
            let dest = "/foo/bar/#1";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}v1", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_9() {
            let dest = "/foo/bar/#9";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}v9", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_colon() {
            let dest = "/foo/bar/#:";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}#:", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_10() {
            let dest = "/foo/bar/#10";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}v10", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_in_dirname() {
            let dest = "/foo/#1/baz";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}v1{}baz", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_in_filename() {
            let dest = "/foo/bar/baz_#1.txt";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}bar{}baz_v1.txt", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_multi_usage() {
            let dest = "/foo/#3/#1#2.#9";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}v3{}v1v2.v9", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_index_out_of_range() {
            let dest = "/foo/#3/#1#2.txt";
            let substrs = vec!["v1"]
                .iter()
                .map(|x| String::from(*x))
                .collect::<Vec<_>>();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("{}foo{}#3{}v1#2.txt", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_slash_substitution() {
            let dest = "foo\\bar/baz";
            let substrs = default_substrs();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("foo{}bar{}baz", SEP, SEP)
            );
        }

        #[test]
        fn substrs_empty() {
            let dest = "foo/bar/baz";
            let substrs: Vec<String> = Vec::new();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("foo{}bar{}baz", SEP, SEP)
            );
        }

        #[test]
        fn substrs_one() {
            let dest = "foo/#1/baz";
            let substrs = vec!["v1"]
                .iter()
                .map(|x| String::from(*x))
                .collect::<Vec<_>>();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("foo{}v1{}baz", SEP, SEP)
            );
        }

        #[test]
        fn substrs_two() {
            let dest = "foo/#1/#2";
            let substrs = vec!["v1", "v2"]
                .iter()
                .map(|x| String::from(*x))
                .collect::<Vec<_>>();
            assert_eq!(
                resolve(dest, &substrs[..]),
                format!("foo{}v1{}v2", SEP, SEP)
            );
        }

        #[test]
        fn substrs_invalid_char() {
            let dest = "foo/#1/#2";
            let substrs = vec!["/", "/"]
                .iter()
                .map(|x| String::from(*x))
                .collect::<Vec<_>>();
            assert_eq!(resolve(dest, &substrs[..]), format!("foo{}/{}/", SEP, SEP));
        }
    }
}
