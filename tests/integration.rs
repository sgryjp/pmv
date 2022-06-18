use function_name::named;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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
    let mut command = make_command();
    let output = command
        .current_dir(&temp_dir)
        .arg("--dry-run")
        .arg("??")
        .arg("B#2")
        .output()
        .expect("Failed to launch pmv (debug build)");
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("AA --> BA"));
    assert!(stdout.contains("AB --> BB"));

    // Confirm nothing moved
    let path_ba = temp_dir.join("BA");
    let path_bb = temp_dir.join("BB");
    assert!(!path_ba.exists());
    assert!(!path_bb.exists());

    // Then do the same without --dry-run
    let mut command = make_command();
    let output = command
        .current_dir(&temp_dir)
        .arg("??")
        .arg("B#2")
        .output()
        .expect("Failed to launch pmv (debug build)");
    assert!(output.status.success());

    // Confirm files were moved
    assert!(path_ba.exists());
    assert!(path_bb.exists());
    assert_eq!(fs::read_to_string(&path_ba).unwrap(), "AA");
    assert_eq!(fs::read_to_string(&path_bb).unwrap(), "AB");
}

#[named]
#[allow(dead_code)]
//#[test]
fn swap_filenames() {
    let temp_dir = prepare(function_name!());

    // Prepare files and directories to testing
    fs::write(&temp_dir.join("AB"), "AB").unwrap();
    fs::write(&temp_dir.join("BA"), "BA").unwrap();

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
    let path_ab = temp_dir.join("AB");
    let path_ba = temp_dir.join("BA");
    assert!(path_ab.exists());
    assert!(path_ba.exists());
    assert_eq!(fs::read_to_string(&path_ab).unwrap(), "BA");
    assert_eq!(fs::read_to_string(&path_ba).unwrap(), "AB");
}
