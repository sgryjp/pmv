use std::fs;
use std::path::PathBuf;
use std::process::Command;

//#[test]
fn swap_filenames() {
    let temp_dir = PathBuf::from("temp/integration_1");

    // Prepare files and directories to testing
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir).expect("Failed to remove temp_dir");
    }
    fs::create_dir_all(&temp_dir).expect("Failed to create temp_dir");
    fs::write(&temp_dir.join("AB"), "AB").unwrap();
    fs::write(&temp_dir.join("BA"), "BA").unwrap();

    // Execute pmv
    let mut command = Command::new("../../target/debug/pmv");
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
