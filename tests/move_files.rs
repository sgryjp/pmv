use std::fs;
use std::path::{Path, PathBuf};

use pmv::move_files;

fn setup(id: &str) {
    let _ = fs::create_dir(Path::new("temp"));
    let _ = fs::remove_dir_all(Path::new(&format!("temp/{}", id)));
    for dir1 in ["foo", "bar", "baz"].iter() {
        for dir2 in ["foo", "bar", "baz"].iter() {
            let _ = fs::create_dir_all(Path::new(&format!("temp/{}/{}/{}", id, dir1, dir2)));
            for fname in ["foo", "bar", "baz"].iter() {
                let path: String = format!("temp/{}/{}/{}/{}", id, dir1, dir2, fname);
                fs::write(Path::new(&path), path.as_bytes()).unwrap();
            }
        }
    }

    #[cfg(unix)]
    fn setup_for_unix(id: &str) {
        let target = PathBuf::from(&format!("temp/{}/baz/baz/baz", id));
        let target = target.canonicalize().unwrap();
        let link = format!("temp/{}/symlink2file", id);
        std::os::unix::fs::symlink(target, Path::new(&link)).unwrap();

        let target = PathBuf::from(&format!("temp/{}/baz/baz", id));
        let target = target.canonicalize().unwrap();
        let link = format!("temp/{}/symlink2dir", id);
        std::os::unix::fs::symlink(target, Path::new(&link)).unwrap();
    }
    #[cfg(unix)]
    setup_for_unix(id);
}

#[test]
fn test_move_files_ok() {
    let id = "test_move_files_ok";
    setup(id);

    let sources: Vec<PathBuf> = vec![
        format!("temp/{}/foo/foo/foo", id),
        format!("temp/{}/foo/bar/foo", id),
        format!("temp/{}/foo/baz/foo", id),
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    let dests: Vec<String> = vec![
        format!("temp/{}/foo/foo/zzz", id),
        format!("temp/{}/foo/bar/zzz", id),
        format!("temp/{}/foo/baz/zzz", id),
    ]
    .iter()
    .map(|x| String::from(x))
    .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert!(!sources[0].exists());
    assert!(!sources[1].exists());
    assert!(!sources[2].exists());
    assert!(Path::new(&dests[0]).exists());
    assert!(Path::new(&dests[1]).exists());
    assert!(Path::new(&dests[2]).exists());
    assert_eq!(
        fs::read_to_string(Path::new(&dests[0])).unwrap(),
        sources[0].to_str().unwrap(),
    );
    assert_eq!(
        fs::read_to_string(Path::new(&dests[1])).unwrap(),
        sources[1].to_str().unwrap(),
    );
    assert_eq!(
        fs::read_to_string(Path::new(&dests[2])).unwrap(),
        sources[2].to_str().unwrap(),
    );

    assert_eq!(num_errors, 0);
}

#[test]
fn test_move_files_dry_run() {
    let id = "test_move_files_dry_run";
    setup(id);

    let sources: Vec<PathBuf> = vec![
        format!("temp/{}/foo/foo/foo", id),
        format!("temp/{}/foo/bar/foo", id),
        format!("temp/{}/foo/baz/foo", id),
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    let dests: Vec<String> = vec![
        format!("temp/{}/foo/foo/zzz", id),
        format!("temp/{}/foo/bar/zzz", id),
        format!("temp/{}/foo/baz/zzz", id),
    ]
    .iter()
    .map(|x| String::from(x))
    .collect();
    let dry_run = true;
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert!(sources[0].exists());
    assert!(sources[1].exists());
    assert!(sources[2].exists());
    assert!(!Path::new(&dests[0]).exists());
    assert!(!Path::new(&dests[1]).exists());
    assert!(!Path::new(&dests[2]).exists());

    assert_eq!(num_errors, 0);
}

#[test]
fn test_move_files_invalid_dest() {
    let id = "test_move_files_invalid_dest";
    setup(id);

    let sources: Vec<PathBuf> = vec![
        format!("temp/{}/foo/foo/foo", id),
        format!("temp/{}/foo/bar/foo", id),
        format!("temp/{}/foo/baz/foo", id),
    ]
    .iter()
    .map(PathBuf::from)
    .collect();
    let dests: Vec<String> = vec![
        format!("temp/{}/foo/foo/\0", id),
        format!("temp/{}/foo/bar/\0", id),
        format!("temp/{}/foo/baz/\0", id),
    ]
    .iter()
    .map(|x| String::from(x))
    .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert!(sources[0].exists());
    assert!(sources[1].exists());
    assert!(sources[2].exists());
    assert!(!Path::new(&dests[0]).exists());
    assert!(!Path::new(&dests[1]).exists());
    assert!(!Path::new(&dests[2]).exists());

    assert_eq!(num_errors, 3);
}

#[test]
fn test_move_files_file_to_file() {
    let id = "test_move_files_file_to_file";
    setup(id);

    let sources: Vec<PathBuf> = vec![format!("temp/{}/foo/foo/foo", id)]
        .iter()
        .map(PathBuf::from)
        .collect();
    let dests: Vec<String> = vec![format!("temp/{}/foo/foo/bar", id)]
        .iter()
        .map(|x| String::from(x))
        .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    assert!(!sources[0].exists());
    assert!(Path::new(&dests[0]).exists());
    assert_eq!(
        fs::read_to_string(Path::new(&dests[0])).unwrap(),
        format!("temp/{}/foo/foo/foo", id)
    );

    assert_eq!(num_errors, 0);
}

#[test]
fn test_move_files_file_to_dir() {
    let id = "test_move_files_file_to_dir";
    setup(id);

    let sources: Vec<PathBuf> = vec![format!("temp/{}/foo/foo/foo", id)]
        .iter()
        .map(PathBuf::from)
        .collect();
    let dests: Vec<String> = vec![format!("temp/{}/foo/bar", id)]
        .iter()
        .map(|x| String::from(x))
        .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, false, None);

    let expected_dest = format!("temp/{}/foo/bar/foo", id);
    assert!(!sources[0].exists());
    assert!(Path::new(&expected_dest).exists());
    assert_eq!(
        fs::read_to_string(Path::new(&expected_dest)).unwrap(),
        format!("temp/{}/foo/foo/foo", id)
    );

    assert_eq!(num_errors, 0);
}

#[cfg(unix)]
#[test]
fn test_move_files_file_to_symlink2file() {
    let id = "test_move_files_file_to_symlink2file";
    setup(id);

    let sources: Vec<PathBuf> = vec![format!("temp/{}/foo/foo/foo", id)]
        .iter()
        .map(PathBuf::from)
        .collect();
    let dests: Vec<String> = vec![format!("temp/{}/symlink2file", id)]
        .iter()
        .map(|x| String::from(x))
        .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, true, None);

    assert!(!sources[0].exists());
    assert!(Path::new(&dests[0]).exists());
    assert_eq!(
        fs::read_to_string(Path::new(&dests[0])).unwrap(),
        format!("temp/{}/foo/foo/foo", id)
    );

    assert_eq!(num_errors, 0);
}

#[cfg(unix)]
#[test]
fn test_move_files_file_to_symlink2dir() {
    let id = "test_move_files_file_to_symlink2dir";
    setup(id);

    let sources: Vec<PathBuf> = vec![format!("temp/{}/foo/foo/foo", id)]
        .iter()
        .map(PathBuf::from)
        .collect();
    let dests: Vec<String> = vec![format!("temp/{}/symlink2dir", id)]
        .iter()
        .map(|x| String::from(x))
        .collect();
    let dry_run = false;
    let num_errors = move_files(&sources, &dests, dry_run, true, None);

    let expected_dest1 = format!("temp/{}/symlink2dir/foo", id);
    let expected_dest2 = format!("temp/{}/baz/baz/foo", id);
    assert!(!sources[0].exists());
    assert!(Path::new(&expected_dest1).exists());
    assert!(Path::new(&expected_dest2).exists());
    assert_eq!(
        fs::read_to_string(Path::new(&expected_dest1)).unwrap(),
        format!("temp/{}/foo/foo/foo", id)
    );
    assert_eq!(
        fs::read_to_string(Path::new(&expected_dest2)).unwrap(),
        format!("temp/{}/foo/foo/foo", id)
    );

    assert_eq!(num_errors, 0);
}
