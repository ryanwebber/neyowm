#[derive(Debug, Clone)]
pub enum ClientBoundMessage {
    Shutdown,
    SetAutopilotMode(AutopilotMode),
    UpdateConnectionStatus(ConnectionStatus),
    UpdateTelemetry(TelemetryUpdate),
}

#[derive(Debug, Clone)]
pub enum ServerBoundMessage {
    Shutdown,
    Broadcast(ClientBoundMessage),
}

#[derive(Debug, Clone)]
pub enum TelemetryUpdate {
    Orientation {
        pitch: f64,
        roll: f64,
        yaw: f64,
    },
    Position {
        latitude: f64,
        longitude: f64,
        altitude: f64,
    },
    Control {
        aileron: f64,
        elevator: f64,
        rudder: f64,
        throttle: f64,
        flaps: f64,
        speedbrake: f64,
    },
    Terrain {
        latitude: f64,
        longitude: f64,
        elevation: f64,
        normal: (f64, f64, f64),
        velocity: (f64, f64, f64),
        wet: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Disconnected,
}

#[derive(Debug, Clone)]
pub enum AutopilotMode {
    Off,
    Hold { roll: f64, pitch: f64 },
}
