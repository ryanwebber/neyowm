use neyowm::{ClientBoundMessage, TelemetryUpdate};
use ratatui::{
    layout::Rect,
    style::Color,
    widgets::canvas::{Canvas, Line, Rectangle},
    Frame,
};

use crate::app::BridgeSink;

pub struct FlightVectorView {
    pub tilt: f64,
    pub attitude: f64,
}

impl FlightVectorView {
    pub fn new() -> Self {
        FlightVectorView {
            tilt: 0.0,
            attitude: 0.0,
        }
    }

    pub fn handle_client_message(&mut self, _: &BridgeSink, msg: &ClientBoundMessage) {
        match msg {
            ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Orientation {
                pitch,
                roll,
                ..
            }) => {
                self.tilt = *roll;
                self.attitude = *pitch;
            }
            _ => {}
        }
    }

    pub fn draw(&self, rect: Rect, frame: &mut Frame) {
        const CROSSHAIR_SIZE: f64 = 0.05;
        let center = (
            f64::clamp(self.tilt / 45.0, -1.0, 1.0),
            f64::clamp(self.attitude / 45.0, -1.0, 1.0),
        );

        let center_box = Rectangle {
            x: -0.25,
            y: -0.25,
            width: 0.5,
            height: 0.5,
            color: Color::White,
        };

        let canvas = Canvas::default()
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                ctx.draw(&Line {
                    x1: -1.0,
                    y1: 0.0,
                    x2: 1.0,
                    y2: 0.0,
                    color: Color::DarkGray,
                });

                ctx.draw(&Line {
                    x1: 0.0,
                    y1: -1.0,
                    x2: 0.0,
                    y2: 1.0,
                    color: Color::DarkGray,
                });
            });

        frame.render_widget(canvas, rect);

        let canvas = Canvas::default()
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                ctx.draw(&center_box);
            });

        frame.render_widget(canvas, rect);

        let canvas = Canvas::default()
            .x_bounds([-1.0, 1.0])
            .y_bounds([-1.0, 1.0])
            .paint(|ctx| {
                ctx.draw(&Line {
                    x1: center.0 - CROSSHAIR_SIZE,
                    y1: center.1,
                    x2: center.0 + CROSSHAIR_SIZE,
                    y2: center.1,
                    color: if center.0.abs() < center_box.width / 2.0 {
                        Color::Green
                    } else {
                        Color::Red
                    },
                });

                ctx.draw(&Line {
                    x1: center.0,
                    y1: center.1 - CROSSHAIR_SIZE,
                    x2: center.0,
                    y2: center.1 + CROSSHAIR_SIZE,
                    color: if center.1.abs() < center_box.height / 2.0 {
                        Color::Green
                    } else {
                        Color::Red
                    },
                });
            });

        frame.render_widget(canvas, rect);
    }
}
