fn is_musl() -> bool {
    std::env::var("CARGO_CFG_TARGET_ENV").unwrap() == "musl"
}

fn main() {
    if is_musl() {
        // Required for OpenSSL with musl
        println!("cargo:rustc-link-arg=-lc");
    }
}