use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::{self, File};
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_cli_ignores_binaries() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // 1. Create a binary file
    let bin_path = root.join("program.exe");
    let mut f = File::create(&bin_path)?;
    // Write null bytes to look like binary
    f.write_all(&[0u8; 100])?;

    // 2. Create a text file
    let text_path = root.join("readme.md");
    let mut f = File::create(&text_path)?;
    writeln!(f, "Important Context")?;

    // 3. Run gitmelt
    let mut cmd = Command::cargo_bin("gitmelt")?;
    cmd.arg(root.to_str().unwrap())
        .arg("--stdout")
        .arg("--no-tokens");

    // 4. Verify output
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Important Context"))
        .stdout(predicate::str::contains("Skipped: Binary"));

    Ok(())
}

#[test]
fn test_gitignore_logic() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path();

    // Create .git directory to ensure ignore crate respects .gitignore
    fs::create_dir(root.join(".git"))?;

    // Create .gitignore
    let mut gitignore = File::create(root.join(".gitignore"))?;
    writeln!(gitignore, "secret.txt")?;

    // Create files
    File::create(root.join("secret.txt"))?;
    let mut public = File::create(root.join("public.txt"))?;
    writeln!(public, "Public info")?;

    let mut cmd = Command::cargo_bin("gitmelt")?;
    cmd.arg(root.to_str().unwrap())
        .arg("--stdout")
        .arg("--no-tokens");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("public.txt"))
        .stdout(predicate::str::contains("secret.txt").not());

    Ok(())
}

#[test]
fn test_file_ordering() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path();

    fs::create_dir_all(root.join("a"))?;
    fs::create_dir_all(root.join("b"))?;

    let mut az = File::create(root.join("a/z.txt"))?;
    writeln!(az, "Content A/Z")?;

    let mut ba = File::create(root.join("b/a.txt"))?;
    writeln!(ba, "Content B/A")?;

    let mut cmd = Command::cargo_bin("gitmelt")?;
    cmd.arg(root.to_str().unwrap())
        .arg("--stdout")
        .arg("--no-tokens");

    let output = cmd.assert().success().get_output().stdout.clone();
    let output_str = String::from_utf8(output)?;

    let pos_az = output_str.find("a/z.txt").unwrap();
    let pos_ba = output_str.find("b/a.txt").unwrap();

    assert!(pos_az < pos_ba, "a/z.txt should come before b/a.txt");

    Ok(())
}

#[test]
fn test_include_exclude_complexity() -> Result<(), Box<dyn std::error::Error>> {
    let temp = TempDir::new()?;
    let root = temp.path();

    fs::create_dir_all(root.join("src"))?;
    fs::create_dir_all(root.join("tests"))?;

    let mut main_rs = File::create(root.join("src/main.rs"))?;
    writeln!(main_rs, "fn main() {{}}")?;

    let mut utils_rs = File::create(root.join("src/utils.rs"))?;
    writeln!(utils_rs, "fn utils() {{}}")?;

    let mut test_rs = File::create(root.join("tests/main_test.rs"))?;
    writeln!(test_rs, "test")?;

    let mut cmd = Command::cargo_bin("gitmelt")?;
    cmd.arg(root.to_str().unwrap())
        .arg("--stdout")
        .arg("--no-tokens")
        .arg("--include")
        .arg("src/*.rs")
        .arg("--exclude")
        .arg("utils.rs");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("src/utils.rs").not())
        .stdout(predicate::str::contains("tests/main_test.rs").not());

    Ok(())
}
