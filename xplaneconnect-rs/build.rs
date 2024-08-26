use cc;

fn main() {
    println!("cargo:rerun-if-changed=src/XPlaneConnect/xplaneconnect.c");
    println!("cargo:rerun-if-changed=src/XPlaneConnect/xplaneconnect.h");

    cc::Build::new()
        .file("src/XPlaneConnect/xplaneconnect.c")
        .flag("-Wno-pointer-sign")
        .flag("-Wno-unused-but-set-parameter")
        .flag("-Wno-tautological-constant-out-of-range-compare")
        .define("XPLANECONNECT_NO_LOG_ERRORS", "1")
        .include("src/XPlaneConnect")
        .compile("xplaneconnect");
}
