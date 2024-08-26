use cc;

fn main() {
    println!("cargo:rerun-if-changed=src/xplaneconnect.c");
    println!("cargo:rerun-if-changed=src/xplaneconnect.h");

    cc::Build::new()
        .file("src/xplaneconnect.c")
        .flag("-Wno-pointer-sign")
        .flag("-Wno-unused-but-set-parameter")
        .flag("-Wno-tautological-constant-out-of-range-compare")
        .define("XPLANECONNECT_NO_LOG_ERRORS", "1")
        .include("include")
        .compile("xplaneconnect");
}
