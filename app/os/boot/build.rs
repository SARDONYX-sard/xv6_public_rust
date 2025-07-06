fn main() {
    println!("cargo:rustc-link-arg=-Ttext=0x7C00");
    println!("cargo:rustc-link-arg=-e");
    println!("cargo:rustc-link-arg=start");
}
