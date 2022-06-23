use crate::Entry;
use rand::random;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

pub fn sort_entries(entries: &[Entry]) -> Result<Vec<Entry>, String> {
    let mut entries: Vec<&Entry> = entries.iter().collect();
    let mut sorted: Vec<Entry> = Vec::new();
    while !entries.is_empty() {
        // Pull a chain starting with the first entry.
        let mut indices = pull_a_chain(&entries)?;
        debug_assert!(!indices.is_empty());

        // Resolve if the end of the chain is the beginnig
        let is_circular = if 2 <= indices.len() {
            let first = entries[indices[0]];
            let last = entries[*indices.last().unwrap()];
            first.src == last.dest
        } else {
            false
        };

        // Copy the pulled entries in reverse order so that they can be executed safely.
        if is_circular {
            // If the moving plan makes a circle, insert a temporary node to break it
            // (for example, turn [C→A, B→C, A→B] into [C→X, B→C, A→B, X→A].)
            // To do that, firstly we resolve a temporary backup file name.
            let first = entries[indices[0]];
            let last = entries[*indices.last().unwrap()];
            let tmp = match make_safeish_filename(&first.src) {
                Some(path) => path,
                None => {
                    return Err(format!(
                        "temporary filename unavailable for {}",
                        first.src.to_string_lossy()
                    ))
                }
            };
            sorted.push(Entry {
                src: last.src.clone(),
                dest: tmp.clone(),
            });
            for i in indices.iter().rev().skip(1) {
                sorted.push(entries[*i].clone());
            }
            sorted.push(Entry {
                src: tmp, // move
                dest: first.src.clone(),
            });
        } else {
            for i in indices.iter().rev() {
                sorted.push(entries[*i].clone());
            }
        }

        // Now remove the pulled and copied entries from the source.
        indices.sort_unstable();
        for i in indices.iter().rev() {
            entries.remove(*i);
        }
    }

    Ok(sorted)
}

fn make_safeish_filename<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let orig_path = path.as_ref();
    let orig_path_str = orig_path.as_os_str();

    // Search for a safe-ish filename with random postfix
    let n: u16 = random();
    for i in (n..65535).chain(0..n) {
        let mut new_path_str = orig_path_str.to_owned();
        new_path_str.push(format!(".pmv{:04x}", i));
        let new_path = Path::new(&new_path_str);
        if !new_path.exists() {
            return Some(new_path_str.into()); // move
        }
    }

    // No filename was available.
    None
}

/// Enumerates a chain of moving actions which must be done in reversed order.
///
/// This function does not detect circular network so the caller must take care of that case.
/// If two or more entries shares a src, this function fails.
fn pull_a_chain(entries: &[&Entry]) -> Result<Vec<usize>, String> {
    let mut indices: Vec<usize> = vec![];

    // If there is nothing to move, we are done.
    if entries.is_empty() {
        return Ok(indices);
    }

    // Remember the first entry for later
    let _head = &entries[0];
    if let Some(e) = entries.iter().skip(1).find(|e| e.src == _head.src) {
        // Fail if there is another entry of which src is the same
        return Err(format!(
            "cannot move a file to mutliple destinations: '{}' to '{}' and '{}'",
            _head.src.to_string_lossy(),
            _head.dest.to_string_lossy(),
            e.dest.to_string_lossy()
        ));
    }
    indices.push(0);

    loop {
        let prev_indices_len = indices.len();

        // Find an entry which can be chained. (e.g.: B→C after A→B)
        for (i, entry) in entries.iter().enumerate().skip(1) {
            // Skip if this entry cannot be as such.
            let curr = entries[*indices.last().unwrap()];
            if entry.src != curr.dest {
                continue;
            }

            // Fail if the src was shared with other entries.
            if let Some(e) = entries.iter().skip(i + 1).find(|e| e.src == curr.dest) {
                return Err(format!(
                    "cannot move a file to mutliple destinations: '{}' to '{}' and '{}'",
                    entry.src.to_string_lossy(),
                    entry.dest.to_string_lossy(),
                    e.dest.to_string_lossy(),
                ));
            }

            // Remember this as a following entry.
            indices.push(i);
            break;
        }

        // Exit if no chaining entry was found.
        if indices.len() == prev_indices_len {
            break;
        }
    }

    Ok(indices)
}

/// Substitute variables with substrings.
///
/// This function replaces every variable notations `#n` in `dest` with
/// `substrings[n-1]` (e.g.: `#2` will be replaced with the second element in
/// `substrings`).
///
/// Note that up to 9 variables (i.e.: `#1` to `#9`) are supported.
pub fn substitute_variables(dest: &str, substrings: &[String]) -> String {
    let dest = dest.as_bytes();
    let mut substituted = String::new();
    let mut i = 0;
    while i < dest.len() {
        if dest[i] == b'#' && i + 1 < dest.len() && b'1' <= dest[i + 1] && dest[i + 1] <= b'9' {
            let index = (dest[i + 1] - b'1') as usize;
            let replacement = match substrings.get(index) {
                Some(s) => s,
                None => {
                    substituted.push('#');
                    substituted.push(dest[i + 1] as char);
                    i += 2;
                    continue;
                }
            };
            substituted.push_str(replacement);
            i += 2;
        } else if dest[i] == b'\\' || dest[i] == b'/' {
            substituted.push(MAIN_SEPARATOR);
            i += 1;
        } else {
            substituted.push(dest[i] as char);
            i += 1;
        }
    }
    substituted
}

#[cfg(test)]
mod tests {
    use super::*;

    mod substitute_variables {
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
            assert_eq!(substitute_variables(dest, &substrs[..]), String::from(""));
        }

        #[test]
        fn dest_no_vars() {
            let dest = "/foo/bar";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar", SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp() {
            let dest = "/foo/bar/#";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}#", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_0() {
            let dest = "/foo/bar/#0";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}#0", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_1() {
            let dest = "/foo/bar/#1";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}v1", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_9() {
            let dest = "/foo/bar/#9";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}v9", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_colon() {
            let dest = "/foo/bar/#:";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}#:", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_sharp_10() {
            let dest = "/foo/bar/#10";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}v10", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_in_dirname() {
            let dest = "/foo/#1/baz";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}v1{}baz", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_in_filename() {
            let dest = "/foo/bar/baz_#1.txt";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}bar{}baz_v1.txt", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_var_multi_usage() {
            let dest = "/foo/#3/#1#2.#9";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
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
                substitute_variables(dest, &substrs[..]),
                format!("{}foo{}#3{}v1#2.txt", SEP, SEP, SEP)
            );
        }

        #[test]
        fn dest_slash_substitution() {
            let dest = "foo\\bar/baz";
            let substrs = default_substrs();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("foo{}bar{}baz", SEP, SEP)
            );
        }

        #[test]
        fn substrs_empty() {
            let dest = "foo/bar/baz";
            let substrs: Vec<String> = Vec::new();
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
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
                substitute_variables(dest, &substrs[..]),
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
                substitute_variables(dest, &substrs[..]),
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
            assert_eq!(
                substitute_variables(dest, &substrs[..]),
                format!("foo{}/{}/", SEP, SEP)
            );
        }
    }

    mod pull_a_chain {
        use super::*;

        #[test]
        fn empty() {
            let entries: Vec<&Entry> = vec![];
            let indices = pull_a_chain(&entries);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices.len(), 0);
        }

        #[test]
        fn single() {
            let entries = vec![Entry::from_str("A", "B")];
            let entries: Vec<&Entry> = entries.iter().collect();
            let indices = pull_a_chain(&entries);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices.len(), 1);
            assert_eq!(indices[0], 0);
        }

        #[test]
        fn chained() {
            let entries = vec![
                Entry::from_str("A", "B"),
                Entry::from_str("C", "X"),
                Entry::from_str("B", "C"),
            ];
            let entries: Vec<&Entry> = entries.iter().collect();
            let indices = pull_a_chain(&entries);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices, vec![0, 2, 1]);
        }

        #[test]
        fn circular() {
            let entries = vec![
                Entry::from_str("A", "B"),
                Entry::from_str("C", "A"),
                Entry::from_str("B", "C"),
            ];
            let entries: Vec<&Entry> = entries.iter().collect();
            let indices = pull_a_chain(&entries);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices, vec![0, 2, 1]);
        }

        #[test]
        fn shared_src_1st() {
            let entries = vec![Entry::from_str("A", "B"), Entry::from_str("A", "C")];
            let entries: Vec<&Entry> = entries.iter().collect();
            let indices = pull_a_chain(&entries);
            assert!(indices.is_err());
            let msg = indices.unwrap_err();
            assert!(msg.contains("'A' to 'B' and 'C'"));
        }

        #[test]
        fn shared_src_2nd() {
            let entries = vec![
                Entry::from_str("A", "B"),
                Entry::from_str("B", "C"),
                Entry::from_str("B", "D"),
            ];
            let entries: Vec<&Entry> = entries.iter().collect();
            let indices = pull_a_chain(&entries);
            assert!(indices.is_err());
            let msg = indices.unwrap_err();
            assert!(msg.contains("'B' to 'C' and 'D'"));
        }
    }

    mod sort_entries {
        use super::*;

        #[test]
        fn emtpy() {
            let entries: Vec<Entry> = vec![];
            let sorted = sort_entries(&entries).unwrap();
            assert_eq!(sorted.len(), 0);
        }

        #[test]
        fn single() {
            let entries = vec![Entry::from_str("A", "B")];
            let sorted = sort_entries(&entries).unwrap();
            assert_eq!(sorted, vec![Entry::from_str("A", "B")]);
        }

        #[test]
        fn chained() {
            let entries = vec![
                Entry::from_str("A", "B"),
                Entry::from_str("C", "X"),
                Entry::from_str("B", "C"),
            ];
            let sorted = sort_entries(&entries).unwrap();
            assert_eq!(
                sorted,
                vec![
                    Entry::from_str("C", "X"),
                    Entry::from_str("B", "C"),
                    Entry::from_str("A", "B"),
                ]
            );
        }

        #[test]
        fn circular() {
            let entries = vec![
                Entry::from_str("A", "B"),
                Entry::from_str("C", "A"),
                Entry::from_str("B", "C"),
            ];
            let sorted = sort_entries(&entries).unwrap();
            let tmp = sorted[0].dest.to_str().unwrap();
            assert_eq!(
                sorted,
                vec![
                    Entry::from_str("C", tmp),
                    Entry::from_str("B", "C"),
                    Entry::from_str("A", "B"),
                    Entry::from_str(tmp, "A"),
                ]
            );
        }
    }
}
