use std::path::{Path, PathBuf};

/// A pair of source and destination in a moving plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Action {
    pub src: PathBuf,
    pub dest: PathBuf,
}

impl Action {
    pub fn from_str(src: &str, dest: &str) -> Action {
        Action {
            src: PathBuf::from(src),
            dest: PathBuf::from(dest),
        }
    }
}

impl<'a> From<&'a Action> for (&'a Path, &'a Path) {
    fn from(action: &'a Action) -> (&'a Path, &'a Path) {
        (action.src.as_path(), action.dest.as_path())
    }
}
