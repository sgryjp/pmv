use std::path::{Path, PathBuf};

use pmv::walk;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let src_ptns = &args[1];
    let dest_ptn: &str = &args[2];

    match walk(Path::new("."), src_ptns) {
        Err(err) => println!("Error: {:?}", err),
        Ok(sources) => {
            for m in sources {
                let (entry, matches) = m;
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
