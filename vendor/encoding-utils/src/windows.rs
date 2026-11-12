pub use codepage::*;
use encoding_rs::Encoding;
use windows_sys::Win32::Globalization::GetACP;

pub type CodePage = u32;

/// Get Current ANSI Code Page via Windows API
pub fn current_acp() -> CodePage {
    unsafe { GetACP() }
}

pub fn current_acp_encoding() -> Option<&'static Encoding> {
    let acp = current_acp();
    to_encoding(acp as u16)
}

pub fn current_acp_encoding_no_replacement() -> Option<&'static Encoding> {
    let acp = current_acp();
    to_encoding_no_replacement(acp as u16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_current_acp() {
        let acp = current_acp();
        eprintln!("Current ANSI Code Page: {}", acp);
        assert_ne!(acp, 0);
    }

    #[test]
    fn test_get_current_acp_encoding() {
        let encoding = current_acp_encoding();
        eprintln!("Current ANSI Code Page Encoding: {:?}", encoding);
        assert!(encoding.is_some());
    }

    #[test]
    fn test_get_current_acp_encoding_no_replacement() {
        let encoding = current_acp_encoding_no_replacement();
        eprintln!(
            "Current ANSI Code Page Encoding (no replacement): {:?}",
            encoding
        );
        assert!(encoding.is_some());
    }
}
