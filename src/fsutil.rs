use std::cmp;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

pub fn move_files(
    sources: &[PathBuf],
    destinations: &[String],
    dry_run: bool,
    interactive: bool,
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
        if dry_run || (verbose && !interactive) {
            println!("{}", line);
        } else if interactive {
            // Ask user to proceed or not
            print!("{} ... ok? [y/N]: ", line);
            let _ = io::stdout().lock().flush();
            let mut line = String::new();
            let nbytes_read = io::stdin().read_line(&mut line).unwrap_or(0);
            if nbytes_read == 0 {
                if let Some(f) = on_error {
                    let err = io::Error::new(io::ErrorKind::Other, "error on reading user input");
                    f(src, dest, &err);
                }
                num_errors += 1;
                continue;
            }

            // Skip if the input was not "y"
            let line = line.trim();
            if line.to_ascii_lowercase() != "y" {
                continue;
            }
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
    use super::*;

    mod move_files {
        use super::*;

        use function_name::named;
        use std::fs;
        #[cfg(unix)]
        use std::os;

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

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
            let num_errors = move_files(&sources, &dests, dry_run, false, false, None);

            assert_eq!(num_errors, 0);
            assert!(!mkpathbuf(id, "ld1").exists());
            assert!(mkpathbuf(id, "d2/ld1").exists());
            assert!(mkpathbuf(id, "ld2/ld1").exists());
        }
    }
}
