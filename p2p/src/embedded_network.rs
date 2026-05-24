use std::{
    sync::{Arc, Mutex, OnceLock},
    thread,
};

use futures::StreamExt;
use libp2p::{kad, swarm::SwarmEvent};
use tokio::{runtime::Builder, sync::oneshot};

use crate::{cli, event_handler, keypair_from_seed, swarm_builder};

struct EmbeddedNetwork {
    status: Arc<Mutex<String>>,
    logs: Arc<Mutex<Vec<String>>>,
    shutdown: Option<oneshot::Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

static NETWORK: OnceLock<Mutex<Option<EmbeddedNetwork>>> = OnceLock::new();
static LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();

fn network_slot() -> &'static Mutex<Option<EmbeddedNetwork>> {
    NETWORK.get_or_init(|| Mutex::new(None))
}

fn logs_slot() -> &'static Mutex<Vec<String>> {
    LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn push_log(line: impl Into<String>) {
    let line = line.into();
    {
        let mut logs = logs_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logs.push(line.clone());
        if logs.len() > 500 {
            let drain_to = logs.len().saturating_sub(500);
            logs.drain(0..drain_to);
        }
    }

    let guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(state) = guard.as_ref() {
        let mut logs = state.logs.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logs.push(line);
        if logs.len() > 500 {
            let drain_to = logs.len().saturating_sub(500);
            logs.drain(0..drain_to);
        }
    }
}

pub fn log_line(line: impl Into<String>) {
    push_log(line);
}

fn with_status<F, T>(f: F) -> T
where
    F: FnOnce(&mut String) -> T,
{
    let guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(state) = guard.as_ref() else {
        return f(&mut String::from("not running"));
    };
    let mut status = state.status.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut status)
}

pub fn status() -> String {
    with_status(|status| status.clone())
}

pub fn logs() -> String {
    let logs = logs_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.join("\n")
}

pub fn clear_logs() {
    logs_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

pub fn start() -> String {
    let mut guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(state) = guard.as_ref() {
        push_log("p2p network already running");
        return state
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
    }

    clear_logs();
    let status = Arc::new(Mutex::new(String::from("starting p2p network")));
    let thread_status = Arc::clone(&status);
    let thread_logs = Arc::new(Mutex::new(Vec::new()));
    let thread_logs_clone = Arc::clone(&thread_logs);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();

    let join = thread::spawn(move || {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        runtime.block_on(async move {
            let keypair = keypair_from_seed(None);
            let peer_id = keypair.public().to_peer_id();
            push_log(format!("p2p network starting peer={peer_id}"));

            {
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("starting p2p network peer={peer_id}");
            }

            let mut swarm = match swarm_builder::build_swarm(keypair).await {
                Ok(swarm) => swarm,
                Err(error) => {
                    push_log(format!("p2p network failed to build: {error}"));
                    let mut status = thread_status
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    *status = format!("p2p network failed to build: {error}");
                    return;
                }
            };

            swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

            if let Err(error) = cli::listen_default_addresses(&mut swarm, None, 0, false) {
                push_log(format!("p2p network failed to listen: {error}"));
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("p2p network failed to listen: {error}");
                return;
            }

            {
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("running p2p network peer={peer_id}");
            }
            push_log(format!("p2p network running peer={peer_id}"));

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        push_log(format!("p2p network stopping peer={peer_id}"));
                        break;
                    }
                    event = swarm.select_next_some() => {
                        if let SwarmEvent::NewListenAddr { address, .. } = &event {
                            push_log(format!("p2p network listening peer={peer_id} addr={address}"));
                            let mut status = thread_status
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            *status = format!("running p2p network peer={peer_id} listen={address}");
                        }
                        event_handler::handle_swarm_event(&mut swarm, event).await;
                    }
                }
            }

            let mut status = thread_status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *status = format!("stopped p2p network peer={peer_id}");
            push_log(format!("p2p network stopped peer={peer_id}"));
        });
    });

    let snapshot = status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    *guard = Some(EmbeddedNetwork {
        status,
        logs: thread_logs_clone,
        shutdown: Some(shutdown_tx),
        join: Some(join),
    });
    snapshot
}

pub fn stop() -> String {
    let mut guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(mut state) = guard.take() else {
        return String::from("p2p network not running");
    };

    if let Some(shutdown) = state.shutdown.take() {
        let _ = shutdown.send(());
    }
    if let Some(join) = state.join.take() {
        let _ = join.join();
    }

    let status = state
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    push_log(status.clone());
    status
}
