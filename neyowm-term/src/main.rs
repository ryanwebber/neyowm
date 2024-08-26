use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use app::App;
use neyowm::{xplane, ClientBoundMessage, Server};
use xplaneconnect::XPlaneConnection;

mod app;
mod ui;

fn main() {
    let connection = XPlaneConnection::open(Ipv4Addr::LOCALHOST);
    let shared_connection = Arc::new(connection);
    let xplane_monitor = xplane::Monitor::new(shared_connection.clone());
    let xplane_telemetry = xplane::Telemetry::new(shared_connection.clone());
    let xplane_autopilot = xplane::Autopilot::new(shared_connection.clone());

    let mut server = Server::new();

    server.spawn_client("xplane:monitor", move |bridge| {
        xplane_monitor.run(bridge, Duration::from_millis(1000));
    });

    server.spawn_client("xplane:telemetry", move |bridge| {
        xplane_telemetry.run(bridge, Duration::from_millis(66));
    });

    server.spawn_client("xplane:autopilot", move |bridge| {
        xplane_autopilot.run(bridge, Duration::from_millis(100));
    });

    server.spawn_client("app", |bridge| {
        if std::env::args().any(|arg| arg == "--non-interactive") {
            loop {
                match bridge.recv() {
                    ClientBoundMessage::Shutdown => {
                        break;
                    }
                    message => {
                        println!("{:?}", message);
                    }
                }
            }
        } else {
            _ = App::new().run(bridge);
        }
    });

    server.run();
}
