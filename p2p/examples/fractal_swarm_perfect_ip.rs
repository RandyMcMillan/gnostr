use std::{fs, path::PathBuf};

use futures::StreamExt;
use gnostr_p2p::keypair_from_seed;
use gnostr_p2p::perfect_ip::{
    build_fractal_swarm, calculate_parity, generate_manifest, packetize, summarize_packets,
    FractalBehaviourEvent, IntegrityManager, ProtocolSlice, MTU_PAYLOAD,
};
use libp2p::{request_response, swarm::SwarmEvent};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct DemoArgs {
    file: Option<PathBuf>,
    recursive: bool,
    depth: usize,
    help: bool,
}

fn usage() {
    println!(
        "Usage: fractal_swarm_perfect_ip [--file PATH] [--recursive] [--depth N] [--help]\n\
         \n\
         Options:\n\
           --file PATH       Read input from PATH instead of generating example.bin\n\
           --recursive       Print a recursive packet tree before the flat packet summary\n\
           --depth N         Limit recursive tree printing to N levels [default: 3]\n\
           --help            Show this help message\n"
    );
}

fn parse_args() -> DemoArgs {
    let mut args = std::env::args().skip(1);
    let mut file = None;
    let mut recursive = false;
    let mut depth = 3usize;
    let mut help = false;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            help = true;
        } else if arg == "--recursive" {
            recursive = true;
        } else if let Some(path) = arg.strip_prefix("--file=") {
            file = Some(PathBuf::from(path));
        } else if arg == "--file" {
            file = args.next().map(PathBuf::from);
        } else if let Some(value) = arg.strip_prefix("--depth=") {
            depth = value.parse().unwrap_or(3);
        } else if arg == "--depth" {
            depth = args.next().and_then(|value| value.parse().ok()).unwrap_or(3);
        } else {
            eprintln!("unrecognized argument: {arg}");
            help = true;
        }
    }

    DemoArgs {
        file,
        recursive,
        depth,
        help,
    }
}

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

fn packet_tree_lines(id: &str, data: &[u8], depth: usize, level: usize, lines: &mut Vec<String>) {
    let indent = "  ".repeat(level);
    lines.push(format!("{indent}{id} ({}B)", data.len()));

    if depth == 0 || data.len() <= MTU_PAYLOAD / 2 {
        return;
    }

    let half = data.len() / 2;
    let left = &data[..half];
    let right = &data[half..];
    let parity = calculate_parity(left, right);

    packet_tree_lines(&format!("{id}.0"), left, depth - 1, level + 1, lines);
    packet_tree_lines(&format!("{id}.1"), right, depth - 1, level + 1, lines);
    lines.push(format!("{indent}{id}.P ({}B parity)", parity.len()));
}

fn recursive_tree_lines(id: &str, data: &[u8], depth: usize) -> Vec<String> {
    let mut lines = Vec::new();
    packet_tree_lines(id, data, depth, 0, &mut lines);
    lines
}

fn input_file_path(file: Option<PathBuf>) -> PathBuf {
    file.unwrap_or_else(|| PathBuf::from("example.bin"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args = parse_args();
    if args.help {
        usage();
        return Ok(());
    }

    let example_path = input_file_path(args.file);
    let original_bytes = if example_path.exists() {
        fs::read(&example_path)?
    } else {
        let bytes = b"perfect_ip demo payload\n".repeat(128);
        fs::write(&example_path, &bytes)?;
        bytes
    };

    let original_sha256 = sha256_hex(&original_bytes);
    let batch = packetize("ROOT".to_string(), original_bytes.clone());
    let manifest = generate_manifest("ROOT".to_string(), original_bytes.len());

    println!("sender wrote {}", example_path.display());
    println!("sender sha256: {original_sha256}");

    if args.recursive {
        for line in recursive_tree_lines("ROOT", &original_bytes, args.depth) {
            println!("{line}");
        }
    }

    println!("perfect_ip packet batch: {} packets", batch.total_packets);
    for line in summarize_packets(&batch.packets) {
        println!("{line}");
    }

    let mut integrity = IntegrityManager::new(manifest);
    for slice in batch.packets.clone() {
        integrity.record_slice(slice);
    }
    println!("missing nodes: {:?}", integrity.get_missing_nodes());
    println!("integrity verified: {}", integrity.verify_integrity());

    let reconstructed = reconstruct_payload(&batch.packets);
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    fs::write("example.reconstructed.bin", &reconstructed)?;

    println!("peer reconstructed example.reconstructed.bin");
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    let local_key = keypair_from_seed(None);
    let mut swarm = build_fractal_swarm(local_key).await?;
    let listen_address = "/ip4/127.0.0.1/udp/0/quic-v1".parse()?;
    let listener_id = swarm.listen_on(listen_address)?;
    println!("repair swarm listener: {listener_id:?}");
    println!("press Ctrl-C to stop the demo");

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                println!("stopping demo");
                break;
            }
            event = swarm.select_next_some() => {
                if let SwarmEvent::Behaviour(FractalBehaviourEvent::RepairRpc(event)) = event {
                    if let request_response::Event::Message { message, .. } = event {
                        if let request_response::Message::Request { request, channel, .. } = message {
                            let response = integrity
                                .received_slices
                                .get(&request.id)
                                .cloned()
                                .unwrap_or_else(|| ProtocolSlice {
                                    id: request.id.clone(),
                                    header: request.header.clone(),
                                    data: Vec::new(),
                                    is_parity: false,
                                });
                            let _ = swarm.behaviour_mut().repair_rpc.send_response(channel, response);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
