pub mod client;
pub mod server;

use std::net;
use std::thread;
use std::time;

const SAMPLE_SERVER_ADDR: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 5000);
const SAMPLE_CLIENT_ADDR: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 0);

pub fn run_sample_net() {
    let mut server_context =
        server::ServerContext::new(&SAMPLE_SERVER_ADDR, server::ConnectionConfig::default());
    let mut client_context = client::ClientContext::new(
        &SAMPLE_CLIENT_ADDR,
        &SAMPLE_SERVER_ADDR,
        server::ConnectionConfig::default(),
    );

    const DELTA: time::Duration = time::Duration::from_millis(16);

    let _client_thread = thread::spawn(move || loop {
        let _ = client_context.update(DELTA);
        thread::sleep(DELTA);
    });

    loop {
        let _ = server_context.update(DELTA);
        thread::sleep(DELTA);
    }
}
