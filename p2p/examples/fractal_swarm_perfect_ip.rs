use std::{fs, path::PathBuf};

use gnostr_p2p::perfect_ip::{generate_manifest, packetize, IntegrityManager, ProtocolSlice};
use sha2::{Digest, Sha256};

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    format!("{digest:x}")
}

fn reconstruct_payload(packets: &[ProtocolSlice]) -> Vec<u8> {
    let mut leaves: Vec<_> = packets.iter().filter(|packet| !packet.is_parity).collect();
    leaves.sort_by_key(|packet| packet.header.seq_num);
    leaves
        .into_iter()
        .flat_map(|packet| packet.data.clone())
        .collect()
}

fn input_file_path() -> Option<PathBuf> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(path) = arg.strip_prefix("--file=") {
            return Some(PathBuf::from(path));
        }
        if arg == "--file" {
            return args.next().map(PathBuf::from);
        }
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let example_path = input_file_path().unwrap_or_else(|| PathBuf::from("example.bin"));
    let original_bytes = if example_path.exists() {
        fs::read(&example_path)?
    } else {
        let bytes = b"perfect_ip demo payload\n".repeat(128);
        fs::write(&example_path, &bytes)?;
        bytes
    };

    let original_sha256 = sha256_hex(&original_bytes);
    println!("sender wrote {}", example_path.display());
    println!("sender sha256: {original_sha256}");

    let packet_batch = packetize("EXAMPLE".to_string(), original_bytes.clone());
    let manifest = generate_manifest("EXAMPLE".to_string(), original_bytes.len());
    let mut peer = IntegrityManager::new(manifest);

    println!("sender packetized into {} packets", packet_batch.total_packets);
    for slice in packet_batch.packets.clone() {
        peer.record_slice(slice);
    }

    println!("peer missing nodes: {:?}", peer.get_missing_nodes());
    println!("peer integrity verified: {}", peer.verify_integrity());

    let reconstructed = reconstruct_payload(&packet_batch.packets);
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    fs::write("example.reconstructed.bin", &reconstructed)?;

    println!("peer reconstructed example.reconstructed.bin");
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    Ok(())
}
