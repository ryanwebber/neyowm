use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use app::App;
use neyowm::{xplane, Server};
use xplaneconnect::XPlaneConnection;

mod app;
mod ui;

fn main() {
    let connection = XPlaneConnection::open(Ipv4Addr::LOCALHOST);
    let shared_connection = Arc::new(connection);
    let monitor = xplane::Monitor::new(Duration::from_millis(1000), shared_connection.clone());
    let telemetry = xplane::Telemetry::new(Duration::from_millis(66), shared_connection.clone());
    let autopilot = xplane::Autopilot::new(Duration::from_millis(100), shared_connection.clone());

    let mut server = Server::new();

    server.spawn_client("xplane:monitor", move |bridge| {
        monitor.run(bridge);
    });

    server.spawn_client("xplane:telemetry", move |bridge| {
        telemetry.run(bridge);
    });

    server.spawn_client("xplane:autopilot", move |bridge| {
        autopilot.run(bridge);
    });

    server.spawn_client("main", |bridge| {
        _ = App::new().run(bridge);
    });

    server.run();
}
