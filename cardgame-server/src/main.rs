use cardgame::messages::*;
use message_io::network::*;
use message_io::node;
use message_io::node::*;
use std::time::Duration;

pub fn main() {
    // Create a node, the main message-io entity. It is divided in 2 parts:
    // The 'handler', used to make actions (connect, send messages, signals, stop the node...)
    // The 'listener', used to read events from the network or signals.
    let (handler, listener) = node::split::<()>();

    // Listen for TCP, UDP and WebSocket messages at the same time.
    handler
        .network()
        .listen(Transport::FramedTcp, "0.0.0.0:3042")
        .unwrap();

    let mut clients: Vec<Endpoint> = vec![];

    let network_handle = std::thread::spawn(|| {
        // Read incoming network events.
        listener.for_each(move |event| match event.network() {
            NetEvent::Connected(endpoint, _) => {
                println!("{} Connected", endpoint);
                clients.push(endpoint);
            }
            NetEvent::Message(endpoint, data) => {
                let message: C2SMessage = bincode::deserialize(&data).unwrap();
                match message {
                    C2SMessage::Ping => {
                        println!("Ping from client")
                    }
                    C2SMessage::Ha => {
                        println!("Ha!")
                    }
                    _ => {
                        println!("Unknown message")
                    }
                }
            }
            NetEvent::Disconnected(endpoint) => {
                println!("{} Disconnected", endpoint);
                clients.remove(
                    clients
                        .iter()
                        .position(|x| *x == endpoint)
                        .expect("No such client found"),
                );
            }
        });
    });

    network_handle.join();
}

fn get_line(prompt: &str) -> String {
    println!("{}", prompt);
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Not a valid input");
    input
}
