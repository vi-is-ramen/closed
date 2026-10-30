fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if rustversion::cfg!(nightly) {
        println!("cargo:rustc-cfg=nightly");
    }
}
