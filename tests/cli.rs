use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ptree() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ptree"))
}

fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ptree_test_{name}_{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn nonexistent_path_fails_with_colorless() {
    let output = ptree()
        .args(["--no-color", "/nonexistent/ptree/path"])
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("path not found"));
}

#[test]
fn max_depth_limits_output() {
    let dir = temp_dir("max_depth");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/file.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "-L", "2"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sub"));
    assert!(!stdout.contains("file.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn gitignore_hides_matching_entries() {
    let dir = temp_dir("gitignore");
    fs::write(dir.join(".gitignore"), "ignored/\n").unwrap();
    fs::create_dir_all(dir.join("ignored")).unwrap();
    fs::create_dir_all(dir.join("visible")).unwrap();

    let output = ptree()
        .args(["--no-color", "--ascii"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("visible"));
    assert!(!stdout.contains("ignored"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn show_gitignore_includes_ignored_entries() {
    let dir = temp_dir("show_gitignore");
    fs::write(dir.join(".gitignore"), "ignored/\n").unwrap();
    fs::create_dir_all(dir.join("ignored")).unwrap();

    let output = ptree()
        .args(["--no-color", "--show-gitignore"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ignored"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn dir_only_shows_directories() {
    let dir = temp_dir("dir_only");
    fs::create_dir_all(dir.join("child")).unwrap();
    fs::write(dir.join("child/file.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "-d"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("child"));
    assert!(!stdout.contains("file.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn nested_gitignore_hides_files_in_subdirectory() {
    let dir = temp_dir("nested_gitignore");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/.gitignore"), "*.generated\n").unwrap();
    fs::write(dir.join("src/main.rs"), "").unwrap();
    fs::write(dir.join("src/out.generated"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--ascii"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("out.generated"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn icons_shown_by_default() {
    let dir = temp_dir("icons_default");
    fs::create_dir_all(dir.join("child")).unwrap();
    fs::write(dir.join("child/main.rs"), "").unwrap();

    let output = ptree().args(["--no-color"]).arg(&dir).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains(" "));
    assert!(stdout.contains("󰉋 "));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn no_icons_flag_hides_icons() {
    let dir = temp_dir("no_icons");
    fs::write(dir.join("main.rs"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("󱘗"));
    assert!(stdout.contains("main.rs"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ascii_icon_type_uses_ascii_markers() {
    let dir = temp_dir("ascii_icons");
    fs::write(dir.join("main.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--icon-theme", "ascii"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("[F]"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn footer_reports_directory_and_file_counts() {
    let dir = temp_dir("footer_stats");
    fs::write(dir.join("readme.txt"), "").unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 directories, 1 file"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn no_report_omits_footer_summary() {
    let dir = temp_dir("no_report");
    fs::write(dir.join("readme.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--no-report"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("readme.txt"));
    assert!(!stdout.contains("directory, "));
    assert!(!stdout.contains("directories, "));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn dir_only_footer_counts_only_listed_directories() {
    let dir = temp_dir("footer_dir_only");
    fs::create_dir_all(dir.join("child")).unwrap();
    fs::write(dir.join("child/file.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "-d"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("2 directories, 0 files"));
    let _ = fs::remove_dir_all(&dir);
}

#[cfg(unix)]
#[test]
fn symlink_cycle_does_not_hang() {
    use std::os::unix::fs::symlink;

    let dir = temp_dir("symlink_cycle");
    let a = dir.join("a");
    let b = dir.join("b");
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    symlink(&b, a.join("link_to_b")).unwrap();
    symlink(&a, b.join("link_to_a")).unwrap();

    let output = ptree()
        .args(["--no-color", "--ascii"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let _ = fs::remove_dir_all(&dir);
}
