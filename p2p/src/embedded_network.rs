use std::{
    collections::HashSet,
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use futures::StreamExt;
use libp2p::{
    gossipsub::IdentTopic,
    kad::{self, record::Key as KadKey},
    swarm::SwarmEvent,
    Multiaddr, PeerId,
};
use serde::{Deserialize, Serialize};
use tokio::{runtime::Builder, sync::oneshot};

use crate::{
    cli,
    event_handler,
    keypair_from_seed,
    swarm_builder,
    utils::multiaddr_with_peer_id,
};

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

pub const DISCOVERY_TOPIC: &str = "gnostr/p2p/presence";
const DISCOVERY_KEY: &[u8] = b"gnostr/p2p/presence";
const DISCOVERY_REFRESH_SECS: u64 = 20;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerPresence {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub timestamp_secs: u64,
}

fn discovery_topic() -> IdentTopic {
    IdentTopic::new(DISCOVERY_TOPIC)
}

fn discovery_key() -> KadKey {
    KadKey::new(DISCOVERY_KEY)
}

fn timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn advertised_addresses(swarm: &libp2p::Swarm<crate::behaviour::Behaviour>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut addresses = Vec::new();

    for addr in swarm.external_addresses() {
        let addr = addr.to_string();
        if seen.insert(addr.clone()) {
            addresses.push(addr);
        }
    }

    addresses
}

pub fn publish_presence(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    peer_id: PeerId,
) -> Result<(), Box<dyn std::error::Error>> {
    let addresses = advertised_addresses(swarm);
    if addresses.is_empty() {
        push_log("skipping presence publish: no external addresses yet");
        return Ok(());
    }

    let presence = PeerPresence {
        peer_id: peer_id.to_string(),
        addresses,
        timestamp_secs: timestamp_secs(),
    };
    let payload = serde_json::to_vec(&presence)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(discovery_topic(), payload)?;
    push_log(format!("published presence for peer={peer_id}"));
    Ok(())
}

pub fn bootstrap_public_dht(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (addr, peer_id) in crate::network_config::Network::Ipfs.bootnodes() {
        let address_with_peer = multiaddr_with_peer_id(&addr, &peer_id);
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, address_with_peer.clone());
        swarm
            .behaviour_mut()
            .autonat
            .add_server(peer_id, Some(address_with_peer.clone()));
        if let Err(error) = swarm.dial(address_with_peer) {
            push_log(format!("failed to dial bootstrap peer {peer_id}: {error}"));
        }
    }

    if let Err(error) = swarm.behaviour_mut().kademlia.start_providing(discovery_key()) {
        push_log(format!("failed to start providing discovery key: {error}"));
    }

    match swarm.behaviour_mut().kademlia.bootstrap() {
        Ok(_) => push_log("started kademlia bootstrap"),
        Err(error) => push_log(format!("kademlia bootstrap deferred: {error}")),
    }

    Ok(())
}

pub fn refresh_wide_area_discovery(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    peer_id: PeerId,
) -> Result<(), Box<dyn std::error::Error>> {
    bootstrap_public_dht(swarm)?;
    publish_presence(swarm, peer_id)?;
    Ok(())
}

pub fn handle_presence_message(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let presence: PeerPresence = serde_json::from_slice(payload)?;
    let local_peer_id = swarm.local_peer_id().clone();
    let remote_peer_id: PeerId = presence.peer_id.parse()?;

    if remote_peer_id == local_peer_id {
        return Ok(());
    }

    let mut announced = 0usize;
    for addr in presence.addresses {
        let Ok(addr) = addr.parse::<Multiaddr>() else {
            continue;
        };

        let addr_with_peer = multiaddr_with_peer_id(&addr, &remote_peer_id);
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&remote_peer_id, addr_with_peer.clone());
        swarm
            .behaviour_mut()
            .autonat
            .add_server(remote_peer_id, Some(addr_with_peer.clone()));

        if let Err(error) = swarm.dial(addr_with_peer.clone()) {
            push_log(format!(
                "failed to dial presence peer {remote_peer_id} at {addr_with_peer}: {error}"
            ));
        } else {
            announced += 1;
        }
    }

    if announced > 0 {
        push_log(format!(
            "presence discovery announced peer={remote_peer_id} addrs={announced}"
        ));
    }

    Ok(())
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

            if let Err(error) = refresh_wide_area_discovery(&mut swarm, peer_id) {
                push_log(format!("wide-area discovery init failed: {error}"));
            }

            {
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("running p2p network peer={peer_id}");
            }
            push_log(format!("p2p network running peer={peer_id}"));

            let mut discovery_tick =
                tokio::time::interval(std::time::Duration::from_secs(DISCOVERY_REFRESH_SECS));
            discovery_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        push_log(format!("p2p network stopping peer={peer_id}"));
                        break;
                    }
                    _ = discovery_tick.tick() => {
                        if let Err(error) = refresh_wide_area_discovery(&mut swarm, peer_id) {
                            push_log(format!("wide-area discovery refresh failed: {error}"));
                        }
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
