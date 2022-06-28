use std::path::{Path, PathBuf};

/// A pair of source and destination in a moving plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub src: PathBuf,
    pub dest: PathBuf,
}

impl Entry {
    pub fn from_str(src: &str, dest: &str) -> Entry {
        Entry {
            src: PathBuf::from(src),
            dest: PathBuf::from(dest),
        }
    }
}

impl<'a> From<&'a Entry> for (&'a Path, &'a Path) {
    fn from(ent: &'a Entry) -> (&'a Path, &'a Path) {
        (ent.src.as_path(), ent.dest.as_path())
    }
}
