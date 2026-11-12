//! Conditional compilation

#[allow(unused_macros)] // might be redefined to expand to nothing
macro_rules! if_cfg_getresuid { { $($y:tt)* } => { $($y)* } }

// Sadly we can't use this in struct MockableLibcFunctions in lmockable.rs
#[cfg(target_os = "macos")]
macro_rules! if_cfg_getresuid { { $($y:tt)* } => { } }
