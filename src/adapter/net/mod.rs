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

pub fn run_sample_server() -> ! {
    let mut server_context =
        server::ServerContextBuilder::default().build(server::ServerContextBuilderParameters {
            address: SAMPLE_SERVER_ADDRESS,
        });

    println!("Server started.");

    loop {
        let _ = server_context.update(SAMPLE_DELTA);

        for id in server_context.get_client_ids() {
            while let Some(message) =
                server_context.receive_from(id, server::DefaultChannel::ReliableOrdered)
            {
                println!("Server Received: {:?}", message);
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

pub fn run_sample_client() -> ! {
    let mut client_context = client::UnsecureClientContextBuilder::default().build(
        client::UnsecureClientContextBuilderParameters {
            server_address: SAMPLE_SERVER_ADDRESS,
            client_address: SAMPLE_CLIENT_ADDRESS,
            client_id: 0,
        },
    );

    println!("Client started.");

    loop {
        let _ = client_context.update(SAMPLE_DELTA);

        if client_context.is_connected() {
            while let Some(message) =
                client_context.receive(server::DefaultChannel::ReliableOrdered)
            {
                println!("Client Received: {:?}", message);
                client_context.send(server::DefaultChannel::ReliableOrdered, "Hi from client.");
            }
        }

        thread::sleep(SAMPLE_DELTA);
    }
}

pub fn run_sample_net() -> ! {
    let server_thread = thread::spawn(run_sample_server);
    let _client_thread = thread::spawn(run_sample_client);

    server_thread.join().unwrap();
}
