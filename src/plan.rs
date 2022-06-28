use crate::Action;
use rand::random;
use std::path::{Path, PathBuf, MAIN_SEPARATOR};

pub fn sort_actions(actions: &[Action]) -> Result<Vec<Action>, String> {
    let mut actions: Vec<&Action> = actions.iter().collect();
    let mut sorted: Vec<Action> = Vec::new();
    while !actions.is_empty() {
        // Pull a chain starting with the first actions.
        let mut indices = pull_a_chain(&actions)?;
        debug_assert!(!indices.is_empty());

        // Resolve if the end of the chain is the beginnig
        let is_circular = if 2 <= indices.len() {
            let first = actions[indices[0]];
            let last = actions[*indices.last().unwrap()];
            first.src() == last.dest
        } else {
            false
        };

        // Copy the pulled actions in reverse order so that they can be executed safely.
        if is_circular {
            // If the network graph forms a circle, insert a temporary node to break it
            // (for example, turn [C→A, B→C, A→B] into [C→X, B→C, A→B, X→A].)
            // To do that, firstly we resolve a temporary backup file name.
            let first = actions[indices[0]];
            let last = actions[*indices.last().unwrap()];
            let tmp = match make_safeish_filename(first.src()) {
                Some(path) => path,
                None => {
                    return Err(format!(
                        "temporary filename unavailable for {}",
                        first.src().to_string_lossy()
                    ))
                }
            };
            sorted.push(Action::new(last.src(), tmp.clone()));
            for i in indices.iter().rev().skip(1) {
                sorted.push(actions[*i].clone());
            }
            sorted.push(Action::new(tmp, first.src())); // moving "tmp"
        } else {
            for i in indices.iter().rev() {
                sorted.push(actions[*i].clone());
            }
        }

        // Now remove the pulled and copied actions from the source.
        indices.sort_unstable();
        for i in indices.iter().rev() {
            actions.remove(*i);
        }
    }

    Ok(sorted)
}

/// Makes a safe-ish filename which does not conflict with no other files.
///
/// This function is basically UNSAFE as it checks for an pre-existing files without creating a
/// file.
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
/// If two or more actions share a src, this function fails.
fn pull_a_chain(actions: &[&Action]) -> Result<Vec<usize>, String> {
    let mut indices: Vec<usize> = vec![];

    // If there is nothing to move, we are done.
    if actions.is_empty() {
        return Ok(indices);
    }

    // Remember the first action for later
    let _head = &actions[0];
    if let Some(a) = actions.iter().skip(1).find(|a| a.src() == _head.src()) {
        // Fail if there is another action of which src is the same
        return Err(format!(
            "cannot move a file to mutliple destinations: '{}' to '{}' and '{}'",
            _head.src().to_string_lossy(),
            _head.dest.to_string_lossy(),
            a.dest.to_string_lossy()
        ));
    }
    indices.push(0);

    loop {
        let prev_indices_len = indices.len();

        // Find an action which can be chained. (e.g.: B→C after A→B)
        for (i, action) in actions.iter().enumerate().skip(1) {
            debug_assert!(action.src().is_absolute());
            debug_assert!(action.dest.is_absolute());

            // Skip if this action cannot be as such.
            let curr = actions[*indices.last().unwrap()];
            if action.src() != curr.dest {
                continue;
            }

            // Fail if the src was shared with other actions.
            if let Some(a) = actions.iter().skip(i + 1).find(|a| a.src() == curr.dest) {
                return Err(format!(
                    "cannot move a file to mutliple destinations: '{}' to '{}' and '{}'",
                    action.src().to_string_lossy(),
                    action.dest.to_string_lossy(),
                    a.dest.to_string_lossy(),
                ));
            }

            // Remember this as a following action.
            indices.push(i);
            break;
        }

        // Exit if no chaining action was found.
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

    fn to_absolute(actions: Vec<Action>) -> Vec<Action> {
        let curdir = std::env::current_dir().unwrap();
        actions
            .iter()
            .map(|a| Action::new(curdir.join(&a.src()), curdir.join(&a.dest)))
            .collect()
    }

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
            let actions: Vec<&Action> = vec![];
            let indices = pull_a_chain(&actions);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices.len(), 0);
        }

        #[test]
        fn single() {
            let actions = to_absolute(vec![Action::new("A", "B")]);
            let actions: Vec<&Action> = actions.iter().collect();
            let indices = pull_a_chain(&actions);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices.len(), 1);
            assert_eq!(indices[0], 0);
        }

        #[test]
        fn chained() {
            let actions = to_absolute(vec![
                Action::new("A", "B"),
                Action::new("C", "X"),
                Action::new("B", "C"),
            ]);
            let actions: Vec<&Action> = actions.iter().collect();
            let indices = pull_a_chain(&actions);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices, vec![0, 2, 1]);
        }

        #[test]
        fn circular() {
            let actions = to_absolute(vec![
                Action::new("A", "B"),
                Action::new("C", "A"),
                Action::new("B", "C"),
            ]);
            let actions: Vec<&Action> = actions.iter().collect();
            let indices = pull_a_chain(&actions);
            assert!(indices.is_ok());
            let indices = indices.unwrap();
            assert_eq!(indices, vec![0, 2, 1]);
        }

        #[test]
        fn shared_src_1st() {
            let actions = to_absolute(vec![Action::new("A", "B"), Action::new("A", "C")]);
            let actions: Vec<&Action> = actions.iter().collect();
            let indices = pull_a_chain(&actions);
            assert!(indices.is_err());
            let msg = indices.unwrap_err();
            assert!(msg.contains("cannot move a file to mutliple destinations"));
            assert!(msg.contains("A' to"));
            assert!(msg.contains("B' and"));
            assert!(msg.ends_with("C'"));
        }

        #[test]
        fn shared_src_2nd() {
            let actions = to_absolute(vec![
                Action::new("A", "B"),
                Action::new("B", "C"),
                Action::new("B", "D"),
            ]);
            let actions: Vec<&Action> = actions.iter().collect();
            let indices = pull_a_chain(&actions);
            assert!(indices.is_err());
            let msg = indices.unwrap_err();
            assert!(msg.contains("cannot move a file to mutliple destinations"));
            assert!(msg.contains("B' to"));
            assert!(msg.contains("C' and"));
            assert!(msg.ends_with("D'"));
        }
    }

    mod sort_actions {
        use super::*;

        #[test]
        fn emtpy() {
            let actions: Vec<Action> = vec![];
            let sorted = sort_actions(&actions).unwrap();
            assert_eq!(sorted.len(), 0);
        }

        #[test]
        fn single() {
            let actions = to_absolute(vec![Action::new("A", "B")]);
            let sorted = sort_actions(&actions).unwrap();
            assert_eq!(sorted, to_absolute(vec![Action::new("A", "B")]));
        }

        #[test]
        fn chained() {
            let actions = to_absolute(vec![
                Action::new("A", "B"),
                Action::new("C", "X"),
                Action::new("B", "C"),
            ]);
            let sorted = sort_actions(&actions).unwrap();
            assert_eq!(
                sorted,
                to_absolute(vec![
                    Action::new("C", "X"),
                    Action::new("B", "C"),
                    Action::new("A", "B"),
                ])
            );
        }

        #[test]
        fn circular() {
            let actions = to_absolute(vec![
                Action::new("A", "B"),
                Action::new("C", "A"),
                Action::new("B", "C"),
            ]);
            let sorted = sort_actions(&actions).unwrap();
            let tmp = sorted[0].dest.to_str().unwrap();
            assert_eq!(
                sorted,
                to_absolute(vec![
                    Action::new("C", tmp),
                    Action::new("B", "C"),
                    Action::new("A", "B"),
                    Action::new(tmp, "A"),
                ])
            );
        }
    }
}
