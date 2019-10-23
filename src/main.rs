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
        } else {
            replaced.push(dest[i] as char);
            i += 1;
        }
    }
    replaced
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let src_ptns = &args[1];
    let dest_ptn: &str = &args[2];

    match walk(Path::new("."), src_ptns) {
        Err(err) => {
            eprintln!("Error: {:?}", err);
            exit(2);
        }
        Ok(matches) => {
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
                //std::fs::rename(&entry.path(), &PathBuf::from(dest));
            }
        }
    }
}
