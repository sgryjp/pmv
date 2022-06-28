use std::path::{Path, PathBuf};

/// A pair of source and destination in a moving plan.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Action {
    src: PathBuf,
    dest: PathBuf,
}

impl Action {
    pub fn new<P1: Into<PathBuf>, P2: Into<PathBuf>>(src: P1, dest: P2) -> Action {
        Action {
            src: src.into(),
            dest: dest.into(),
        }
    }

    pub fn src(self: &Action) -> &Path {
        self.src.as_path()
    }

    pub fn dest(self: &Action) -> &Path {
        self.dest.as_path()
    }
}

impl<'a> From<&'a Action> for (&'a Path, &'a Path) {
    fn from(action: &'a Action) -> (&'a Path, &'a Path) {
        (action.src.as_path(), action.dest.as_path())
    }
}
