pub mod client;
pub mod server;

use std::net;
use std::thread;
use std::time;

const SAMPLE_CLIENT_ADDRESS: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 0);
const SAMPLE_SERVER_ADDRESS: net::SocketAddr =
    net::SocketAddr::new(net::IpAddr::V4(net::Ipv4Addr::new(127, 0, 0, 1)), 5000);
const SAMPLE_DELTA: time::Duration = time::Duration::from_millis(16);

pub fn start_sample_server() -> ! {
    let mut server_context =
        server::ServerContextBuilder::default().build(&server::ServerContextBuilderParameters {
            address: SAMPLE_SERVER_ADDRESS,
        });

    println!("Server started.");

    loop {
        let _ = server_context.update(SAMPLE_DELTA);

        for id in server_context.get_client_ids() {
            while let Some(message) =
                server_context.receive_from(id, server::DefaultChannel::ReliableOrdered)
            {
                println!("Server Received: {:?}", message)
            }
        }

        while let Some(event) = server_context.get_event() {
            if let server::ServerEvent::ClientConnected { client_id } = event {
                println!("Client joined: {:?}", client_id);
            }
        }

        server_context.send_all(
            server::DefaultChannel::ReliableOrdered,
            "Hello from server.",
        );

        thread::sleep(SAMPLE_DELTA);
    }
}

pub fn start_sample_client() -> ! {
    let mut client_context = client::ClientContext::new(
        &SAMPLE_CLIENT_ADDRESS,
        &SAMPLE_SERVER_ADDRESS,
        server::ConnectionConfig::default(),
    );

    println!("Client started.");

    loop {
        let _ = client_context.update(SAMPLE_DELTA);
        thread::sleep(SAMPLE_DELTA);
    }
}

pub fn start_sample_net() -> ! {
    let server_thread = thread::spawn(start_sample_server);
    let _client_thread = thread::spawn(start_sample_client);

    server_thread.join().unwrap();
}
