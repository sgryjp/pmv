mod fsutil;

pub use fsutil::{fnmatch, walk};

use std::cmp;
use std::io;
use std::path::{Path, PathBuf};

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
            resolved.push(std::path::MAIN_SEPARATOR);
            i += 1;
        } else {
            resolved.push(dest[i] as char);
            i += 1;
        }
    }
    resolved
}

pub fn move_files(
    sources: &[PathBuf],
    destinations: &[String],
    dry_run: bool,
    verbose: bool,
    on_error: Option<&dyn Fn(&str, &str, &io::Error) -> ()>,
) -> i32 {
    let mut num_errors = 0;

    // Calculate max width for printing
    let src_max_len = sources
        .iter()
        .map(|x| x.to_str().unwrap().len())
        .fold(0, cmp::max);

    // Move files
    let mut line = String::new();
    for (src, dest) in sources.iter().zip(destinations.iter()) {
        // Reject if moving a directory to path where a file exists
        // (Windows accepts this case but Linux does not)
        if src.is_dir() && Path::new(&dest).is_file() {
            if let Some(f) = on_error {
                let err = io::Error::new(
                    io::ErrorKind::Other,
                    "overwriting a file with a directory is not allowed",
                );
                f(src.to_str().unwrap(), dest, &err);
            }
            num_errors += 1;
            continue;
        }

        // Append basename of src to dest if dest is a directory
        let mut dest = PathBuf::from(dest);
        if dest.is_dir() {
            dest.push(src.file_name().unwrap());
        }
        let dest = dest.to_str().unwrap();
        let src = src.to_str().unwrap();

        line.clear();
        line.push_str(src);
        for _ in src.len()..src_max_len {
            line.push(' ');
        }
        line.push_str(" --> "); //TODO: Wrap line if it's too long
        line.push_str(dest);
        if verbose || dry_run {
            println!("{}", line);
        }
        if !dry_run {
            if let Err(err) = std::fs::rename(src, dest) {
                if let Some(f) = on_error {
                    f(src, dest, &err);
                }
                num_errors += 1;
            }
        }
    }

    num_errors
}
