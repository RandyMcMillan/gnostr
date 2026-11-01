# Encoding utils

A utils to help with encoding and decoding os strings and more.

* [x] on windows it will use `GetACP` to get the code page, and return the `Encoding` instance from `encoding_rs` crate.
* [ ] on linux, we should detect the encoding from the locale, and return the `Encoding` instance from `encoding_rs` crate.
* on macOs, it seems that the encoding is always `UTF-8`, so we can always get the `Encoding` instance from `encoding_rs` crate.
