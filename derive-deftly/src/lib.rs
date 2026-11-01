// This is the "library" crate.
//
// It is conventional for proc macro crates to be wrapped up
// in a library crate, which reexport the proc macros, mostly because
// a proc macro crate cannot contain anything else.
//
// Currently our plans do not appear to include any items other than
// proc macros.  But we shouldn't foreclose that.  So for now
// this library crate ought to declare a dependency on the macro crate
// and reexport the macros.
//
// Also this crate is where the docs live, right here as crate-level
// docs.

#![allow(clippy::style, clippy::complexity)]
#![allow(rustdoc::invalid_html_tags)] // @derive-deftly invalid-html-tags
#![doc=include_str!("../README.md")]
//
// We take some serious liberties with rustdoc's HTML, here.
// We want this "overall TOC" to be an h1 heading
// and we want it *not* to be within the div
// that rustdoc puts this this whole toplevel docs in.
// So we *close that div* and reopen a new one.
//!
//! </div><!-- @derive-deftly invalid-html-tags -->
//!
//! <h1 class="section-header main-heading" id="overall-toc"><a class="doc-anchor" href="#overall-toc">§</a><strong>derive-deftly documentation, overall table of contents</strong></h1>
//!
//! <div class="docblock">
//!
//! <!-- @dd-navbar overall-toc . -->
//! <!-- this line automatically maintained by update-navbars --><nav style="text-align: right; margin-bottom: 12px;">[ <em>docs: <a href="index.html">crate top-level</a> | <strong>overall toc, macros</strong> | <a href="doc_reference/index.html">template etc. reference</a> | <a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">guide/tutorial</a></em> ]</nav>
//!
//! </div>
//! <div><!-- @derive-deftly invalid-html-tags -->
//! <p style="margin-top: 25px"></p>
//!
// This next part is is also unwarranted chumminess with rustdoc.
// Some time between Rust 1.85 and 1.86, rustdoc changed from using
// <ul> and a pair of <div> with <class>, to this <dd>, which is better.
// This does mean this doesn't render precisely right on Stable right now.
//
//! <dl class="item-table">
//! <dt><a href="https://diziet.pages.torproject.net/rust-derive-deftly/latest/guide/">Guide</a></dt>
//! <dd>Tutorial and walkthrough, in book form</dd>
//! </dl>

pub use derive_deftly_macros::derive_deftly_engine;
pub use derive_deftly_macros::{
    define_derive_deftly, derive_deftly_adhoc, template_export_semver_check,
    Deftly,
};

// We (ab)use the module system as places to hang our documentation.

#[deny(rustdoc::invalid_html_tags)]
#[doc=include_str!("../doc/reference.md")]
pub mod doc_reference {}

#[deny(rustdoc::invalid_html_tags)]
#[doc=include_str!("../doc/implementation.md")]
pub mod doc_implementation {}

#[deny(rustdoc::invalid_html_tags)]
#[doc=include_str!("../CHANGELOG.md")]
pub mod doc_changelog {}

#[cfg(not(feature = "minimal-1"))]
compile_error! { "You must enable (directly or indirectly) the derive-deftly crate feature `minimal-1`!" }
