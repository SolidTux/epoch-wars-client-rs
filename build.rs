pub fn main() {
    #[cfg(target-os = "windows")]
    println!("cargo:rustc-link-search=target/sdl");
}
