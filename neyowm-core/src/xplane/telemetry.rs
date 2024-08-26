use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use xplaneconnect::XPlaneConnection;

use crate::{
    server::Bridge,
    types::{ClientBoundMessage, ServerBoundMessage, TelemetryUpdate},
};

pub struct Telemetry {
    interval: Duration,
    connection: Arc<Mutex<XPlaneConnection>>,
}

impl Telemetry {
    pub fn new(interval: Duration, connection: Arc<Mutex<XPlaneConnection>>) -> Self {
        Self {
            interval,
            connection,
        }
    }

    pub fn run(self, bridge: Bridge) {
        let mut ticker = 0;
        bridge.recv_with_interval(self.interval, |_, tx| {
            match ticker {
                0 => {
                    if let Ok(connection) = self.connection.try_lock() {
                        if let Ok(data) = connection.read_position() {
                            _ = tx.send(ServerBoundMessage::Broadcast(
                                ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Orientation {
                                    pitch: data.pitch,
                                    roll: data.roll,
                                    yaw: data.yaw,
                                }),
                            ));

                            _ = tx.send(ServerBoundMessage::Broadcast(
                                ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Position {
                                    latitude: data.latitude,
                                    longitude: data.longitude,
                                    altitude: data.altitude,
                                }),
                            ));
                        }
                    }
                }
                1 => {
                    if let Ok(connection) = self.connection.try_lock() {
                        if let Ok(data) = connection.read_controls() {
                            _ = tx.send(ServerBoundMessage::Broadcast(
                                ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Control {
                                    aileron: data.aileron,
                                    elevator: data.elevator,
                                    rudder: data.rudder,
                                    throttle: data.throttle,
                                    flaps: data.flaps,
                                    speedbrake: data.speedbrake,
                                }),
                            ));
                        }
                    }
                }
                2 => {
                    if let Ok(connection) = self.connection.try_lock() {
                        if let Ok(data) = connection.read_terrain() {
                            _ = tx.send(ServerBoundMessage::Broadcast(
                                ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Terrain {
                                    latitude: data.latitude,
                                    longitude: data.longitude,
                                    elevation: data.elevation,
                                    normal: data.normal,
                                    velocity: data.velocity,
                                    wet: data.wet,
                                }),
                            ));
                        }
                    }
                }
                _ => {}
            }

            ticker = (ticker + 1usize) % 3;
        });
    }
}
