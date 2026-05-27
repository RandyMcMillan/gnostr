uniffi::setup_scaffolding!();
 
#[uniffi::export]
fn rust_hello() -> String {
    "Hello from Rust!".to_string()
}

#[uniffi::export]
pub fn rust_add(a: u32, b: u32) -> u32 {
    a + b
}

#[uniffi::export]
pub fn p2p_network_start() -> String {
    gnostr_p2p::embedded_network::start()
}

#[uniffi::export]
pub fn p2p_network_status() -> String {
    gnostr_p2p::embedded_network::status()
}

#[uniffi::export]
pub fn p2p_network_stop() -> String {
    gnostr_p2p::embedded_network::stop()
}

#[uniffi::export]
pub fn p2p_network_logs() -> String {
    gnostr_p2p::embedded_network::logs()
}

#[uniffi::export]
pub fn p2p_network_peers() -> String {
    gnostr_p2p::embedded_network::peers()
}
