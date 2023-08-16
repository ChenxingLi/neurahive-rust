use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../neurahive-client");

    let status = Command::new("go")
        .current_dir("../neurahive-client")
        .args(vec!["build", "-o", "../target"])
        .status()
        .unwrap();

    println!("build neurahive-client with status {}", status);
}
