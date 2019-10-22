use std::fs::{self, DirEntry};
use std::io;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

use crate::fnmatch;

/// Returns the directory entries which matched the given pattern.
///
/// This function recursively search directory tree for entries matching the
/// given pattern. While this function walks the directory tree, it remembers
/// which part of the path corresponds to which wildcard in the pattern.
/// For every matched entry this function finds, it creates a pair of an
/// `std::fs::DirEntry` for it and a vector of the substrings, collect them as
/// a vector, and return the vector.
pub fn walk(dir: &Path, pattern: &str) -> io::Result<Vec<(DirEntry, Vec<String>)>> {
    let mut matches: Vec<(DirEntry, Vec<String>)> = Vec::new();
    let mut matched_parts: Vec<String> = Vec::new();
    let patterns: Vec<Component> = Path::new(pattern).components().collect();
    match walk1(dir, &patterns[..], &mut matches, &mut matched_parts) {
        Ok(_) => Ok(matches),
        Err(err) => Err(err),
    }
}

pub fn walk1(
    dir: &Path,
    patterns: &[Component],
    matches: &mut Vec<(DirEntry, Vec<String>)>,
    matched_parts: &mut Vec<String>,
) -> io::Result<()> {
    if patterns.len() == 0 {
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
            for result in fs::read_dir(dir)? {
                let entry = result?;
                let fname = entry.file_name();
                let pattern = pattern.to_str().unwrap();
                if let Some(mut m) = fnmatch(pattern, fname.to_str().unwrap()) {
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
