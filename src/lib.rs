mod fnmatch;
mod walk;

pub use fnmatch::fnmatch;
pub use walk::walk;

/// Replaces variables in the given destination path string using the given
/// substrings.
pub fn replace(dest_ptn: &str, substrings: &[String]) -> String {
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
