use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use xplaneconnect::XPlaneConnection;

use crate::{
    server::Bridge,
    types::{ClientBoundMessage, ServerBoundMessage},
};

pub struct Monitor {
    connection: Arc<Mutex<XPlaneConnection>>,
}

impl Monitor {
    pub fn new(connection: Arc<Mutex<XPlaneConnection>>) -> Self {
        Self { connection }
    }

    pub fn run(self, bridge: Bridge, interval: Duration) {
        bridge.recv_with_interval(interval, |_, tx| {
            if let Ok(connection) = self.connection.lock() {
                match connection.read_position() {
                    Ok(_) => {
                        _ = tx.send(ServerBoundMessage::Broadcast(
                            ClientBoundMessage::UpdateConnectionStatus(
                                crate::ConnectionStatus::Connected,
                            ),
                        ));
                    }
                    Err(_) => {
                        _ = tx.send(ServerBoundMessage::Broadcast(
                            ClientBoundMessage::UpdateConnectionStatus(
                                crate::ConnectionStatus::Disconnected,
                            ),
                        ));
                    }
                }
            }
        });
    }
}
