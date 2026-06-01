//! Recursive packet framing and parity helpers for the `perfect_ip` protocol.
//!
//! The protocol is intentionally small and reusable:
//! - [`process_slice`] recursively splits payloads until each emitted packet
//!   stays within the transport budget.
//! - [`packetize`] finalizes a batch and fills in packet counts up front.
//! - [`recover_missing_data`] and [`recover_missing_slice`] rebuild a missing
//!   packet from a sibling packet and parity slice.
//!
//! The parity model is XOR-based and matches the packet-tree sketch in the
//! `perfect_ip.rs` gist.

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::io;
use std::path::Path;

use futures::StreamExt;
use libp2p::{
    request_response,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    StreamProtocol,
};
use serde::{Deserialize, Serialize};

/// Packet header metadata shared by all packet types in the tree.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Header {
    /// Monotonic sequence number assigned during packetization.
    pub seq_num: u32,
    /// Total number of packets in the finalized batch.
    pub total_packets: u32,
}

/// A single packet produced by the recursive packetizer.
///
/// `is_parity` marks parity frames that describe the XOR of sibling payloads.
/// `id` carries the recursive path for the packet, such as `ROOT.0.1.P`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProtocolSlice {
    /// Stable recursive packet identifier.
    pub id: String,
    /// Packet sequencing metadata.
    pub header: Header,
    /// Raw payload bytes for the packet or parity frame.
    pub data: Vec<u8>,
    /// `true` when this slice is a parity frame.
    pub is_parity: bool,
}

/// Finalized packet tree output.
///
/// `total_packets` is duplicated here so callers can inspect batch metadata
/// without walking the packet list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PacketBatch {
    /// Number of packets in the batch.
    pub total_packets: u32,
    /// Finalized packets with matching `total_packets` headers.
    pub packets: Vec<ProtocolSlice>,
}

/// libp2p repair behaviour for exchanging packet slices.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "FractalBehaviourEvent")]
pub struct FractalBehaviour {
    /// CBOR request/response channel for packet slice repair.
    pub repair_rpc: request_response::cbor::Behaviour<ProtocolSlice, ProtocolSlice>,
}

/// Behaviour events emitted by the repair RPC layer.
#[derive(Debug)]
pub enum FractalBehaviourEvent {
    /// A request/response protocol event.
    RepairRpc(request_response::Event<ProtocolSlice, ProtocolSlice>),
}

impl From<request_response::Event<ProtocolSlice, ProtocolSlice>> for FractalBehaviourEvent {
    fn from(event: request_response::Event<ProtocolSlice, ProtocolSlice>) -> Self {
        Self::RepairRpc(event)
    }
}

/// Tracks the expected manifest and the packets that have arrived so far.
#[derive(Debug, Clone, Default)]
pub struct IntegrityManager {
    manifest: HashSet<String>,
    pub received_slices: HashMap<String, ProtocolSlice>,
}

impl IntegrityManager {
    /// Create a manager from the set of expected packet ids.
    pub fn new(expected_ids: Vec<String>) -> Self {
        Self {
            manifest: expected_ids.into_iter().collect(),
            received_slices: HashMap::new(),
        }
    }

    /// Record a received packet by id, replacing any prior packet with the same id.
    pub fn record_slice(&mut self, slice: ProtocolSlice) {
        self.received_slices.insert(slice.id.clone(), slice);
    }

    /// Persist the received packet map to disk using bincode.
    pub fn persist(&self, path: impl AsRef<Path>) -> io::Result<()> {
        let encoded = bincode::serialize(&self.received_slices)
            .map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        std::fs::write(path, encoded)
    }

    /// Load a persisted packet map from disk and attach it to the given manifest.
    pub fn load_from_disk(
        path: impl AsRef<Path>,
        manifest: HashSet<String>,
    ) -> io::Result<Self> {
        let bytes = std::fs::read(path)?;
        let received_slices: HashMap<String, ProtocolSlice> =
            bincode::deserialize(&bytes).map_err(|error| io::Error::new(io::ErrorKind::Other, error))?;
        Ok(Self {
            manifest,
            received_slices,
        })
    }

    /// Return the packet ids from the manifest that are still missing.
    pub fn get_missing_nodes(&self) -> Vec<String> {
        self.manifest
            .iter()
            .filter(|id| !self.received_slices.contains_key(*id))
            .cloned()
            .collect()
    }

    /// Verify that every parity slice matches the XOR of its sibling data slices.
    pub fn verify_integrity(&self) -> bool {
        for (id, slice) in &self.received_slices {
            if !slice.is_parity {
                continue;
            }

            let Some(base_id) = id.strip_suffix(".P") else {
                return false;
            };
            let left = self.received_slices.get(&format!("{}.0", base_id));
            let right = self.received_slices.get(&format!("{}.1", base_id));

            if let (Some(left), Some(right)) = (left, right) {
                if calculate_parity(&left.data, &right.data) != slice.data {
                    return false;
                }
            }
        }

        true
    }
}

/// Build the repair swarm used to exchange packet slices with peers.
pub async fn build_fractal_swarm(
    local_key: libp2p::identity::Keypair,
) -> Result<Swarm<FractalBehaviour>, Box<dyn Error + Send + Sync>> {
    let swarm = libp2p::SwarmBuilder::with_existing_identity(local_key)
        .with_tokio()
        .with_quic()
        .with_behaviour(|_| {
            Ok::<_, Box<dyn Error + Send + Sync>>(FractalBehaviour {
                repair_rpc: request_response::cbor::Behaviour::new(
                    [(
                        StreamProtocol::new("/fractal/repair/1.0.0"),
                        request_response::ProtocolSupport::Full,
                    )],
                    request_response::Config::default(),
                ),
            })
        })?
        .build();

    Ok(swarm)
}

/// Start the repair swarm and forward incoming repair requests to the packet map.
///
/// This is a minimal networking entrypoint for the packet protocol; it listens
/// for `ProtocolSlice` repair requests and replies with the matching slice when
/// the packet is present in `IntegrityManager`.
pub async fn run_fractal_engine(
    local_key: libp2p::identity::Keypair,
    listen_address: libp2p::Multiaddr,
    manager: IntegrityManager,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let mut swarm = build_fractal_swarm(local_key).await?;
    swarm.listen_on(listen_address)?;

    loop {
        match swarm.select_next_some().await {
            SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) => {
                if let request_response::Event::Message { message, .. } = event {
                    if let request_response::Message::Request {
                        request, channel, ..
                    } = message
                    {
                        let response = manager
                            .received_slices
                            .get(&request.id)
                            .cloned()
                            .unwrap_or_else(|| ProtocolSlice {
                                id: request.id.clone(),
                                header: request.header.clone(),
                                data: Vec::new(),
                                is_parity: false,
                            });
                        let _ = swarm
                            .behaviour_mut()
                            .repair_rpc
                            .send_response(channel, response);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Maximum payload size this packet protocol is allowed to emit.
pub const MTU_PAYLOAD: usize = 1460;
const MAX_LEAF_PAYLOAD: usize = MTU_PAYLOAD / 2;

/// XOR two payloads into a parity buffer.
///
/// Missing bytes are treated as zero so the returned buffer is as long as the
/// larger input.
pub fn calculate_parity(left: &[u8], right: &[u8]) -> Vec<u8> {
    let max_len = left.len().max(right.len());
    let mut parity = vec![0; max_len];
    for i in 0..max_len {
        let l = if i < left.len() { left[i] } else { 0 };
        let r = if i < right.len() { right[i] } else { 0 };
        parity[i] = l ^ r;
    }
    parity
}

/// Generate the expected packet ids for a recursive packet tree.
///
/// The returned manifest mirrors [`process_slice`] exactly: leaf nodes are
/// emitted when the payload fits within [`MAX_LEAF_PAYLOAD`], and internal
/// nodes append `.0`, `.1`, and `.P` entries in tree order.
pub fn generate_manifest(id: String, len: usize) -> Vec<String> {
    if len <= MAX_LEAF_PAYLOAD {
        return vec![id];
    }

    let half = len / 2;
    let mut ids = generate_manifest(format!("{}.0", id), half);
    ids.append(&mut generate_manifest(format!("{}.1", id), len - half));
    ids.push(format!("{}.P", id));
    ids
}

/// Recover the missing payload by XORing the sibling payload and parity frame.
///
/// `expected_len` trims the recovered buffer back to the original payload length.
pub fn recover_missing_data(expected_len: usize, sibling: &[u8], parity: &[u8]) -> Vec<u8> {
    let recovered = calculate_parity(sibling, parity);
    recovered.into_iter().take(expected_len).collect()
}

/// Rebuild a missing packet using a sibling packet and parity packet.
///
/// The returned slice is marked as data and receives the next sequence number.
pub fn recover_missing_slice(
    id: String,
    expected_len: usize,
    sibling: &ProtocolSlice,
    parity: &ProtocolSlice,
    seq: &mut u32,
) -> ProtocolSlice {
    let header = Header {
        seq_num: *seq,
        total_packets: sibling.header.total_packets.max(parity.header.total_packets),
    };
    *seq += 1;

    ProtocolSlice {
        id,
        header,
        data: recover_missing_data(expected_len, &sibling.data, &parity.data),
        is_parity: false,
    }
}

/// Recursively split a payload into MTU-safe slices and parity frames.
///
/// Leaf packets are emitted when the payload is at or below
/// [`MAX_LEAF_PAYLOAD`]. Internal nodes emit left, right, and parity frames.
pub fn process_slice(id: String, data: Vec<u8>, seq: &mut u32) -> Vec<ProtocolSlice> {
    if data.len() <= MAX_LEAF_PAYLOAD {
        let slice = ProtocolSlice {
            id,
            header: Header {
                seq_num: *seq,
                total_packets: 0,
            },
            data,
            is_parity: false,
        };
        *seq += 1;
        return vec![slice];
    }

    let half = data.len() / 2;
    let left_data = data[..half].to_vec();
    let right_data = data[half..].to_vec();

    let parity = calculate_parity(&left_data, &right_data);
    let mut slices = process_slice(format!("{}.0", id), left_data, seq);
    slices.append(&mut process_slice(format!("{}.1", id), right_data, seq));

    slices.push(ProtocolSlice {
        id: format!("{}.P", id),
        header: Header {
            seq_num: *seq,
            total_packets: 0,
        },
        data: parity,
        is_parity: true,
    });
    *seq += 1;

    slices
}

/// Packetize a payload and finalize the batch metadata.
///
/// This is the preferred entrypoint for callers that want a ready-to-send
/// packet tree with `total_packets` filled in.
pub fn packetize(id: String, data: Vec<u8>) -> PacketBatch {
    let mut seq = 0;
    let mut packets = process_slice(id, data, &mut seq);
    let total_packets = packets.len() as u32;
    for packet in &mut packets {
        packet.header.total_packets = total_packets;
    }
    PacketBatch {
        total_packets,
        packets,
    }
}

/// Render packet summaries for logs or diagnostics.
pub fn summarize_packets(packets: &[ProtocolSlice]) -> Vec<String> {
    packets
        .iter()
        .map(|packet| {
            format!(
                "ID: {:<8} | Seq: {:>2}/{} | Type: {:<6} | Size: {}B",
                packet.id,
                packet.header.seq_num,
                packet.header.total_packets,
                if packet.is_parity { "PARITY" } else { "DATA" },
                packet.data.len()
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keypair_from_seed;
    use libp2p::Multiaddr;
    use tempfile::NamedTempFile;

    #[test]
    fn process_slice_emits_parity_and_data() {
        let raw_data = vec![0xAB; 2000];
        let batch = packetize("ROOT".to_string(), raw_data);
        let packets = batch.packets;

        assert!(!packets.is_empty());
        assert!(packets.iter().any(|packet| packet.is_parity));
        assert!(packets.iter().any(|packet| packet.id == "ROOT.P"));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.0.")));
        assert!(packets.iter().any(|packet| packet.id.starts_with("ROOT.1.")));
        assert!(packets.iter().all(|packet| packet.data.len() <= MTU_PAYLOAD));
        assert!(packets
            .iter()
            .all(|packet| packet.header.total_packets == batch.total_packets));
    }

    #[test]
    fn calculate_parity_xors_equal_length_blocks() {
        let left = [0xDE, 0xAD, 0xBE];
        let right = [0x01, 0x02, 0x03];
        assert_eq!(calculate_parity(&left, &right), vec![0xDF, 0xAF, 0xBD]);
    }

    #[test]
    fn generate_manifest_matches_recursive_packet_tree() {
        let manifest = generate_manifest("ROOT".to_string(), 2000);

        assert!(manifest.contains(&"ROOT.P".to_string()));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.0.")));
        assert!(manifest.iter().any(|id| id.starts_with("ROOT.1.")));
        assert_eq!(manifest.len(), packetize("ROOT".to_string(), vec![0xAB; 2000]).total_packets as usize);
    }

    #[test]
    fn recover_missing_data_restores_xor_partner() {
        let left = vec![0xDE, 0xAD, 0xBE];
        let right = vec![0x01, 0x02, 0x03];
        let parity = calculate_parity(&left, &right);

        assert_eq!(recover_missing_data(left.len(), &right, &parity), left);
        assert_eq!(recover_missing_data(right.len(), &left, &parity), right);
    }

    #[test]
    fn recover_missing_slice_rebuilds_header_and_payload() {
        let sibling = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: vec![0xDF, 0xAF, 0xBD],
            is_parity: true,
        };
        let mut seq = 3;

        let recovered = recover_missing_slice("ROOT.0".to_string(), 3, &sibling, &parity, &mut seq);

        assert_eq!(recovered.id, "ROOT.0");
        assert_eq!(recovered.data, vec![0xDE, 0xAD, 0xBE]);
        assert_eq!(recovered.header.seq_num, 3);
        assert_eq!(recovered.header.total_packets, 3);
        assert!(!recovered.is_parity);
    }

    #[test]
    fn get_missing_nodes_returns_unseen_manifest_entries() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE],
            is_parity: false,
        });

        let missing = manager.get_missing_nodes();
        assert_eq!(missing.len(), 2);
        assert!(missing.contains(&"ROOT.1".to_string()));
        assert!(missing.contains(&"ROOT.P".to_string()));
    }

    #[test]
    fn verify_integrity_checks_parity_frames() {
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        let left = ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        };
        let right = ProtocolSlice {
            id: "ROOT.1".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0x01, 0x02, 0x03],
            is_parity: false,
        };
        let parity = ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 2,
                total_packets: 3,
            },
            data: calculate_parity(&left.data, &right.data),
            is_parity: true,
        };

        manager.record_slice(left);
        manager.record_slice(right);
        manager.record_slice(parity);
        assert!(manager.verify_integrity());

        let mut corrupted = manager.clone();
        corrupted
            .received_slices
            .get_mut("ROOT.P")
            .expect("parity slice")
            .data[0] ^= 0xFF;
        assert!(!corrupted.verify_integrity());
    }

    #[test]
    fn persist_and_load_from_disk_round_trips_received_slices() {
        let temp = NamedTempFile::new().expect("temp file");
        let mut manager = IntegrityManager::new(vec![
            "ROOT.0".to_string(),
            "ROOT.1".to_string(),
            "ROOT.P".to_string(),
        ]);
        manager.record_slice(ProtocolSlice {
            id: "ROOT.0".to_string(),
            header: Header {
                seq_num: 0,
                total_packets: 3,
            },
            data: vec![0xDE, 0xAD, 0xBE],
            is_parity: false,
        });
        manager.record_slice(ProtocolSlice {
            id: "ROOT.P".to_string(),
            header: Header {
                seq_num: 1,
                total_packets: 3,
            },
            data: vec![0xAA],
            is_parity: true,
        });

        manager.persist(temp.path()).expect("persist");
        let loaded = IntegrityManager::load_from_disk(
            temp.path(),
            vec![
                "ROOT.0".to_string(),
                "ROOT.1".to_string(),
                "ROOT.P".to_string(),
            ]
            .into_iter()
            .collect(),
        )
        .expect("load");

        assert_eq!(loaded.received_slices.len(), 2);
        assert!(loaded.received_slices.contains_key("ROOT.0"));
        assert!(loaded.received_slices.contains_key("ROOT.P"));
        assert!(loaded.get_missing_nodes().contains(&"ROOT.1".to_string()));
    }

    #[tokio::test]
    async fn build_fractal_swarm_accepts_quic_listen_address() {
        let keypair = keypair_from_seed(Some(
            gnostr_asyncgit::default_gnostr_private_key_hex(),
        ));
        let mut swarm = build_fractal_swarm(keypair).await.expect("swarm");
        let addr: Multiaddr = "/ip4/127.0.0.1/udp/0/quic-v1"
            .parse()
            .expect("quic multiaddr");

        swarm.listen_on(addr).expect("quic listen");
    }
}
