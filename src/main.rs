use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf, MAIN_SEPARATOR};

use pmv::fnmatch;

fn walk(
    dir: &Path,
    patterns: &[Component],
    matched_paths: &mut Vec<(fs::DirEntry, Vec<String>)>,
) -> io::Result<()> {
    println!("# walk(dir=\"{:?}\", patterns=\"{:?}\")", dir, patterns);

    if 1 < patterns.len() {
        match patterns[0] {
            Component::Prefix(p) => {
                // Reset the curdir to the path
                let curdir = p.as_os_str();
                let curdir = PathBuf::from(curdir);
                walk(&curdir, &patterns[1..], matched_paths)
            }
            Component::RootDir => {
                // Move to the root
                let root = MAIN_SEPARATOR.to_string();
                let root = PathBuf::from(root);
                walk(root.as_path(), &patterns[1..], matched_paths)
            }
            Component::ParentDir => {
                // Move to the parent
                let parent = dir.parent().unwrap(); //TODO: Handle error
                walk(parent, &patterns[1..], matched_paths)
            }
            Component::CurDir => {
                // Ignore the path component
                walk(dir, &patterns[1..], matched_paths)
            }
            Component::Normal(p) => {
                // Move into the matched sub-directories
                let dir = dir.join(p);
                let patterns = &patterns[1..];
                walk(dir.as_path(), patterns, matched_paths)
            }
        }
    } else if patterns.len() == 1 {
        match patterns[0] {
            Component::Prefix(_) => {
                // Move to the root
                panic!("Prefix is not supported") //TODO: Support
            }
            Component::RootDir => {
                // Move to the root
                panic!("RootDir is not supported") //TODO: Support
            }
            Component::ParentDir => {
                // Move to the parent
                let parent = dir.parent().unwrap(); //TODO: Handle error
                walk(parent, &patterns[1..], matched_paths)
            }
            Component::CurDir => {
                // Ignore the path component
                walk(dir, &patterns[1..], matched_paths)
            }
            Component::Normal(p) => {
                // Store the matched paths
                for result in fs::read_dir(dir)? {
                    let entry = result?;
                    let fname = entry.file_name();
                    let pattern = p.to_str().unwrap(); //TODO: Handle error
                    if let Some(matches) = fnmatch(&pattern, &fname.to_str().unwrap()) {
                        //TODO: Handle error
                        //println!("{:?} --> {:?}", entry, matches);
                        matched_paths.push((entry, matches));
                    }
                }
                Ok(())
            }
        }
    } else {
        Ok(())
    }
}

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let src_ptns = &args[1];
    let dest_ptn: &str = &args[2];

    let src_ptns: Vec<_> = Path::new(&src_ptns).components().collect();
    let mut sources: Vec<(fs::DirEntry, Vec<String>)> = Vec::new();

    match walk(Path::new("."), src_ptns.as_slice(), &mut sources) {
        Err(e) => println!("Error: {:?}", e),
        Ok(_) => {
            println!("Ok: {:?}", sources);
            for (entry, matches) in sources {
                //println!("# {:?} {:?}", entry, matches);
                let dest_bytes = dest_ptn.as_bytes();
                let mut dest = String::new();
                let mut i = 0;
                while i < dest_bytes.len() {
                    if dest_bytes[i] == 0x5c // Backslash
                        && i + 1 < dest_bytes.len()
                        && 0x30 <= dest_bytes[i + 1] // 0
                        && dest_bytes[i + 1] <= 0x39
                    // 9
                    {
                        let index = (dest_bytes[i + 1] - 0x30 - 1) as usize;
                        let replacement = &matches[index];
                        dest.push_str(&replacement);
                        i += 2;
                    } else {
                        dest.push_str(&dest_ptn[i..=i]);
                        i += 1;
                    }
                }
                println!("{:?} --> {:?}", &entry, &PathBuf::from(dest));
                //std::fs::rename(&entry.path(), &PathBuf::from(dest));
            }
        }
    }
}
