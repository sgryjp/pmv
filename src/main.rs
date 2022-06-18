#[macro_use]
extern crate clap;
extern crate ansi_term;
extern crate atty;

use std::env;
use std::path::{Path, PathBuf};
use std::process::exit;

mod fsutil;
mod plan;
mod walk;
use fsutil::move_files;
use plan::substitute_variables;
use walk::walk;

#[derive(Debug)]
struct Config {
    src_ptn: String,
    dest_ptn: String,
    dry_run: bool,
    verbose: bool,
    interactive: bool,
}

/// A pair of source and destination in a moving plan.
struct Entry {
    src: PathBuf,
    dest: String,
}

/// Returns an object which will be rendered as colored string on terminal.
fn style_error(s: &str) -> ansi_term::ANSIString {
    if atty::is(atty::Stream::Stderr) {
        ansi_term::Color::Red.bold().paint(s)
    } else {
        ansi_term::ANSIGenericString::from(s) // LCOV_EXCL_LINE
    }
}

fn parse_args(args: env::ArgsOs) -> Config {
    let matches = clap::App::new("pmv")
        .version(crate_version!())
        .about(crate_description!())
        .setting(clap::AppSettings::ColorAuto)
        .setting(clap::AppSettings::ColoredHelp)
        .arg(
            clap::Arg::with_name("dry-run")
                .short("n")
                .long("dry-run")
                .help("Does not move files but just shows what would be done"),
        )
        .arg(
            clap::Arg::with_name("interactive")
                .short("i")
                .long("interactive")
                .help("Prompts before moving an each file"),
        )
        .arg(
            clap::Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Writes verbose message"),
        )
        .arg(
            clap::Arg::with_name("SOURCE")
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
            clap::Arg::with_name("DEST")
                .required(true)
                .index(2)
                .help("Destination pattern (use --help for details)")
                .long_help(
                    "A pattern string specifying where to move the targeted files. If the pattern \
                     contains tokens like `#1` or `#2`, each of them will be replaced with a \
                     substring extracted from the targeted file path. Those substrings matches \
                     the wildcard patterns in SOURCE; `#1` matches the first wildcard, `#2` \
                     matches the second, respectively. For example, if SOURCE is `*_test.py` and \
                     DEST is `tests/test_#1.py`:\n\n    \
                     Exisitng File | Destination\n    \
                     ------------- | -----------------\n    \
                     foo_test.py   | tests/test_foo.py\n    \
                     bar_test.py   | tests/test_bar.py\n    \
                     hoge_test.py  | tests/test_hoge.py",
                ),
        )
        .get_matches_from(args);

    let src_ptn = matches.value_of("SOURCE").unwrap().to_owned();
    let dest_ptn = matches.value_of("DEST").unwrap().to_owned();
    let dry_run = 0 < matches.occurrences_of("dry-run");
    let verbose = 0 < matches.occurrences_of("verbose");
    let interactive = 0 < matches.occurrences_of("interactive");

    Config {
        src_ptn,
        dest_ptn,
        dry_run,
        verbose,
        interactive,
    }
}

fn matches_to_entries(src_ptn: &str, dest_ptn: &str) -> Vec<Entry> {
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

    let mut entries = Vec::new();
    for m in matches {
        let ent = Entry {
            src: m.path(),
            dest: substitute_variables(dest_ptn, &m.matched_parts[..]),
        };
        entries.push(ent);
    }
    entries
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

fn main() {
    // Enable colored output
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();

    // Parse arguments
    let config = parse_args(env::args_os());

    // Collect paths of the files to move with their destination
    let entries = matches_to_entries(config.src_ptn.as_str(), config.dest_ptn.as_str());
    let sources: Vec<_> = entries.iter().map(|ent| ent.src.to_owned()).collect(); //TODO: Do not copy
    let destinations: Vec<_> = entries.iter().map(|ent| ent.dest.to_owned()).collect(); //TODO: Do not copy

    // Validate them
    if let Err(err) = validate(&sources, &destinations) {
        eprintln!("{}: {}", style_error("error"), err);
        exit(1);
    }

    // Move files
    move_files(
        &sources,
        &destinations,
        config.dry_run,
        config.interactive,
        config.verbose,
        Some(&|src, _dest, err| {
            eprintln!(
                "{}: failed to copy \"{}\": {}",
                style_error("error"),
                src,
                err
            );
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    mod matches_to_entries {
        use super::*;

        #[test]
        fn no_match() {
            let entries = matches_to_entries("zzzzz", "zzzzz");
            assert_eq!(entries.len(), 0);
        }

        #[test]
        fn multiple_matches() {
            let entries = matches_to_entries("Cargo.*", "Foobar.#1");
            assert_eq!(entries.len(), 2);
            assert_eq!(
                entries[0].src.file_name().unwrap(),
                PathBuf::from("Cargo.toml")
            );
            assert_eq!(
                entries[1].src.file_name().unwrap(),
                PathBuf::from("Cargo.lock")
            );
            assert_eq!(
                PathBuf::from(&entries[0].dest).file_name().unwrap(),
                PathBuf::from("Foobar.toml")
            );
            assert_eq!(
                PathBuf::from(&entries[1].dest).file_name().unwrap(),
                PathBuf::from("Foobar.lock")
            );
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
}
