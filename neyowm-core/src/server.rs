use crossbeam_channel::RecvTimeoutError;

use crate::types::{ClientBoundMessage, ServerBoundMessage};

pub struct Server {
    clients: Vec<ClientHandle>,
    rx: crossbeam_channel::Receiver<ServerBoundMessage>,
    tx: crossbeam_channel::Sender<ServerBoundMessage>,
}

impl Server {
    pub fn new() -> Self {
        let (tx, rx) = crossbeam_channel::unbounded();
        Server {
            clients: Vec::new(),
            rx,
            tx,
        }
    }

    pub fn spawn_client(&mut self, id: &'static str, f: impl FnOnce(Bridge) + Send + 'static) {
        let (client_tx, client_rx) = crossbeam_channel::unbounded();
        let server_tx = self.tx.clone();

        let join_handle = std::thread::spawn(move || {
            let bridge = Bridge {
                rx: client_rx,
                tx: server_tx,
            };

            f(bridge);
        });

        self.clients.push(ClientHandle {
            id,
            join_handle,
            tx: client_tx,
        });
    }

    pub fn run(self) {
        println!("Server is running...");

        if self.clients.len() > 0 {
            loop {
                match self.rx.recv() {
                    Ok(ServerBoundMessage::Broadcast(message)) => {
                        self.post_to_all_clients(message);
                    }
                    Ok(ServerBoundMessage::Shutdown) => {
                        println!("Server is shutting down...");
                        break;
                    }
                    Err(e) => {
                        println!("Server channel is closed: {}", e);
                        break;
                    }
                }
            }
        }

        println!("Server is shutting down...");

        self.post_to_all_clients(ClientBoundMessage::Shutdown);
        for client in self.clients {
            client
                .join_handle
                .join()
                .expect(&format!("Client {} panicked", client.id));
        }
    }

    fn post_to_all_clients(&self, message: ClientBoundMessage) {
        for client in &self.clients {
            _ = client.tx.send(message.clone());
        }
    }
}

pub struct Bridge {
    rx: crossbeam_channel::Receiver<ClientBoundMessage>,
    tx: crossbeam_channel::Sender<ServerBoundMessage>,
}

impl Bridge {
    pub fn into_inner(
        self,
    ) -> (
        crossbeam_channel::Receiver<ClientBoundMessage>,
        crossbeam_channel::Sender<ServerBoundMessage>,
    ) {
        (self.rx, self.tx)
    }

    pub fn send(&self, message: ServerBoundMessage) {
        self.tx.send(message).expect("Server channel is closed");
    }

    pub fn broadcast(&self, message: ClientBoundMessage) {
        self.send(ServerBoundMessage::Broadcast(message));
    }

    pub fn recv(&self) -> ClientBoundMessage {
        self.rx.recv().expect("Client channel is closed")
    }

    pub fn recv_with_interval(
        self,
        interval: std::time::Duration,
        mut f: impl FnMut(&[ClientBoundMessage], &crossbeam_channel::Sender<ServerBoundMessage>),
    ) {
        let (rx, tx) = self.into_inner();
        let mut queue = Vec::new();
        let mut next_invocation = std::time::Instant::now() + interval;
        loop {
            match rx.recv_timeout(interval) {
                Ok(ClientBoundMessage::Shutdown) => {
                    break;
                }
                Ok(message) => {
                    queue.push(message);
                }
                Err(RecvTimeoutError::Disconnected) => {
                    break;
                }
                Err(RecvTimeoutError::Timeout) => {}
            }

            if std::time::Instant::now() >= next_invocation {
                f(&queue, &tx);
                queue.clear();
                next_invocation = std::time::Instant::now() + interval;
            }
        }
    }
}

pub struct ClientHandle {
    id: &'static str,
    join_handle: std::thread::JoinHandle<()>,
    tx: crossbeam_channel::Sender<ClientBoundMessage>,
}

pub trait Client {
    const ID: &'static str;

    fn run(self, bridge: Bridge);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lifecycle() {
        let mut server = Server::new();

        server.spawn_client("c1", move |bridge| loop {
            match bridge.recv() {
                ClientBoundMessage::Shutdown => break,
                _ => {}
            }
        });

        server.spawn_client("c2", move |bridge| {
            bridge.send(ServerBoundMessage::Shutdown);
            loop {
                match bridge.recv() {
                    ClientBoundMessage::Shutdown => break,
                    _ => {}
                }
            }
        });

        server.run();
    }
}
