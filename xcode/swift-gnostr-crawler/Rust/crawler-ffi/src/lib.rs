use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    sync::mpsc,
    sync::{Mutex, OnceLock},
    thread,
    time::Duration,
};

use gnostr_crawler::{
    dispatch_cli_command, init_tracing, Cli,
    query::build_gnostr_query,
    relay_fetch::websocket_http_url,
    relay_metadata::Relay,
    run_api_server_with_shutdown_and_ready,
};
use clap::Parser;
use serde::{Deserialize, Serialize};
use tokio::{runtime::Builder, sync::oneshot};

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

#[derive(Debug, Deserialize)]
struct RuntimeRequest {
    port: Option<u16>,
}

#[derive(Debug, Deserialize)]
struct CrawlRequest {}

#[derive(Debug, Serialize)]
struct RuntimeState {
    running: bool,
    pid: Option<u32>,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    disk_usage_bytes: Option<u64>,
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

fn runtime_state(running: bool, message: impl Into<String>) -> RuntimeState {
    RuntimeState {
        running,
        pid: None,
        message: message.into(),
        disk_usage_bytes: None,
    }
}

struct RuntimeHandle {
    port: u16,
    stop_tx: Option<oneshot::Sender<()>>,
    join: thread::JoinHandle<()>,
}

struct CrawlHandle {
    stop_tx: Option<oneshot::Sender<()>>,
    join: thread::JoinHandle<()>,
}

static RUNTIME: OnceLock<Mutex<Option<RuntimeHandle>>> = OnceLock::new();
static CRAWL_RUNTIME: OnceLock<Mutex<Option<CrawlHandle>>> = OnceLock::new();

fn runtime_slot() -> &'static Mutex<Option<RuntimeHandle>> {
    RUNTIME.get_or_init(|| Mutex::new(None))
}

fn crawl_runtime_slot() -> &'static Mutex<Option<CrawlHandle>> {
    CRAWL_RUNTIME.get_or_init(|| Mutex::new(None))
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
pub unsafe extern "C" fn crawler_runtime_start_json(request_json: *const c_char) -> *mut c_char {
    let request_json = match read_c_string(request_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let request: RuntimeRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };
    let port = request.port.unwrap_or(3030);

    let slot = runtime_slot();
    let mut guard = slot.lock().unwrap();
    if let Some(existing) = guard.as_ref() {
        return encode(&Envelope::ok(serde_json::to_string(&runtime_state(
            true,
            format!("crawler runtime already running on port {}", existing.port),
        ))
        .unwrap()));
    }

    std::env::set_var("RUST_LOG", "debug");
    if let Err(error) = init_tracing() {
        eprintln!("crawler runtime tracing init: {}", error);
    }
    eprintln!("crawler runtime start: tracing initialized for port {}", port);

    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let join = thread::spawn(move || {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("gnostr-crawler-ffi")
            .build();

        let Ok(runtime) = runtime else {
            eprintln!("crawler runtime failed to build tokio runtime");
            return;
        };

        let _ = runtime.block_on(async move {
            tracing::info!("crawler runtime starting on port {}", port);
            let shutdown = async move {
                let _ = stop_rx.await;
            };
            match run_api_server_with_shutdown_and_ready(port, shutdown, Some(ready_tx)).await {
                Ok(_) => tracing::info!("crawler runtime stopped on port {}", port),
                Err(error) => tracing::error!("crawler runtime failed: {}", error),
            }
        });
    });

    match ready_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(Ok(())) => {
            eprintln!("crawler runtime start: ready on port {}", port);
            *guard = Some(RuntimeHandle {
                port,
                stop_tx: Some(stop_tx),
                join,
            });

            encode(&Envelope::ok(serde_json::to_string(&runtime_state(
                true,
                format!("crawler runtime running on port {}", port),
            ))
            .unwrap()))
        }
        Ok(Err(error)) => {
            eprintln!("crawler runtime start: failed ready on port {}: {}", port, error);
            let _ = stop_tx.send(());
            let _ = join.join();
            encode(&Envelope::<String>::err(format!("crawler runtime failed to start: {}", error)))
        }
        Err(_) => {
            eprintln!("crawler runtime start: timed out on port {}", port);
            let _ = stop_tx.send(());
            let _ = join.join();
            encode(&Envelope::<String>::err("crawler runtime start timed out"))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn crawler_runtime_stop_json(_request_json: *const c_char) -> *mut c_char {
    let slot = runtime_slot();
    let mut guard = slot.lock().unwrap();
    let Some(mut handle) = guard.take() else {
        return encode(&Envelope::ok(serde_json::to_string(&runtime_state(
            false,
            "crawler runtime not running",
        ))
        .unwrap()));
    };

    if let Some(stop_tx) = handle.stop_tx.take() {
        let _ = stop_tx.send(());
    }

    drop(guard);
    let _ = handle.join.join();
    encode(&Envelope::ok(serde_json::to_string(&runtime_state(
        false,
        format!("crawler runtime stopped on port {}", handle.port),
    ))
    .unwrap()))
}

#[no_mangle]
pub unsafe extern "C" fn crawler_runtime_status_json(_request_json: *const c_char) -> *mut c_char {
    let slot = runtime_slot();
    let guard = slot.lock().unwrap();
    let state = guard.as_ref().map(|handle| {
        runtime_state(true, format!("crawler runtime running on port {}", handle.port))
    }).unwrap_or_else(|| runtime_state(false, "crawler runtime not running"));
    encode(&Envelope::ok(serde_json::to_string(&state).unwrap()))
}

#[no_mangle]
pub unsafe extern "C" fn crawler_crawl_start_json(request_json: *const c_char) -> *mut c_char {
    let request_json = match read_c_string(request_json) {
        Ok(value) => value,
        Err(error) => return encode(&Envelope::<String>::err(error)),
    };

    let _request: CrawlRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(error) => return encode(&Envelope::<String>::err(error.to_string())),
    };

    let slot = crawl_runtime_slot();
    let mut guard = slot.lock().unwrap();
    if guard.is_some() {
        return encode(&Envelope::ok(serde_json::to_string(&runtime_state(
            true,
            "crawler crawl already running",
        ))
        .unwrap()));
    }

    std::env::set_var("RUST_LOG", "debug");
    if let Err(error) = init_tracing() {
        eprintln!("crawler crawl tracing init: {}", error);
    }
    eprintln!("crawler crawl start: tracing initialized");

    let (stop_tx, stop_rx) = oneshot::channel::<()>();
    let (ready_tx, ready_rx) = mpsc::channel::<Result<(), String>>();
    let join = thread::spawn(move || {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .thread_name("gnostr-crawler-crawl")
            .build();

        let Ok(runtime) = runtime else {
            eprintln!("crawler crawl failed to build tokio runtime");
            return;
        };

        let _ = runtime.block_on(async move {
            let cli = match Cli::try_parse_from(["gnostr", "crawl"]) {
                Ok(cli) => cli,
                Err(error) => {
                    let _ = ready_tx.send(Err(error.to_string()));
                    return;
                }
            };
            let client = reqwest::Client::new();
            let _ = ready_tx.send(Ok(()));
            let result = tokio::select! {
                result = dispatch_cli_command(cli, &client) => {
                    result.map_err(|error| error.to_string())
                }
                _ = stop_rx => Ok(()),
            };
            match result {
                Ok(()) => eprintln!("crawler crawl stopped"),
                Err(error) => eprintln!("crawler crawl failed: {}", error),
            }
        });
    });

    match ready_rx.recv_timeout(Duration::from_secs(15)) {
        Ok(Ok(())) => {
            *guard = Some(CrawlHandle {
                stop_tx: Some(stop_tx),
                join,
            });
            encode(&Envelope::ok(serde_json::to_string(&runtime_state(
                true,
                "crawler crawl running",
            ))
            .unwrap()))
        }
        Ok(Err(error)) => {
            let _ = stop_tx.send(());
            let _ = join.join();
            encode(&Envelope::<String>::err(format!("crawler crawl failed to start: {}", error)))
        }
        Err(_) => {
            let _ = stop_tx.send(());
            let _ = join.join();
            encode(&Envelope::<String>::err("crawler crawl start timed out"))
        }
    }
}

#[no_mangle]
pub unsafe extern "C" fn crawler_crawl_stop_json(_request_json: *const c_char) -> *mut c_char {
    let slot = crawl_runtime_slot();
    let mut guard = slot.lock().unwrap();
    let Some(mut handle) = guard.take() else {
        return encode(&Envelope::ok(serde_json::to_string(&runtime_state(
            false,
            "crawler crawl not running",
        ))
        .unwrap()));
    };

    if let Some(stop_tx) = handle.stop_tx.take() {
        let _ = stop_tx.send(());
    }

    drop(guard);
    let _ = handle.join.join();
    encode(&Envelope::ok(serde_json::to_string(&runtime_state(
        false,
        "crawler crawl stopped",
    ))
    .unwrap()))
}

#[no_mangle]
pub unsafe extern "C" fn crawler_crawl_status_json(_request_json: *const c_char) -> *mut c_char {
    let slot = crawl_runtime_slot();
    let guard = slot.lock().unwrap();
    let state = if guard.is_some() {
        runtime_state(true, "crawler crawl running")
    } else {
        runtime_state(false, "crawler crawl not running")
    };
    encode(&Envelope::ok(serde_json::to_string(&state).unwrap()))
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
