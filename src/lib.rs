mod fnmatch;
mod walk;

pub use fnmatch::fnmatch;
pub use walk::walk;

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
