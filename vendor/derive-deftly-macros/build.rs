// build.rs for derive-deftly-macros

// We recompile some of our macro code in tests, with cargo
// features that aren't declared in the package's Cargo.toml.
// And, we have a nonstandard RUSTFLAGS --cfg=derive_deftly_dprint
// for debugging output.
//
// Here, we tell rustc, via cargo, that these are all allowed.
//
// Another effect is that this build.rs file
// causes OUT_DIR to be available in `check.rs`.

// This is a subset of tests/build.rs, written to be as minimal
// (and fast to compile) as possible.

fn main() {
    // We must use deprecated single colon `cargo:rustc...` syntax,
    // because otherwise cargo thinks we're violating our MSRV.
    //  https://github.com/rust-lang/cargo/issues/14147
    println!(
        r#"cargo:rustc-check-cfg=cfg(derive_deftly_dprint)
cargo:rustc-check-cfg=cfg(feature, values("bizarre"))"#
    );
}
