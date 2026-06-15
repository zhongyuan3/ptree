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

#[test]
fn dirsfirst_lists_directories_before_files() {
    let dir = temp_dir("dirsfirst");
    fs::write(dir.join("zebra.txt"), "").unwrap();
    fs::create_dir_all(dir.join("alpha")).unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--ascii"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let alpha_pos = stdout.find("alpha").unwrap();
    let zebra_pos = stdout.find("zebra.txt").unwrap();
    assert!(alpha_pos < zebra_pos);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn no_dirsfirst_sorts_entries_by_name_only() {
    let dir = temp_dir("no_dirsfirst");
    fs::create_dir_all(dir.join("zebra")).unwrap();
    fs::write(dir.join("alpha.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--ascii", "--no-dirsfirst"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let alpha_pos = stdout.find("alpha.txt").unwrap();
    let zebra_pos = stdout.find("zebra").unwrap();
    assert!(alpha_pos < zebra_pos);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn ignore_pattern_excludes_matching_entries() {
    let dir = temp_dir("ignore_pattern");
    fs::write(dir.join("keep.txt"), "").unwrap();
    fs::write(dir.join("skip.log"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "-I", "*.log"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("keep.txt"));
    assert!(!stdout.contains("skip.log"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn include_pattern_shows_only_matching_files_and_parents() {
    let dir = temp_dir("include_pattern");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "").unwrap();
    fs::write(dir.join("src/readme.txt"), "").unwrap();
    fs::write(dir.join("notes.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "-P", "*.rs"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("src"));
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("readme.txt"));
    assert!(!stdout.contains("notes.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn paths_mode_prints_relative_paths_one_per_line() {
    let dir = temp_dir("paths_mode");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "").unwrap();
    fs::write(dir.join("readme.txt"), "").unwrap();

    let output = ptree()
        .args(["--paths", "--no-icons"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("readme.txt"));
    assert!(stdout.contains("src/main.rs"));
    assert!(!stdout.contains("├──"));
    assert!(!stdout.contains("directory, "));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn paths_mode_respects_max_depth() {
    let dir = temp_dir("paths_depth");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/deep.txt"), "").unwrap();

    let output = ptree()
        .args(["--paths", "--no-icons", "-L", "2"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("sub"));
    assert!(!stdout.contains("deep.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn invalid_glob_pattern_fails() {
    let dir = temp_dir("invalid_glob");
    fs::write(dir.join("file.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "-I", "[invalid"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("invalid glob pattern"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn json_output_is_valid_tree_document() {
    let dir = temp_dir("json_output");
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();

    let output = ptree()
        .args(["-J", "--no-icons"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(value["type"], "directory");
    assert!(value["children"].is_array());
    assert!(stdout.contains("main.rs"));
    assert!(!stdout.contains("directory, "));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn size_flag_shows_byte_size() {
    let dir = temp_dir("size_bytes");
    fs::write(dir.join("data.bin"), vec![0_u8; 512]).unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--size"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("512"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn human_size_flag_shows_readable_units() {
    let dir = temp_dir("size_human");
    fs::write(dir.join("data.bin"), vec![0_u8; 1536]).unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--human-size"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1.5K"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn config_file_applies_defaults() {
    let dir = temp_dir("config_file");
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/deep.txt"), "").unwrap();
    fs::write(dir.join("root.txt"), "").unwrap();

    let config_path = dir.join("ptree.toml");
    fs::write(&config_path, "level = 2\n").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--config"])
        .arg(&config_path)
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("root.txt"));
    assert!(stdout.contains("sub"));
    assert!(!stdout.contains("deep.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn bash_completions_include_ptree_command() {
    let output = ptree().args(["completions", "bash"]).output().unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ptree"));
}

#[test]
fn light_color_theme_runs_successfully() {
    let dir = temp_dir("light_theme");
    fs::write(dir.join("main.rs"), "fn main() {}\n").unwrap();

    let output = ptree()
        .args(["--no-icons", "--color-theme", "light"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("main.rs"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn filelimit_stops_listing_after_limit() {
    let dir = temp_dir("filelimit");
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("b.txt"), "").unwrap();
    fs::create_dir_all(dir.join("sub")).unwrap();
    fs::write(dir.join("sub/c.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--filelimit", "2"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("file limit of 2 entries reached"));
    assert!(!stdout.contains("c.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn sort_ext_groups_files_by_extension() {
    let dir = temp_dir("sort_ext_cli");
    fs::write(dir.join("b.rs"), "").unwrap();
    fs::write(dir.join("a.txt"), "").unwrap();
    fs::write(dir.join("c.rs"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons", "--sort", "ext"])
        .arg(&dir)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_rs = stdout.find("b.rs").unwrap();
    let txt_pos = stdout.find("a.txt").unwrap();
    assert!(first_rs < txt_pos);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn multiple_paths_print_separate_trees() {
    let dir = temp_dir("multi_path");
    let first = dir.join("first");
    let second = dir.join("second");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::write(first.join("one.txt"), "").unwrap();
    fs::write(second.join("two.txt"), "").unwrap();

    let output = ptree()
        .args(["--no-color", "--no-icons"])
        .arg(&first)
        .arg(&second)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("one.txt"));
    assert!(stdout.contains("two.txt"));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn json_multiple_paths_outputs_array() {
    let dir = temp_dir("multi_json");
    let first = dir.join("alpha");
    let second = dir.join("beta");
    fs::create_dir_all(&first).unwrap();
    fs::create_dir_all(&second).unwrap();
    fs::write(first.join("a.txt"), "").unwrap();
    fs::write(second.join("b.txt"), "").unwrap();

    let output = ptree()
        .args(["-J", "--no-icons"])
        .arg(&first)
        .arg(&second)
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let value: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert!(value.is_array());
    assert_eq!(value.as_array().unwrap().len(), 2);
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
