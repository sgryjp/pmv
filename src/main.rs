#[macro_use]
extern crate clap;
use clap::Arg;

use std::cmp;
use std::env;
use std::path::Path;
use std::process::exit;

use pmv::walk;

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

fn main() {
    let matches = app_from_crate!()
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
                .help("Source pattern"),
        )
        .arg(
            Arg::with_name("DEST")
                .required(true)
                .index(2)
                .help("Destination pattern"),
        )
        .get_matches_from(env::args_os());
    let src_ptn = matches.value_of("SOURCE").unwrap();
    let dest_ptn = matches.value_of("DEST").unwrap();
    let dry_run = 0 < matches.occurrences_of("dry-run");

    let matches = match walk(Path::new("."), src_ptn) {
        Err(err) => {
            eprintln!("Error: {:?}", err);
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

    let src_max_len = sources
        .iter()
        .map(|x| x.to_str().unwrap().len())
        .fold(0, |acc, x| cmp::max(acc, x));

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
                eprintln!("Failed to copy \"{}\": {}", src, err);
            }
        }
    }
}
