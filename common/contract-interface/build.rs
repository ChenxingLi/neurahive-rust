use std::process::Command;

const INSTALL_ERROR_MESSAGE: &str =
    "Install dependencies for contract fail, try to run `yarn` in folder 'neurahive-contracts'";
const COMPILE_ERROR_MESSAGE: &str =
    "Compile solidity contracts fail, try to run `yarn compile` in folder 'neurahive-contracts'";

fn check_prerequisites() {
    // Check if node is installed
    let output = Command::new("node")
        .arg("-v")
        .output()
        .expect("Node is not installed or not found in PATH");

    let version = String::from_utf8(output.stdout).expect("Failed to parse node version");
    let major_version: i32 = version.split('.').next().unwrap_or("v0")[1..].parse().expect("Failed to parse major version");
    if major_version != 16 {
        panic!("Node v16 is required, but current version is: {}", version);
    }

    // Check if yarn is installed
    Command::new("yarn")
        .arg("--version")
        .output()
        .expect("Yarn is not installed or not found in PATH");
}


fn main() {
    if cfg!(feature = "compile-contracts") {
        println!("cargo:rerun-if-changed=../../neurahive-contracts/contracts/");
        println!("cargo:rerun-if-changed=../../neurahive-contracts/hardhat.config.ts");

        check_prerequisites();

        let output = Command::new("yarn")
            .arg("--cwd")
            .arg("../../neurahive-contracts")
            .status()
            .expect(INSTALL_ERROR_MESSAGE);
        assert!(output.success(), "{}", INSTALL_ERROR_MESSAGE);

        let output = Command::new("yarn")
            .arg("--cwd")
            .arg("../../neurahive-contracts")
            .arg("compile")
            .status()
            .expect(COMPILE_ERROR_MESSAGE);
        assert!(output.success(), "{}", COMPILE_ERROR_MESSAGE);
    } else {
        println!("cargo:rerun-if-changed=../../neurahive-contracts/artifacts/");
    }
}
