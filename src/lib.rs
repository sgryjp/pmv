mod action;
mod fnmatch;
mod fsutil;
mod plan;
mod walk;

use action::Action;
use fsutil::move_files;
use plan::sort_actions;
use plan::substitute_variables;
use std::ffi::OsString;
use std::process::exit;
use walk::walk;

#[derive(Debug)]
struct Config {
    src_ptn: String,
    dest_ptn: String,
    dry_run: bool,
    verbose: bool,
    interactive: bool,
}

/// Returns an object which will be rendered as colored string on terminal.
pub fn style_error(s: &str) -> ansi_term::ANSIString {
    if atty::is(atty::Stream::Stderr) {
        ansi_term::Color::Red.bold().paint(s)
    } else {
        ansi_term::ANSIGenericString::from(s) // LCOV_EXCL_LINE
    }
}

fn parse_args(args: &[OsString]) -> Config {
    let matches = clap::Command::new("pmv")
        .version(clap::crate_version!())
        .about(clap::crate_description!())
        .arg(
            clap::Arg::new("dry-run")
                .short('n')
                .long("dry-run")
                .action(clap::builder::ArgAction::SetTrue)
                .help("Does not move files but just shows what would be done"),
        )
        .arg(
            clap::Arg::new("interactive")
                .short('i')
                .long("interactive")
                .action(clap::builder::ArgAction::SetTrue)
                .help("Prompts before moving an each file"),
        )
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(clap::builder::ArgAction::Count)
                .help("Writes verbose message"),
        )
        .arg(
            clap::Arg::new("SOURCE")
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
            clap::Arg::new("DEST")
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

    let src_ptn = matches.get_one::<String>("SOURCE").unwrap();
    let dest_ptn = matches.get_one::<String>("DEST").unwrap();
    let dry_run = *matches.get_one::<bool>("dry-run").unwrap();
    let verbose = 0 < *matches.get_one::<u8>("verbose").unwrap(); // limited by clap so it's safe
    let interactive = *matches.get_one::<bool>("interactive").unwrap();

    Config {
        src_ptn: src_ptn.to_owned(),
        dest_ptn: dest_ptn.to_owned(),
        dry_run,
        verbose,
        interactive,
    }
}

fn matches_to_actions(src_ptn: &str, dest_ptn: &str) -> Vec<Action> {
    //TODO: Fix for when curdir is not available
    let curdir = std::env::current_dir().unwrap();
    let matches = match walk(&curdir, src_ptn) {
        Err(err) => {
            eprintln!(
                "{}: failed to scan directory tree: {}",
                style_error("error"),
                err
            );
            exit(2); //TODO: Do not exit here
        }
        Ok(matches) => matches,
    };

    let mut actions = Vec::new();
    for m in matches {
        let src = m.path();
        let dest = substitute_variables(dest_ptn, &m.matched_parts[..]);
        let dest = curdir.join(dest);
        actions.push(Action::new(src, dest));
    }
    actions
}

pub fn try_main(args: &[OsString]) -> Result<(), String> {
    // Enable colored output
    #[cfg(windows)]
    ansi_term::enable_ansi_support().unwrap();

    // Parse arguments
    let config = parse_args(args);

    // Collect paths of the files to move with their destination
    let actions = matches_to_actions(config.src_ptn.as_str(), config.dest_ptn.as_str());

    let actions = sort_actions(&actions)?;

    // Move files
    move_files(
        &actions,
        config.dry_run,
        config.interactive,
        config.verbose,
        Some(&|src, _dest, err| {
            eprintln!(
                "{}: failed to move \"{}\": {}",
                style_error("error"),
                src.to_string_lossy(),
                err
            );
        }),
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    mod matches_to_actions {
        use super::*;

        #[test]
        fn no_match() {
            let actions = matches_to_actions("zzzzz", "zzzzz");
            assert_eq!(actions.len(), 0);
        }

        #[test]
        fn multiple_matches() {
            let mut actions = matches_to_actions("Cargo.*", "Foobar.#1");
            actions.sort();
            assert_eq!(actions.len(), 2);
            assert_eq!(
                actions[0].src().file_name().unwrap(),
                PathBuf::from("Cargo.lock")
            );
            assert_eq!(
                actions[1].src().file_name().unwrap(),
                PathBuf::from("Cargo.toml")
            );
            assert_eq!(
                PathBuf::from(actions[0].dest()).file_name().unwrap(),
                PathBuf::from("Foobar.lock")
            );
            assert_eq!(
                PathBuf::from(actions[1].dest()).file_name().unwrap(),
                PathBuf::from("Foobar.toml")
            );
        }
    }
}
