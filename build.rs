fn main() {
    // Tell Rust to compile and link your C code
    println!("cargo:rerun-if-changed=c_code/win32.c");
    cc::Build::new()
        .file("c_code/win32.c")
        .compile("win32");
}
