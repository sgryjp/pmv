use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

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

fn fnmatch(pattern: &str, name: &str) -> Option<Vec<String>> {
    println!("# fnmatch(pattern={:}, name={:})", pattern, name);
    let pattern = pattern.as_bytes();
    let name = name.as_bytes();
    let mut i: usize = 0;
    let mut j: usize = 0;
    let mut matches: Vec<String> = Vec::new();
    loop {
        let name_j = if j < name.len() { name[j] } else { '_' as u8 };
        println!(
            "# fnmatch(): pattern[{}]=\"{}\" name[{}]=\"{}\"",
            i, pattern[i] as char, j, name_j as char
        );
        match pattern[i] {
            b'?' => {
                if name.len() <= j {
                    return None; // no more chars available for this '?'
                }

                // Match one character
                matches.push(String::from_utf8(name[j..=j].to_vec()).unwrap());
                i += 1;
                j += 1;
            }
            b'*' => {
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
                    let mut k = j;
                    if i + 1 < pattern.len() {
                        let term = pattern[i + 1];
                        while name[k] != term {
                            k += 1;
                        }
                    } else {
                        k = name.len();
                    }
                    matches.push(String::from_utf8(name[j..k].to_vec()).unwrap());
                    i += 1;
                    j = k;
                }
            }
            c if c == name[j] => {
                i += 1;
                j += 1;
            }
            _ => return None,
        }

        if pattern.len() <= i {
            if name.len() == j {
                return Some(matches);
            } else {
                return None;
            }
        }
        if name.len() <= j {
            return None;
        }
    }
}

fn walk(
    dir: &Path,
    patterns: &[Component],
    matched_paths: &mut Vec<(fs::DirEntry, Vec<String>)>,
) -> io::Result<()> {
    println!("# walk(dir=\"{:?}\", patterns=\"{:?}\")", dir, patterns);

    if 1 < patterns.len() {
        match patterns[0] {
            Component::Prefix(p) => {
                // Reset the curdir to the path
                let curdir = p.as_os_str();
                let curdir = PathBuf::from(curdir);
                walk(&curdir, &patterns[1..], matched_paths)
            }
            Component::RootDir => {
                // Move to the root
                let root = MAIN_SEPARATOR.to_string();
                let root = PathBuf::from(root);
                walk(root.as_path(), &patterns[1..], matched_paths)
            }
            Component::ParentDir => {
                // Move to the parent
                let parent = dir.parent().unwrap(); //TODO: Handle error
                walk(parent, &patterns[1..], matched_paths)
            }
            Component::CurDir => {
                // Ignore the path component
                walk(dir, &patterns[1..], matched_paths)
            }
            Component::Normal(p) => {
                // Move into the matched sub-directories
                let dir = dir.join(p);
                let patterns = &patterns[1..];
                walk(dir.as_path(), patterns, matched_paths)
            }
        }
    } else if patterns.len() == 1 {
        match patterns[0] {
            Component::Prefix(_) => {
                // Move to the root
                panic!("Prefix is not supported") //TODO: Support
            }
            Component::RootDir => {
                // Move to the root
                panic!("RootDir is not supported") //TODO: Support
            }
            Component::ParentDir => {
                // Move to the parent
                let parent = dir.parent().unwrap(); //TODO: Handle error
                walk(parent, &patterns[1..], matched_paths)
            }
            Component::CurDir => {
                // Ignore the path component
                walk(dir, &patterns[1..], matched_paths)
            }
            Component::Normal(p) => {
                // Store the matched paths
                for result in fs::read_dir(dir)? {
                    let entry = result?;
                    let fname = entry.file_name();
                    let pattern = p.to_str().unwrap(); //TODO: Handle error
                    if let Some(matches) = fnmatch(&pattern, &fname.to_str().unwrap()) {
                        //TODO: Handle error
                        //println!("{:?} --> {:?}", entry, matches);
                        matched_paths.push((entry, matches));
                    }
                }
                Ok(())
            }
        }
    } else {
        Ok(())
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let src_ptns = &args[1];
    let dest_ptn: &str = &args[2];

    let src_ptns: Vec<_> = Path::new(&src_ptns).components().collect();
    let mut sources: Vec<(fs::DirEntry, Vec<String>)> = Vec::new();

    match walk(Path::new("."), src_ptns.as_slice(), &mut sources) {
        Err(e) => println!("Error: {:?}", e),
        Ok(_) => {
            println!("Ok: {:?}", sources);
            for (entry, matches) in sources {
                //println!("# {:?} {:?}", entry, matches);
                let dest_bytes = dest_ptn.as_bytes();
                let mut dest = String::new();
                let mut i = 0;
                while i < dest_bytes.len() {
                    if dest_bytes[i] == 0x5c // Backslash
                        && i + 1 < dest_bytes.len()
                        && 0x30 <= dest_bytes[i + 1] // 0
                        && dest_bytes[i + 1] <= 0x39
                    // 9
                    {
                        let index = (dest_bytes[i + 1] - 0x30 - 1) as usize;
                        let replacement = &matches[index];
                        dest.push_str(&replacement);
                        i += 2;
                    } else {
                        dest.push_str(&dest_ptn[i..=i]);
                        i += 1;
                    }
                }
                println!("{:?} --> {:?}", &entry, &PathBuf::from(dest));
                //std::fs::rename(&entry.path(), &PathBuf::from(dest));
            }
        }
    }
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
}
