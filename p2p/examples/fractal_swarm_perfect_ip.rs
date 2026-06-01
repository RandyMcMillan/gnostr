use futures::StreamExt;
use gnostr_p2p::keypair_from_seed;
use gnostr_p2p::perfect_ip::{
    build_fractal_swarm, generate_manifest, packetize, summarize_packets, FractalBehaviourEvent,
    IntegrityManager, ProtocolSlice,
};
use libp2p::{request_response, swarm::SwarmEvent};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let payload = vec![0xAB; 3000];
    let batch = packetize("ROOT".to_string(), payload.clone());
    let manifest = generate_manifest("ROOT".to_string(), payload.len());

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

    let local_key = keypair_from_seed(Some(gnostr_asyncgit::default_gnostr_private_key_hex()));
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
