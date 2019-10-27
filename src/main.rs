#[macro_use]
extern crate clap;
extern crate ansi_term;
use clap::{App, Arg};

use std::cmp;
use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;

use pmv::walk;

fn style_error(s: &str) -> ansi_term::ANSIString {
    ansi_term::Color::Red.bold().paint(s)
}

/// Replaces variables in the given destination path string using the given
/// substrings.
fn replace(dest_ptn: &str, substrings: &[String]) -> String {
    let dest = dest_ptn.as_bytes();
    let mut replaced = String::new();
    let mut i = 0;
    while i < dest.len() {
        if dest[i] == b'#' && i + 1 < dest.len() && b'1' <= dest[i + 1] && dest[i + 1] <= b'9' {
            let index = (dest[i + 1] - b'1') as usize;
            let replacement = &substrings[index]; //TODO: Index out of range
            replaced.push_str(replacement);
            i += 2;
        } else if dest[i] == b'\\' || dest[i] == b'/' {
            replaced.push(std::path::MAIN_SEPARATOR);
            i += 1;
        } else {
            replaced.push(dest[i] as char);
            i += 1;
        }
    }
    replaced
}

fn validate(sources: &[PathBuf], destinations: &[String]) -> Result<(), String> {
    // Ensure that no files share a same destination path
    let mut sorted: Vec<_> = destinations.iter().enumerate().collect();
    sorted.sort_by(|a, b| a.1.cmp(&b.1));
    for i in 1..sorted.len() {
        let p1 = &sorted[i - 1];
        let p2 = &sorted[i];
        if p1.1 == p2.1 {
            return Err(format!(
                "destination must be different for each file: \
                 tried to move both \"{}\" and \"{}\" to \"{}\"",
                sources[p1.0].to_str().unwrap(),
                sources[p2.0].to_str().unwrap(),
                destinations[p1.0],
            ));
        }
    }

    Ok(())
}

fn move_files(sources: &[PathBuf], destinations: &[String], dry_run: bool) -> i32 {
    let mut num_errors = 0;

    // Calculate max width for printing
    let src_max_len = sources
        .iter()
        .map(|x| x.to_str().unwrap().len())
        .fold(0, cmp::max);

    // Move files
    let mut line = String::new();
    for (src, dest) in sources.iter().zip(destinations.iter()) {
        let src = src.to_str().unwrap();

        line.clear();
        line.push_str(src);
        for _ in src.len()..src_max_len {
            line.push(' ');
        }
        line.push_str(" --> "); //TODO: Wrap line if it's too long
        line.push_str(dest);
        println!("{}", line);
        if !dry_run {
            if let Err(err) = std::fs::rename(src, dest) {
                eprintln!(
                    "{}: failed to copy \"{}\": {}",
                    style_error("error"),
                    src,
                    err
                );
                num_errors += 1;
            }
        }
    }

    num_errors
}

fn main() {
    // Enable colored output
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();

    // Parse arguments
    let matches = App::new("pmv")
        .version(crate_version!())
        .about(crate_description!())
        .arg(
            Arg::with_name("dry-run")
                .short("n")
                .long("dry-run")
                .help("Do not actually move the files, just show what would be done."),
        )
        .arg(
            Arg::with_name("SOURCE")
                .required(true)
                .index(1)
                .help("Source pattern (use --help for details)")
                .long_help(
                    "A pattern string specifying files to move. If the pattern contains \
                     wildcard(s), multiple files matching to the pattern will be targeted. \
                     Supported wildcards are:\n\n    \
                     ? ... Matches a single character\n    \
                     * ... Matches zero or more characters",
                ),
        )
        .arg(
            Arg::with_name("DEST")
                .required(true)
                .index(2)
                .help("Destination pattern (use --help for details)")
                .long_help(
                    "A pattern string specifying where to move the targeted files. If the pattern \
                     contains tokens like `#1` or `#2`, each of them will be replaced with a \
                     substring extracted from the targeted file path. Those substrings matches \
                     the wildcard patterns in SOURCE; `#1` matches the first wildcard, `#2` \
                     matches the second wildcard, respectively. For example, if SOURCE is \
                     `*_test.py` and DEST is `tests/test_#1.py`:\n\n    \
                     Exisitng File | Destination\n    \
                     ------------- | -----------------\n    \
                     foo_test.py   | tests/test_foo.py\n    \
                     bar_test.py   | tests/test_bar.py\n    \
                     hoge_test.py  | tests/test_hoge.py",
                ),
        )
        .get_matches_from(env::args_os());
    let src_ptn = matches.value_of("SOURCE").unwrap();
    let dest_ptn = matches.value_of("DEST").unwrap();
    let dry_run = 0 < matches.occurrences_of("dry-run");

    // Gather source and destination paths
    let matches = match walk(Path::new("."), src_ptn) {
        Err(err) => {
            eprintln!(
                "{}: failed to scan directory tree: {}",
                style_error("error"),
                err
            );
            exit(2);
        }
        Ok(matches) => matches,
    };
    let destinations: Vec<_> = matches
        .iter()
        .map(|x| replace(dest_ptn, &x.1[..]))
        .collect();
    let sources: Vec<_> = matches.iter().map(|x| x.0.path()).collect();
    assert_eq!(sources.len(), destinations.len());

    // Validate them
    if let Err(err) = validate(&sources, &destinations) {
        eprintln!("{}: {}", style_error("error"), err);
        exit(1);
    }

    // Move files
    move_files(&sources, &destinations, dry_run);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn setup(id: &str) {
        let _ = fs::create_dir(Path::new("temp"));
        let _ = fs::remove_dir_all(Path::new(&format!("temp/{}", id)));
        for dir1 in ["foo", "bar", "baz"].iter() {
            for dir2 in ["foo", "bar", "baz"].iter() {
                let _ = fs::create_dir_all(Path::new(&format!("temp/{}/{}/{}", id, dir1, dir2)));
                for fname in ["foo", "bar", "baz"].iter() {
                    let path: String = format!("temp/{}/{}/{}/{}", id, dir1, dir2, fname);
                    fs::write(Path::new(&path), path.as_bytes()).unwrap();
                }
            }
        }
    }

    #[test]
    fn test_validation_ok() {
        let sources: Vec<PathBuf> = vec!["src/foo.rs"].iter().map(PathBuf::from).collect();
        let destinations: Vec<_> = vec![String::from("src/foo")];
        let result = validate(&sources, &destinations);
        result.unwrap();
    }

    #[test]
    fn test_validation_duplicated_dest() {
        let sources: Vec<PathBuf> = vec!["src/foo.rs", "src/bar.rs"]
            .iter()
            .map(PathBuf::from)
            .collect();
        let destinations: Vec<String> = vec!["src/foo.rs", "src/foo.rs"]
            .iter()
            .map(|x| String::from(*x))
            .collect();
        let result = validate(&sources, &destinations);
        let err = result.unwrap_err();
        assert!(err.contains("destination must be different for each file"));
        assert!(err.contains("src/foo.rs"));
    }

    #[test]
    fn test_move_files_ok() {
        let id = "test_move_files_ok";
        setup(id);

        let sources: Vec<PathBuf> = vec![
            format!("temp/{}/foo/foo/foo", id),
            format!("temp/{}/foo/bar/foo", id),
            format!("temp/{}/foo/baz/foo", id),
        ]
        .iter()
        .map(PathBuf::from)
        .collect();
        let dests: Vec<String> = vec![
            format!("temp/{}/foo/foo/zzz", id),
            format!("temp/{}/foo/bar/zzz", id),
            format!("temp/{}/foo/baz/zzz", id),
        ]
        .iter()
        .map(|x| String::from(x))
        .collect();
        let dry_run = false;
        let num_errors = move_files(&sources, &dests, dry_run);

        assert!(!sources[0].exists());
        assert!(!sources[1].exists());
        assert!(!sources[2].exists());
        assert!(Path::new(&dests[0]).exists());
        assert!(Path::new(&dests[1]).exists());
        assert!(Path::new(&dests[2]).exists());
        assert_eq!(
            fs::read_to_string(Path::new(&dests[0])).unwrap(),
            sources[0].to_str().unwrap(),
        );
        assert_eq!(
            fs::read_to_string(Path::new(&dests[1])).unwrap(),
            sources[1].to_str().unwrap(),
        );
        assert_eq!(
            fs::read_to_string(Path::new(&dests[2])).unwrap(),
            sources[2].to_str().unwrap(),
        );

        assert_eq!(num_errors, 0);
    }

    #[test]
    fn test_move_files_dry_run() {
        let id = "test_move_files_dry_run";
        setup(id);

        let sources: Vec<PathBuf> = vec![
            format!("temp/{}/foo/foo/foo", id),
            format!("temp/{}/foo/bar/foo", id),
            format!("temp/{}/foo/baz/foo", id),
        ]
        .iter()
        .map(PathBuf::from)
        .collect();
        let dests: Vec<String> = vec![
            format!("temp/{}/foo/foo/zzz", id),
            format!("temp/{}/foo/bar/zzz", id),
            format!("temp/{}/foo/baz/zzz", id),
        ]
        .iter()
        .map(|x| String::from(x))
        .collect();
        let dry_run = true;
        let num_errors = move_files(&sources, &dests, dry_run);

        assert!(sources[0].exists());
        assert!(sources[1].exists());
        assert!(sources[2].exists());
        assert!(!Path::new(&dests[0]).exists());
        assert!(!Path::new(&dests[1]).exists());
        assert!(!Path::new(&dests[2]).exists());

        assert_eq!(num_errors, 0);
    }

    #[test]
    fn test_move_files_invalid_dest() {
        let id = "test_move_files_invalid_dest";
        setup(id);

        let sources: Vec<PathBuf> = vec![
            format!("temp/{}/foo/foo/foo", id),
            format!("temp/{}/foo/bar/foo", id),
            format!("temp/{}/foo/baz/foo", id),
        ]
        .iter()
        .map(PathBuf::from)
        .collect();
        let dests: Vec<String> = vec![
            format!("temp/{}/foo/foo/\0", id),
            format!("temp/{}/foo/bar/\0", id),
            format!("temp/{}/foo/baz/\0", id),
        ]
        .iter()
        .map(|x| String::from(x))
        .collect();
        let dry_run = false;
        let num_errors = move_files(&sources, &dests, dry_run);

        assert!(sources[0].exists());
        assert!(sources[1].exists());
        assert!(sources[2].exists());
        assert!(!Path::new(&dests[0]).exists());
        assert!(!Path::new(&dests[1]).exists());
        assert!(!Path::new(&dests[2]).exists());

        assert_eq!(num_errors, 3);
    }
}
