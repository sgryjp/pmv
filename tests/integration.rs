use function_name::named;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use pmv::try_main;

fn prepare(function_name: &str) -> PathBuf {
    assert!(PathBuf::from("./Cargo.toml").exists());
    let temp_dir = PathBuf::from(format!("temp/system/{}", function_name));

    // Prepare files and directories to testing
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove temp_dir");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp_dir");

    temp_dir
}

fn make_command() -> Command {
    let cmd_path = PathBuf::from("./target/debug/pmv").canonicalize().unwrap();
    Command::new(cmd_path)
}

#[named]
#[test]
fn dry_run() {
    let temp_dir = prepare(function_name!());

    // Prepare files and directories to testing
    fs::write(&temp_dir.join("AA"), "AA").unwrap();
    fs::write(&temp_dir.join("AB"), "AB").unwrap();

    // Execute pmv with --dry-run
    let mut args: Vec<OsString> = vec![
        PathBuf::from("--dry-run"),
        temp_dir.join("??"),
        temp_dir.join("B#2"),
    ]
    .iter()
    .map(|s| OsString::from(s))
    .collect();
    args.insert(0, env::args_os().next().unwrap());
    let result = try_main(&args);
    assert!(result.is_ok());

    // Confirm nothing moved
    let path_ba = temp_dir.join("BA");
    let path_bb = temp_dir.join("BB");
    assert!(!path_ba.exists());
    assert!(!path_bb.exists());

    // Then do the same without --dry-run
    let mut args: Vec<OsString> = vec![temp_dir.join("??"), temp_dir.join("B#2")]
        .iter()
        .map(|s| OsString::from(s))
        .collect();
    args.insert(0, env::args_os().next().unwrap());
    let result = try_main(&args);
    assert!(result.is_ok());

    // Confirm files were moved
    assert!(path_ba.exists());
    assert!(path_bb.exists());
    assert_eq!(fs::read_to_string(&path_ba).unwrap(), "AA");
    assert_eq!(fs::read_to_string(&path_bb).unwrap(), "AB");
}

#[named]
#[test]
fn interactive() {
    let temp_dir = prepare(function_name!());
    let path_a = temp_dir.join("A");
    let path_b = temp_dir.join("B");

    // Prepare files and directories to testing
    fs::write(&temp_dir.join("A"), "A").unwrap();
    fs::write(&temp_dir.join("B"), "B").unwrap();

    // Execute pmv in interactive mode and enter 'N'
    let mut command = make_command();
    let mut proc = command
        .current_dir(&temp_dir)
        .arg("--interactive")
        .arg("A")
        .arg("B")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to launch pmv (debug build)");
    let mut stdin = proc.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        stdin.write_all(b"N").expect("failed to write 'N' to stdin");
    });
    let output = proc.wait_with_output().expect("wait for child proc failed");
    assert!(output.status.success());
    assert!(path_a.exists());
    assert!(path_b.exists());
    assert_eq!(fs::read_to_string(&path_a).unwrap(), "A");
    assert_eq!(fs::read_to_string(&path_b).unwrap(), "B");

    // Execute pmv in interactive mode and enter 'y'
    let mut command = make_command();
    let mut proc = command
        .current_dir(&temp_dir)
        .arg("--interactive")
        .arg("A")
        .arg("B")
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()
        .expect("Failed to launch pmv (debug build)");
    let mut stdin = proc.stdin.take().expect("failed to get stdin");
    std::thread::spawn(move || {
        stdin.write_all(b"y").expect("failed to write 'y' to stdin");
    });
    let output = proc.wait_with_output().expect("wait for child proc failed");
    assert!(output.status.success());

    // Test the result
    assert!(!path_a.exists());
    assert!(path_b.exists());
    assert_eq!(fs::read_to_string(&path_b).unwrap(), "A");
}

#[named]
#[allow(dead_code)]
//#[test]
fn swap_filenames() {
    let temp_dir = prepare(function_name!());
    let path_ab = temp_dir.join("AB");
    let path_ba = temp_dir.join("BA");

    // Prepare files and directories to testing
    fs::write(&path_ab, "AB").unwrap();
    fs::write(&path_ba, "BA").unwrap();

    // Execute pmv
    let mut command = make_command();
    let output = command
        .current_dir(&temp_dir)
        .arg("-v")
        .arg("??")
        .arg("#2#1")
        .output()
        .expect("Failed to launch pmv (debug build)");
    assert!(output.status.success());

    // Test the result
    assert!(path_ab.exists());
    assert!(path_ba.exists());
    assert_eq!(fs::read_to_string(&path_ab).unwrap(), "BA");
    assert_eq!(fs::read_to_string(&path_ba).unwrap(), "AB");
}
