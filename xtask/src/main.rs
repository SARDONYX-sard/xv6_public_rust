use std::fs::{File, read};
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};

fn main() {
    let mut args = std::env::args().skip(1);
    let subcommand = args.next();

    match subcommand.as_deref() {
        Some("image") => {
            let mut profile = "debug".to_string(); // default

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--profile" => {
                        if let Some(p) = args.next() {
                            profile = p;
                        } else {
                            eprintln!("--profile requires a value (e.g. debug or release)");
                            std::process::exit(1);
                        }
                    }
                    unknown => {
                        eprintln!("Unknown argument: {}", unknown);
                        std::process::exit(1);
                    }
                }
            }

            let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../target/i686-xv6-none")
                .join(&profile);

            build_crate("boot", &profile);
            build_crate("kernel", &profile);

            if let Err(e) = create_image(&target_dir) {
                eprintln!("Failed to create image: {}", e);
                std::process::exit(1);
            }
        }
        Some("qemu") => {
            let mut profile = "debug".to_string(); // default

            while let Some(arg) = args.next() {
                match arg.as_str() {
                    "--profile" => {
                        if let Some(p) = args.next() {
                            profile = p;
                        } else {
                            eprintln!("--profile requires a value (e.g. debug or release)");
                            std::process::exit(1);
                        }
                    }
                    unknown => {
                        eprintln!("Unknown argument: {}", unknown);
                        std::process::exit(1);
                    }
                }
            }

            let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../target/i686-xv6-none")
                .join(&profile);

            let img_path = target_dir.join("xv6.img");

            if !img_path.exists() {
                eprintln!(
                    "Image not found: {}\nDid you run `cargo xtask image` first?",
                    img_path.display()
                );
                std::process::exit(1);
            }

            run_qemu(&img_path);
        }
        _ => {
            eprintln!("Usage: cargo run -p xtask -- image [--profile <debug|release>]");
            std::process::exit(1);
        }
    }
}

fn build_crate(crate_name: &str, profile: &str) {
    println!("Building `{}` with profile `{}`", crate_name, profile);

    let target_json = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../app/os/kernel/targets/i686-xv6-none.json");
    let target_json = target_json.display().to_string();

    let mut cmd = Command::new("cargo");
    cmd.arg("build")
        .arg("-p")
        .arg(crate_name)
        .args(&["--target", &target_json])
        .env("CARGO_UNSTABLE_BUILD_STD", "1")
        .args(&["-Z", "build-std=core"])
        .args(&["--profile", profile]);

    let status = cmd.status().expect("Failed to run cargo build");
    if !status.success() {
        eprintln!("Failed to build `{}`", crate_name);
        std::process::exit(1);
    }
}

/// ```txt
/// [ 0x0000 ---------------------- ]
/// |                               |
/// |   bootloader image            |
/// |   ├─ 32 KiB zero padding      |
/// |   ├─ boot                     |
/// |   └─ (zero-filled up to 510)  |
/// |   0x55AA <- boot symbol       |
/// |                               |
/// [ 0x0200 ---------------------- ]
/// |                               |
/// |   kernel binary               |
/// |   └─ raw ELF or flat image    |
/// |                               |
/// [ end --------------------------]
/// ```
fn create_image(target_dir: &Path) -> std::io::Result<()> {
    let boot_bin = target_dir.join("boot.bin");
    {
        let boot = target_dir.join("boot");
        to_bin(&boot, &boot_bin).expect("cargo-objcopy boot failed");
    }
    let mut boot = read(&boot_bin).expect("Failed to read boot.bin binary");
    strip_trailing_zeros(&mut boot);

    // Add 0x55AA boot signature if missing
    const BOOT_SIGNATURE: &[u8] = &[0x55, 0xAA];
    let boot_len = boot.len();
    match boot_len {
        510 => boot.extend_from_slice(BOOT_SIGNATURE),
        len if len < 510 => {
            boot.extend(std::iter::repeat(0).take(510 - len));
            boot.extend_from_slice(BOOT_SIGNATURE);
        }
        _ => panic!("Bootloader is too large ({boot_len} bytes). need <= 510bytes",),
    }

    let kernel = read(target_dir.join("kernel")).expect("Failed to read kernel binary");

    let mut img = File::create(target_dir.join("xv6.img"))?;
    img.write_all(&boot)?;
    img.write_all(&kernel)?;

    println!("xv6.img created successfully in {}", target_dir.display());
    Ok(())
}

fn strip_trailing_zeros(data: &mut Vec<u8>) {
    while data.last() == Some(&0) {
        data.pop();
    }
}

fn to_bin(src: &Path, dst: &Path) -> std::io::Result<ExitStatus> {
    Command::new("rust-objcopy")
        // .arg("--binary-architecture=i386")
        // .arg("--strip-all")
        .arg("-S")
        .args(&["-O", "binary"])
        .arg(src)
        .arg(dst)
        .status()
}

fn run_qemu(img_path: &Path) {
    println!("Running QEMU with image: {}", img_path.display());

    let status = Command::new("qemu-system-i386")
        .arg("-drive")
        .arg(format!("format=raw,file={}", img_path.display()))
        .arg("-m")
        .arg("64M")
        .arg("-no-reboot")
        .arg("-serial")
        .arg("mon:stdio")
        // .arg("-display")
        // .arg("none")
        .args(&["-d", "guest_errors"])
        .args(&["-no-reboot", "-no-shutdown"])
        .status()
        .expect("Failed to execute qemu-system-i386");

    if !status.success() {
        eprintln!("QEMU exited with error code: {:?}", status.code());
        std::process::exit(1);
    }
}
