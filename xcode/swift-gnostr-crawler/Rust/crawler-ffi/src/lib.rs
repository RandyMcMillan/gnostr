use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
};

use gnostr_crawler::query::build_gnostr_query;
use gnostr_crawler::relay_fetch::websocket_http_url;
use gnostr_crawler::relay_metadata::Relay;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct Envelope<T> {
    ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

impl<T> Envelope<T> {
    fn ok(data: T) -> Self {
        Self { ok: true, data: Some(data), error: None }
    }

    fn err(error: impl Into<String>) -> Self {
        Self { ok: false, data: None, error: Some(error.into()) }
    }
}

#[derive(Debug, Deserialize)]
struct QueryRequest {
    authors: Option<String>,
    ids: Option<String>,
    limit: Option<i32>,
    generic_tag: Option<String>,
    generic_value: Option<String>,
    hashtag: Option<String>,
    mentions: Option<String>,
    references: Option<String>,
    kinds: Option<String>,
    search: Option<String>,
}

fn to_c_string(json: String) -> *mut c_char {
    CString::new(json)
        .unwrap_or_else(|_| CString::new(r#"{"ok":false,"error":"embedded nul"}"#).unwrap())
        .into_raw()
}

fn encode<T: Serialize>(envelope: &Envelope<T>) -> *mut c_char {
    let json = serde_json::to_string(envelope).unwrap_or_else(|error| {
        format!(r#"{{"ok":false,"error":"failed to serialize response: {error}"}}"#)
    });
    to_c_string(json)
}

unsafe fn read_c_string<'a>(ptr: *const c_char) -> Result<&'a str, String> {
    if ptr.is_null() {
        return Err("null pointer".to_string());
    }
    CStr::from_ptr(ptr).to_str().map_err(|error| error.to_string())
}

#[no_mangle]
pub unsafe extern "C" fn crawler_string_free(ptr: *mut c_char) {
    if !ptr.is_null() {
        drop(CString::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn crawler_build_gnostr_query_json(request_json: *const c_char) -> *mut c_char {
    let request_json = match read_c_string(request_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let request: QueryRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    let generic = match (request.generic_tag.as_deref(), request.generic_value.as_deref()) {
        (Some(tag), Some(value)) => Some((tag, value)),
        _ => None,
    };
    let search = request.search.as_deref().map(|value| ("search", value));

    match build_gnostr_query(
        request.authors.as_deref(),
        request.ids.as_deref(),
        request.limit,
        generic,
        request.hashtag.as_deref(),
        request.mentions.as_deref(),
        request.references.as_deref(),
        request.kinds.as_deref(),
        search,
    ) {
        Ok(query) => encode(&Envelope::ok(query)),
        Err(error) => encode(&Envelope::<String>::err(error.to_string())),
    }
}

#[no_mangle]
pub unsafe extern "C" fn crawler_websocket_http_url_json(url: *const c_char) -> *mut c_char {
    let url = match read_c_string(url) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    encode(&Envelope::ok(websocket_http_url(url)))
}

#[no_mangle]
pub unsafe extern "C" fn crawler_roundtrip_relay_metadata_json(relay_json: *const c_char) -> *mut c_char {
    let relay_json = match read_c_string(relay_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let relay: Relay = match serde_json::from_str(relay_json) {
        Ok(relay) => relay,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    match serde_json::to_string(&relay) {
        Ok(serialized) => encode(&Envelope::ok(serialized)),
        Err(error) => encode(&Envelope::<String>::err(error.to_string())),
    }
}
