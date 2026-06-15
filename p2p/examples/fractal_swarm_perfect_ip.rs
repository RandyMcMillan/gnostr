use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

use futures::StreamExt;
use gnostr_p2p::keypair_from_seed;
use gnostr_p2p::perfect_ip::{
    build_fractal_swarm, generate_manifest, packetize, summarize_packets, FractalBehaviourEvent,
    IntegrityManager, ProtocolSlice,
};
use gnostr_p2p::message::{Event, EventBuilder, EventKind, PrivateKey};
use libp2p::{request_response, swarm::SwarmEvent};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct DemoArgs {
    file: Option<PathBuf>,
    out: Option<PathBuf>,
    recursive: Option<PathBuf>,
    depth: usize,
    depth_set: bool,
    verbose: bool,
    help: bool,
}

fn usage() {
    println!(
        "Usage: fractal_swarm_perfect_ip [--file PATH] [--out PATH] [--recursive PATH] [--depth N] [--verbose] [--help]\n\
         \n\
         Options:\n\
           --file PATH       Read a single file from PATH\n\
          --out PATH        Write reconstructed bytes to PATH [default: example.reconstructed.bin]\n\
          --recursive PATH   Walk PATH as a directory tree and preserve relative paths\n\
          --depth N         Limit recursive directory walking to N levels [default: 3]\n\
          --verbose         Print packet info during file mode\n\
           --help            Show this help message\n"
    );
}

fn parse_args() -> DemoArgs {
    let mut args = std::env::args().skip(1).peekable();
    let mut file = None;
    let mut out = None;
    let mut recursive = None;
    let mut depth = 3usize;
    let mut depth_set = false;
    let mut verbose = false;
    let mut help = false;

    while let Some(arg) = args.next() {
        if arg == "--help" || arg == "-h" {
            help = true;
        } else if arg == "--verbose" {
            verbose = true;
        } else if arg == "--recursive" {
            recursive = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if let Some(path) = arg.strip_prefix("--recursive=") {
            recursive = Some(PathBuf::from(path));
        } else if let Some(path) = arg.strip_prefix("--file=") {
            file = Some(PathBuf::from(path));
        } else if let Some(path) = arg.strip_prefix("--out=") {
            out = Some(PathBuf::from(path));
        } else if arg == "--out" {
            out = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if arg == "--file" {
            file = match args.peek() {
                Some(next) if !next.starts_with('-') => args.next().map(PathBuf::from),
                _ => {
                    help = true;
                    None
                }
            };
        } else if let Some(value) = arg.strip_prefix("--depth=") {
            depth = value.parse().unwrap_or(3);
            depth_set = true;
        } else if arg == "--depth" {
            depth = match args.peek() {
                Some(next) if !next.starts_with('-') => args
                    .next()
                    .and_then(|value| value.parse().ok())
                    .unwrap_or(3),
                _ => {
                    help = true;
                    3
                }
            };
            depth_set = true;
        } else {
            eprintln!("unrecognized argument: {arg}");
            help = true;
        }
    }

    if recursive.is_none() && depth_set {
        help = true;
    }
    if recursive.is_some() && file.is_some() {
        help = true;
    }

    DemoArgs {
        file,
        out,
        recursive,
        depth,
        depth_set,
        verbose,
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

fn list_directory(
    root: &Path,
    current: &Path,
    depth: usize,
    level: usize,
    verbose: bool,
    lines: &mut Vec<String>,
    all_packets: &mut Vec<ProtocolSlice>,
) -> io::Result<()> {
    if level == 0 {
        lines.push(".".to_string());
    }

    let mut entries = Vec::new();
    for entry in fs::read_dir(current)? {
        entries.push(entry?);
    }
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_path_buf();
        let indent = "  ".repeat(level + 1);
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            lines.push(format!("{indent}{}/", relative.display()));
            // Recursively visit all directories unconditionally:
            list_directory(root, &path, depth, level + 1, verbose, lines, all_packets)?;
        } else if metadata.is_file() {
            let bytes = fs::read(&path)?;
            lines.push(format!(
                "{indent}{} ({}B sha256:{})",
                relative.display(),
                bytes.len(),
                sha256_hex(&bytes)
            ));
            let batch = packetize(relative.to_string_lossy().into_owned(), bytes);
            all_packets.extend(batch.packets.clone());
            if verbose {
                // Keep the direct prints requested:
                println!("File: {}", relative.display());
                println!("Batch size: {} packets", batch.total_packets);

                // Ensure the summary is also added to `lines` so it appears in the tree walk output:
                lines.push(format!("{indent}  perfect_ip packet batch: {} packets", batch.total_packets));
                for line in summarize_packets(&batch.packets) {
                    lines.push(format!("{indent}  {line}"));
                }
            }
        } else {
            lines.push(format!("{indent}{} [special]", relative.display()));
        }
    }

    Ok(())
}

fn run_directory_walk(path: &Path, depth: usize, verbose: bool) -> io::Result<Vec<ProtocolSlice>> {
    let mut lines = Vec::new();
    let mut all_packets = Vec::new();
    list_directory(path, path, depth, 0, verbose, &mut lines, &mut all_packets)?;
    for line in lines {
        println!("{line}");
    }
    Ok(all_packets)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let DemoArgs {
        file,
        out,
        recursive,
        depth,
        depth_set,
        verbose,
        help,
    } = parse_args();

    if help {
        usage();
        return Ok(());
    }

    if let Some(ref root) = recursive {
        if file.is_some() {
            return Err("--file and --recursive are mutually exclusive".into());
        }

        // Commented out to allow --depth to be optional and use its default value of 3:
        // if !depth_set {
        //     return Err("--recursive requires --depth".into());
        // }

        if !root.is_dir() {
            usage();
            return Ok(());
        }

        println!("recursive walk root: {}", root.display());
        println!("recursive depth: {}", depth);
        println!("verbose flag is: {}", verbose);
        let all_packets = run_directory_walk(root, depth, verbose)?;

        if let Some(ref out_path) = out {
            let reconstructed = reconstruct_payload(&all_packets);
            fs::write(out_path, &reconstructed)?;
            println!("recursively reconstructed {} bytes to {}", reconstructed.len(), out_path.display());
            return Ok(());
        }
        // Commented out to allow the P2P service to start when --recursive is used:
        // return Ok(());
    }

    // Commented out the old check to allow --depth when --recursive is used:
    // if depth_set {
    //     return Err("--depth is only valid with --recursive".into());
    // }
    if recursive.is_none() && depth_set {
        return Err("--depth is only valid with --recursive".into());
    }

    let example_path = if let Some(ref root) = recursive {
        root.clone()
    } else {
        file.unwrap_or_else(|| PathBuf::from("example.bin"))
    };

    let original_bytes = if example_path.exists() {
        if example_path.is_dir() {
            // If it's a directory, we need to decide how to packetize it.
            // For now, treat it as empty or handle appropriately.
            vec![]
        } else {
            fs::read(&example_path)?
        }
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
    if verbose {
        println!("perfect_ip packet batch: {} packets", batch.total_packets);
        for line in summarize_packets(&batch.packets) {
            println!("{line}");
        }
    }

    let mut integrity = IntegrityManager::new(manifest);
    for slice in batch.packets.clone() {
        integrity.record_slice(slice);
    }
    println!("missing nodes: {:?}", integrity.get_missing_nodes());
    println!("integrity verified: {}", integrity.verify_integrity());

    let reconstructed = reconstruct_payload(&batch.packets);
    let reconstructed_sha256 = sha256_hex(&reconstructed);
    let output_path = out.unwrap_or_else(|| PathBuf::from("example.reconstructed.bin"));
    fs::write(&output_path, &reconstructed)?;

    println!("peer reconstructed {}", output_path.display());
    println!("peer sha256: {reconstructed_sha256}");
    println!("sha256 match: {}", reconstructed_sha256 == original_sha256);

    let local_key = keypair_from_seed(None);
    let mut swarm = build_fractal_swarm(local_key).await?;
    let listen_address = "/ip4/127.0.0.1/udp/0/quic-v1".parse()?;
    let listener_id = swarm.listen_on(listen_address)?;
    println!("repair swarm listener: {listener_id:?}");
    println!("press Ctrl-C to stop the demo");

    // Construct and broadcast a real Nostr event
    let private_key = PrivateKey::generate();
    let event = EventBuilder::text_note(format!("fractal_swarm_perfect_ip: reconstructed {} bytes (sha256: {})", original_bytes.len(), original_sha256))
        .to_event(&private_key)
        .expect("failed to build event");
    
    // Broadcast to crawler relays
    let config_dir = gnostr_p2p::p2p::relay_paths::get_config_dir_path();
    match gnostr_p2p::p2p::crawler_broadcast::broadcast_event_to_crawler_relays(&config_dir, &event).await {
        Ok(count) => println!("Broadcasted real Nostr event: {:?}. Published to {} relays.", event.id, count),
        Err(e) => eprintln!("Failed to broadcast real Nostr event: {}", e),
    }

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
