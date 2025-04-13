#![doc = include_str!("../README.md")]

//mod network;
use packfile::network;
use std::{error::Error, io::Write, path::PathBuf};
use std::path::Path;
//use async_std::path::Path;

use clap::Parser;
use futures::{prelude::*, StreamExt};
use libp2p::{core::Multiaddr, multiaddr::Protocol};
use libp2p::swarm::{NetworkBehaviour, Swarm, SwarmEvent};

use tokio::task::spawn;
use tokio::io;
use tokio::fs;
use tracing_subscriber::EnvFilter;

#[derive(NetworkBehaviour)]
struct MyBehaviour {
    floodsub: libp2p::floodsub::Floodsub,
}

async fn send_packfile_to_peers(
    swarm: &mut Swarm<MyBehaviour>,
    packfile_path: &Path,
) -> io::Result<()> {
    let file_data = fs::read(packfile_path).await?;
    let topic = libp2p::floodsub::Topic::new("packfile_transfer");

    swarm.behaviour_mut().floodsub.publish(topic, file_data);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();

    let opt = Opt::parse();


    let packfile_path = Path::new("repo.pack");

    // Create a dummy packfile for testing
    fs::write(packfile_path, b"This is a dummy packfile.").await?;


    let (mut network_client, mut network_events, network_event_loop) =
        network::new(opt.secret_key_seed).await?;

    // Spawn the network task for it to run in the background.
    spawn(network_event_loop.run());

    // In case a listen address was provided use it, otherwise listen on any
    // address.
    match opt.listen_address {
        Some(addr) => network_client
            .start_listening(addr)
            .await
            .expect("Listening not to fail."),
        None => network_client
            .start_listening("/ip4/0.0.0.0/tcp/0".parse()?)
            .await
            .expect("Listening not to fail."),
    };

    // In case the user provided an address of a peer on the CLI, dial it.
    if let Some(addr) = opt.peer {
        let Some(Protocol::P2p(peer_id)) = addr.iter().last() else {
            return Err("Expect peer multiaddr to contain peer ID.".into());
        };
        network_client
            .dial(peer_id, addr)
            .await
            .expect("Dial to succeed");
    }

    match opt.argument {
        // Providing a file.
        CliArgument::Provide { path, name } => {
            // Advertise oneself as a provider of the file on the DHT.
            network_client.start_providing(name.clone()).await;

            loop {
                match network_events.next().await {
                    // Reply with the content of the file on incoming requests.
                    Some(network::Event::InboundRequest { request, channel }) => {
                        if request == name {
                            network_client
                                .respond_file(std::fs::read(&path)?, channel)
                                .await;
                        }
                    }
                    e => todo!("{:?}", e),
                }
            }
        }
        // Locating and getting a file.
        CliArgument::Get { name } => {
            // Locate all nodes providing the file.
            let providers = network_client.get_providers(name.clone()).await;
            if providers.is_empty() {
                return Err(format!("Could not find provider for file {name}.").into());
            }

            // Request the content of the file from each node.
            let requests = providers.into_iter().map(|p| {
                let mut network_client = network_client.clone();
                let name = name.clone();
                async move { network_client.request_file(p, name).await }.boxed()
            });

            // Await the requests, ignore the remaining once a single one succeeds.
            let file_content = futures::future::select_ok(requests)
                .await
                .map_err(|_| "None of the providers returned file.")?
                .0;

            std::io::stdout().write_all(&file_content)?;
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
#[command(name = "libp2p file sharing example")]
struct Opt {
    /// Fixed value to generate deterministic peer ID.
    #[arg(long, short, default_value = "0")]
    secret_key_seed: Option<u8>,

    #[arg(long)]
    peer: Option<Multiaddr>,

    #[arg(long, short)]
    listen_address: Option<Multiaddr>,

    #[command(subcommand)]
    argument: CliArgument,
}

#[derive(Debug, Parser)]
enum CliArgument {
    Provide {
        #[arg(long, short)]
        path: PathBuf,
        #[arg(long, short)]
        name: String,
    },
    Get {
        #[arg(long, short)]
        name: String,
    },
}
