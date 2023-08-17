use std::process::Command;

fn check_prerequisites() {
    const ERROR_MESSAGE: &str = "Go is not installed or not found in PATH";
    let output = Command::new("go")
        .arg("version")
        .output()
        .expect(ERROR_MESSAGE);
    assert!(output.status.success(), "{}", ERROR_MESSAGE);
}

fn main() {
    println!("cargo:rerun-if-changed=../neurahive-client");

    check_prerequisites();

    let status = Command::new("go")
        .current_dir("../neurahive-client")
        .args(vec!["build", "-o", "../target"])
        .status()
        .unwrap();

    println!("build neurahive-client with status {}", status);
}
