use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os;

use pmv::move_files;

fn prepare_test(id: &str) -> Result<(), io::Error> {
    if !Path::new("temp").exists() {
        fs::create_dir("temp").unwrap();
    }
    let path = format!("temp/{}", id);
    if Path::new(&path).exists() {
        fs::remove_dir_all(Path::new(&path)).unwrap();
    }
    fs::create_dir(Path::new(&path))
}

fn mkpathstring(id: &str, name: &str) -> String {
    format!("temp/{}/{}", id, name)
}

fn mkpathbuf(id: &str, name: &str) -> PathBuf {
    let path = mkpathstring(id, name);
    PathBuf::from(&path)
}

fn mkfile(id: &str, name: &str) -> Result<(), io::Error> {
    let path = mkpathstring(id, name);
    fs::write(Path::new(&path), &path)
}

fn mkdir(id: &str, name: &str) -> Result<(), io::Error> {
    let path = mkpathstring(id, name);
    fs::create_dir(Path::new(&path))
}

#[cfg(unix)]
fn mklink(id: &str, src: &str, dest: &str) -> Result<(), io::Error> {
    let dest = mkpathstring(id, dest);
    let src = PathBuf::from(mkpathstring(id, src));
    let src = src.canonicalize().unwrap();
    os::unix::fs::symlink(src, dest)
}

fn content_of(id: &str, name: &str) -> String {
    let path = mkpathstring(id, name);
    fs::read_to_string(Path::new(&path)).unwrap()
}

#[test]
fn dry_run() {
    let id = "dry_run";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();
    mkfile(id, "f2").unwrap();

    let dry_run = true;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "f2")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(mkpathbuf(id, "f1").exists());
    assert!(mkpathbuf(id, "f2").exists());
    assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
    assert_eq!(content_of(id, "f2"), format!("temp/{}/f2", id));
}

#[test]
fn invalid_dest() {
    let id = "invalid_dest";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "\0")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 1);
    assert!(mkpathbuf(id, "f1").exists());
    assert!(!mkpathbuf(id, "\0").exists());
    assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
}

#[test]
fn file_to_file() {
    let id = "file_to_file";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();
    mkfile(id, "f2").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "f2")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "f1").exists());
    assert!(mkpathbuf(id, "f2").exists());
    assert_eq!(content_of(id, "f2"), format!("temp/{}/f1", id));
}

#[test]
fn file_to_dir() {
    let id = "file_to_dir";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();
    mkdir(id, "d1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "d1")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "f1").exists());
    assert!(mkpathbuf(id, "d1/f1").exists());
    assert_eq!(content_of(id, "d1/f1"), format!("temp/{}/f1", id));
}

#[cfg(unix)]
#[test]
fn file_to_symlink2file() {
    let id = "file_to_symlink2file";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();
    mklink(id, "f1", "lf1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "lf1")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "f1").exists());
    assert!(mkpathbuf(id, "lf1").exists());
    assert_eq!(content_of(id, "lf1"), format!("temp/{}/f1", id));
}

#[cfg(unix)]
#[test]
fn file_to_symlink2dir() {
    let id = "file_to_symlink2dir";

    prepare_test(id).unwrap();
    mkfile(id, "f1").unwrap();
    mkdir(id, "d1").unwrap();
    mklink(id, "d1", "ld1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "f1")];
    let dests: Vec<String> = vec![mkpathstring(id, "ld1")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "f1").exists());
    assert!(mkpathbuf(id, "d1/f1").is_file());
    assert!(mkpathbuf(id, "ld1/f1").is_file());
    assert_eq!(content_of(id, "ld1/f1"), format!("temp/{}/f1", id));
}

#[test]
fn dir_to_file() {
    let id = "dir_to_file";

    prepare_test(id).unwrap();
    mkdir(id, "d1").unwrap();
    mkfile(id, "f1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
    let dests: Vec<String> = vec![mkpathstring(id, "f1")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 1);
    assert!(mkpathbuf(id, "d1").exists());
    assert!(mkpathbuf(id, "f1").exists());
    assert_eq!(content_of(id, "f1"), format!("temp/{}/f1", id));
}

#[test]
fn dir_to_dir() {
    let id = "dir_to_dir";

    prepare_test(id).unwrap();
    mkdir(id, "d1").unwrap();
    mkdir(id, "d2").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
    let dests: Vec<String> = vec![mkpathstring(id, "d2")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "d1").exists());
    assert!(mkpathbuf(id, "d2").exists());
    assert!(mkpathbuf(id, "d2/d1").exists());
}

#[cfg(unix)]
#[test]
fn dir_to_symlink2file() {
    let id = "dir_to_symlink2file";

    prepare_test(id).unwrap();
    mkdir(id, "d1").unwrap();
    mkfile(id, "f1").unwrap();
    mklink(id, "f1", "lf1").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
    let dests: Vec<String> = vec![mkpathstring(id, "lf1")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 1);
    assert!(mkpathbuf(id, "d1").is_dir());
    assert!(mkpathbuf(id, "f1").is_file());
}

#[cfg(unix)]
#[test]
fn dir_to_symlink2dir() {
    let id = "dir_to_symlink2dir";

    prepare_test(id).unwrap();
    mkdir(id, "d1").unwrap();
    mkdir(id, "d2").unwrap();
    mklink(id, "d2", "ld2").unwrap();

    let dry_run = false;
    let sources: Vec<PathBuf> = vec![mkpathbuf(id, "d1")];
    let dests: Vec<String> = vec![mkpathstring(id, "ld2")];
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert_eq!(num_errors, 0);
    assert!(!mkpathbuf(id, "d1").exists());
    assert!(mkpathbuf(id, "d2/d1").is_dir());
    assert!(mkpathbuf(id, "ld2/d1").is_dir());
}
