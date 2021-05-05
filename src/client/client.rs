use crate::messages::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::time::Duration;

pub fn run_client() {
    let transport = Transport::FramedTcp;
    let remote_addr = "127.0.0.1:3042".to_remote_addr().unwrap();
    let (handler, listener) = node::split();

    let server_id = match handler.network().connect(transport, remote_addr.clone()) {
        Ok((server_id, local_addr)) => {
            println!(
                "Connected to server by {} at {}",
                transport,
                server_id.addr()
            );
            println!("Client identified by local port: {}", local_addr.port());
            server_id
        }
        Err(_) => {
            return println!(
                "Can not connect to the server by {} to {}",
                transport, remote_addr
            )
        }
    };

    handler.signals().send(crate::Signal::Greet);

    listener.for_each(move |event| match event {
        NodeEvent::Signal(signal) => match signal {
            crate::Signal::Greet => {
                let message = C2SMessage::Ping;
                let output_data = bincode::serialize(&message).unwrap();
                handler.network().send(server_id, &output_data);
                handler
                    .signals()
                    .send_with_timer(crate::Signal::Greet, Duration::from_secs(1));
            }
        },
        NodeEvent::Network(net_event) => match net_event {
            NetEvent::Message(_, input_data) => {
                let message: S2CMessage = bincode::deserialize(&input_data).unwrap();
                match message {
                    S2CMessage::Pong => {
                        println!("Pong from server")
                    }
                    S2CMessage::Pu => {
                        println!("Pu!")
                    }
                    _ => {
                        println!("Unknown message")
                    }
                }
            }
            NetEvent::Connected(_, _) => unreachable!(), // Only generated when a listener accepts
            NetEvent::Disconnected(_) => {
                println!("Server is disconnected");
                handler.stop();
            }
        },
    });
}
