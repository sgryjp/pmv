use std::{
    fs::{self, DirEntry},
    path::{Component, Path, PathBuf, MAIN_SEPARATOR},
};

mod fnmatch;
use fnmatch::fnmatch;

/// A directory entry found in a walk paired with pattern matched substrings.
///
/// This is a pair of a `std::fs::DirEntry` found while the walk and a vector
/// of the substrings.
pub struct Match {
    pub dir_entry: DirEntry,
    pub matched_parts: Vec<String>,
}

impl Match {
    pub fn path(&self) -> PathBuf {
        self.dir_entry.path()
    }
}

/// Returns the directory entries which matched the given pattern.
///
/// This function recursively search directory tree for entries matching the
/// given pattern. While this function walks the directory tree, it remembers
/// which part of the path corresponds to which wildcard in the pattern.
pub fn walk(dir: &Path, pattern: &str) -> Result<Vec<Match>, String> {
    let mut matches: Vec<Match> = Vec::new();
    let mut matched_parts: Vec<String> = Vec::new();
    let patterns: Vec<Component> = Path::new(pattern).components().collect();
    walk1(dir, &patterns[..], &mut matches, &mut matched_parts)?;
    Ok(matches)
}

pub fn walk1(
    dir: &Path,
    patterns: &[Component],
    matches: &mut Vec<Match>,
    matched_parts: &mut Vec<String>,
) -> Result<(), String> {
    assert!(dir.is_dir());
    assert!(!patterns.is_empty());

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
                // Acquire the entry
                let entry = match maybe_entry {
                    Err(err) => return Err(format!("failed to get a directory entry: {}", err)), //TODO: Test this
                    Ok(entry) => entry,
                };

                // Match its name
                let fname = entry.file_name();
                let pattern = pattern.to_str().unwrap();
                if let Some(mut m) = fnmatch(pattern, fname.to_str().unwrap()) {
                    // It matched, then query its metadata
                    let file_type = match entry.path().metadata() {
                        Err(err) => {
                            return Err(format!(
                                "failed to get metadata of {:?}: {}",
                                entry.path().to_str().unwrap_or("<UNKNOWN>"),
                                err
                            ))
                        }
                        Ok(v) => v.file_type(),
                    };

                    // Distinguish and switch procedure according to its type
                    let mut matched_parts = matched_parts.clone();
                    matched_parts.append(&mut m);
                    if file_type.is_dir() {
                        let subdir = dir.join(fname);
                        if 1 < patterns.len() {
                            // Walk into the found sub directory
                            let patterns_ = &patterns[1..];
                            walk1(subdir.as_path(), patterns_, matches, &mut matched_parts)?;
                        } else {
                            // Found a matched directory as a leaf; store the path
                            matches.push(Match {
                                dir_entry: entry,
                                matched_parts,
                            });
                        }
                    } else {
                        // Found a file; store the path only if it matched the last pattern (leaf)
                        if patterns.len() <= 1 {
                            matches.push(Match {
                                dir_entry: entry,
                                matched_parts: matched_parts.clone(),
                            });
                        }
                    }
                }
            }
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use function_name::named;

    mod walk {
        use super::*;

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

        fn new_setup(id: &str, prereq_dirs: Vec<&str>, prereq_files: Vec<&str>) -> PathBuf {
            // Prepare working directory
            let workdir = Path::join(Path::new("temp"), id);
            let _ = fs::remove_dir_all(workdir.as_path());
            fs::create_dir_all(workdir.as_path()).unwrap();

            // Create directories and files for the test
            for dirpath in prereq_dirs.iter() {
                fs::create_dir_all(Path::join(workdir.as_path(), dirpath)).unwrap();
            }
            for filepath in prereq_files.iter() {
                fs::write(Path::join(workdir.as_path(), filepath), filepath.as_bytes()).unwrap();
            }

            return workdir;
        }

        #[test]
        fn no_specials() {
            setup("no_specials");
            let matches = walk(Path::new("temp/no_specials"), "foo/bar/baz").unwrap();
            assert_eq!(matches.len(), 1);
            assert_eq!(matches[0].path(), Path::new("temp/no_specials/foo/bar/baz"));
            assert_eq!(matches[0].matched_parts, Vec::<String>::new());
        }

        #[test]
        fn question() {
            setup("question");
            let mut matches = walk(Path::new("temp/question"), "ba?/ba?/ba?").unwrap();
            assert_eq!(matches.len(), 8);
            matches.sort_by(|a, b| a.path().cmp(&b.path()));

            let paths: Vec<_> = matches.iter().map(|m| m.path()).collect();
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
                    x.matched_parts
                        .iter()
                        .fold("".to_string(), |acc, x| acc + "." + x)
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
            matches.sort_by(|a, b| a.path().cmp(&b.path()));

            let paths: Vec<_> = matches.iter().map(|x| x.path()).collect();
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
                    x.matched_parts
                        .iter()
                        .fold("".to_string(), |acc, x| acc + "." + x)
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

        #[named]
        #[test]
        fn issue17() {
            let prereq_dirs: Vec<&str> = vec![];
            let prereq_files = vec!["foo"];
            let workdir = new_setup(function_name!(), prereq_dirs, prereq_files);

            // pmv should not misrecognize "foo" as a directory
            walk(workdir.as_path(), "foo/bar").unwrap();
        }
    }
}
