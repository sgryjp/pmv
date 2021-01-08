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

#[cfg(test)]
mod tests {
    use function_name::named;
    use std::fs;
    use super::*;

    mod resolve {
        use super::*;
        use std::path::MAIN_SEPARATOR;

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

    mod move_files {
        use super::*;

        #[cfg(unix)]
        use std::os;

        use super::move_files;

        fn prepare_test(id: &str) -> Result<(), io::Error> {
            let _ = fs::create_dir("temp");
            let path = format!("temp/{}", id);
            if Path::new(&path).exists() {
                fs::remove_dir_all(Path::new(&path)).unwrap();
            }
            fs::create_dir(Path::new(&path))
        }

        fn mkpathstring(id: &str, name: &str) -> String {
            format!("temp/{}/{}", id, name)
        }

        fn mkpathbuf(id: &str, name: &str) -> PathBuf {
            let path = mkpathstring(id, name);
            PathBuf::from(&path)
        }

        fn mkfile(id: &str, name: &str) -> Result<(), io::Error> {
            let path = mkpathstring(id, name);
            fs::write(Path::new(&path), &path)
        }

        fn mkdir(id: &str, name: &str) -> Result<(), io::Error> {
            let path = mkpathstring(id, name);
            fs::create_dir(Path::new(&path))
        }

        #[cfg(unix)]
        fn mklink(id: &str, src: &str, dest: &str) -> Result<(), io::Error> {
            let dest = mkpathstring(id, dest);
            let src = PathBuf::from(mkpathstring(id, src));
            let src = src.canonicalize().unwrap();
            os::unix::fs::symlink(src, dest)
        }

        fn content_of(id: &str, name: &str) -> String {
            let path = mkpathstring(id, name);
            fs::read_to_string(Path::new(&path)).unwrap()
        }

        #[named]
        #[test]
        fn dry_run() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mkfile(id, "f2").unwrap();

            let dry_run = true;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "f2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(mkpathbuf(id, "f1").exists());
            assert!(mkpathbuf(id, "f2").exists());
            assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
            assert_eq!(content_of(id, "f2"), format!("temp/{}/f2", id));
        }

        #[named]
        #[test]
        fn invalid_dest() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "\0")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 1);
            assert!(mkpathbuf(id, "f1").exists());
            assert!(!mkpathbuf(id, "\0").exists());
            assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
        }

        #[named]
        #[test]
        fn file_to_file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mkfile(id, "f2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "f2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "f1").exists());
            assert!(mkpathbuf(id, "f2").exists());
            assert_eq!(content_of(id, "f2"), format!("temp/{}/f1", id));
        }

        #[named]
        #[test]
        fn file_to_dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mkdir(id, "d1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "d1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "f1").exists());
            assert!(mkpathbuf(id, "d1/f1").exists());
            assert_eq!(content_of(id, "d1/f1"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn file_to_symlink2file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "lf1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "f1").exists());
            assert!(mkpathbuf(id, "lf1").exists());
            assert_eq!(content_of(id, "lf1"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn file_to_symlink2dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
            let dests: Vec<String> = vec![mkpathstring(id, "ld1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "f1").exists());
            assert!(mkpathbuf(id, "d1/f1").is_file());
            assert!(mkpathbuf(id, "ld1/f1").is_file());
            assert_eq!(content_of(id, "ld1/f1"), format!("temp/{}/f1", id));
        }

        #[named]
        #[test]
        fn dir_to_file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mkfile(id, "f1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
            let dests: Vec<String> = vec![mkpathstring(id, "f1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 1);
            assert!(mkpathbuf(id, "d1").exists());
            assert!(mkpathbuf(id, "f1").exists());
            assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
        }

        #[named]
        #[test]
        fn dir_to_dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mkdir(id, "d2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
            let dests: Vec<String> = vec![mkpathstring(id, "d2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "d1").exists());
            assert!(mkpathbuf(id, "d2").exists());
            assert!(mkpathbuf(id, "d2/d1").exists());
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn dir_to_symlink2file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
            let dests: Vec<String> = vec![mkpathstring(id, "lf1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 1);
            assert!(mkpathbuf(id, "d1").is_dir());
            assert!(mkpathbuf(id, "f1").is_file());
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn dir_to_symlink2dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mkdir(id, "d2").unwrap();
            mklink(id, "d2", "ld2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
            let dests: Vec<String> = vec![mkpathstring(id, "ld2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "d1").exists());
            assert!(mkpathbuf(id, "d2/d1").is_dir());
            assert!(mkpathbuf(id, "ld2/d1").is_dir());
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2file_to_file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();
            mkfile(id, "f2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "lf1")];
            let dests: Vec<String> = vec![mkpathstring(id, "f2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "lf1").is_file());
            assert!(mkpathbuf(id, "f2").exists());
            assert_eq!(content_of(id, "f2"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2file_to_dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();
            mkdir(id, "d1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "lf1")];
            let dests: Vec<String> = vec![mkpathstring(id, "d1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "lf1").exists());
            assert!(mkpathbuf(id, "d1/lf1").exists());
            assert_eq!(content_of(id, "d1/lf1"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2file_to_symlink2file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();
            mkfile(id, "f2").unwrap();
            mklink(id, "f2", "lf2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "lf1")];
            let dests: Vec<String> = vec![mkpathstring(id, "lf2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "lf1").exists());
            assert!(mkpathbuf(id, "lf2").exists());
            assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
            assert_eq!(content_of(id, "f2"), format!("temp/{}/f2", id));
            assert_eq!(content_of(id, "lf2"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2file_to_symlink2dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "lf1")];
            let dests: Vec<String> = vec![mkpathstring(id, "ld1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "lf1").exists());
            assert!(mkpathbuf(id, "d1/lf1").is_file());
            assert!(mkpathbuf(id, "ld1/lf1").is_file());
            assert_eq!(content_of(id, "ld1/lf1"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2dir_to_file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();
            mkfile(id, "f1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "ld1")];
            let dests: Vec<String> = vec![mkpathstring(id, "f1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 1);
            assert!(mkpathbuf(id, "ld1").exists());
            assert!(mkpathbuf(id, "f1").exists());
            assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2dir_to_dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();
            mkdir(id, "d2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "ld1")];
            let dests: Vec<String> = vec![mkpathstring(id, "d2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "ld1").exists());
            assert!(mkpathbuf(id, "d2").exists());
            assert!(mkpathbuf(id, "d2/ld1").exists());
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2dir_to_symlink2file() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();
            mkfile(id, "f1").unwrap();
            mklink(id, "f1", "lf1").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "ld1")];
            let dests: Vec<String> = vec![mkpathstring(id, "lf1")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 1);
            assert!(mkpathbuf(id, "ld1").exists());
            assert!(mkpathbuf(id, "lf1").exists());
        }

        #[cfg(unix)]
        #[named]
        #[test]
        fn symlink2dir_to_symlink2dir() {
            let id = function_name!();

            prepare_test(id).unwrap();
            mkdir(id, "d1").unwrap();
            mklink(id, "d1", "ld1").unwrap();
            mkdir(id, "d2").unwrap();
            mklink(id, "d2", "ld2").unwrap();

            let dry_run = false;
            let sources: Vec<PathBuf> = vec![mkpathbuf(id, "ld1")];
            let dests: Vec<String> = vec![mkpathstring(id, "ld2")];
            let num_errors = move_files(&sources, &dests, dry_run, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "ld1").exists());
            assert!(mkpathbuf(id, "d2/ld1").exists());
            assert!(mkpathbuf(id, "ld2/ld1").exists());
        }
    }
}
