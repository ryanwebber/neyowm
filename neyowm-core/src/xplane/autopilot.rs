use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use pid::Pid;
use xplaneconnect::{SetControlSurface, XPlaneConnection};

use crate::{server::Bridge, AutopilotMode, ClientBoundMessage, TelemetryUpdate};

pub struct Autopilot {
    interval: Duration,
    connection: Arc<Mutex<XPlaneConnection>>,
}

impl Autopilot {
    pub fn new(interval: Duration, connection: Arc<Mutex<XPlaneConnection>>) -> Self {
        Self {
            interval,
            connection,
        }
    }

    pub fn run(self, bridge: Bridge) {
        let mut state = State {
            mode: AutopilotMode::Off,
            roll: PidState::new(Pid::new(0.0, 5.0).p(0.05, 1.0).i(0.01, 1.0).to_owned()),
            pitch: PidState::new(Pid::new(2.0, 5.0).p(0.05, 1.0).i(0.01, 1.0).to_owned()),
        };

        bridge.recv_with_interval(self.interval, |queue, _| {
            for msg in queue.iter().cloned() {
                match msg {
                    ClientBoundMessage::Shutdown => break,
                    ClientBoundMessage::SetAutopilotMode(mode) => {
                        state.mode = mode;
                        state.roll.pid.reset_integral_term();
                        state.pitch.pid.reset_integral_term();
                    }
                    ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Orientation {
                        roll,
                        pitch,
                        ..
                    }) => {
                        state.roll.update(roll);
                        state.pitch.update(pitch);
                    }
                    _ => {}
                }
            }

            match state.mode {
                AutopilotMode::Off => {}
                AutopilotMode::Hold { roll, pitch } => {
                    state.roll.pid.setpoint(roll);
                    state.pitch.pid.setpoint(pitch);

                    let Ok(connection) = self.connection.lock() else {
                        return;
                    };

                    let controls = SetControlSurface {
                        aileron: state.roll.finite_value(),
                        elevator: state.pitch.finite_value(),
                        ..Default::default()
                    };

                    _ = connection.write_controls(controls);
                }
            }
        });
    }
}

pub struct State {
    mode: AutopilotMode,
    roll: PidState,
    pitch: PidState,
}

pub struct PidState {
    pid: Pid<f64>,
    value: Option<f64>,
}

impl PidState {
    pub fn new(pid: Pid<f64>) -> Self {
        Self { pid, value: None }
    }

    pub fn finite_value(&self) -> Option<f64> {
        self.value
            .and_then(|value| if value.is_finite() { Some(value) } else { None })
    }

    pub fn update(&mut self, measurement: f64) {
        let state = self.pid.next_control_output(measurement);
        self.value = Some(state.output);
    }
}
