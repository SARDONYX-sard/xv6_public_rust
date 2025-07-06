use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    // TODO: support other target arch.
    println!("cargo:rerun-if-changed=src/arch/x86/boot.asm");

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));

    let boot_o = out_dir.join("boot.o");

    let status = Command::new("nasm")
        .args(&[
            "-f",
            "elf32",
            "src/arch/x86/boot.asm",
            "-o",
            boot_o.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nasm");

    assert!(status.success(), "nasm failed");

    println!("cargo:rustc-link-arg={}", boot_o.to_str().unwrap());
}
