use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let out = PathBuf::from(env::var("OUT_DIR").unwrap());
    let linker_script = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("targets/i686-xv6-none.ld");
    let target = out.join("kernel.ld");

    fs::copy(&linker_script, &target).expect("Could not copy linker script to OUT_DIR");

    // Notification: Rebuild if link script has changed.
    println!("cargo:rerun-if-changed=kernel.ld");

    // Tell the linker the path.
    println!("cargo:rustc-link-arg=-T{}", target.display());
}
