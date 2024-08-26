use std::fmt::Write;

use neyowm::{ClientBoundMessage, TelemetryUpdate};
use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Cell, Row, Table},
    Frame,
};

use crate::app::BridgeSink;

pub struct TelemetryView {
    pitch_row: DisplayRow,
    roll_row: DisplayRow,
    yaw_row: DisplayRow,

    altitude_row: DisplayRow,
    latitude_row: DisplayRow,
    longitude_row: DisplayRow,

    elevation_row: DisplayRow,
    velocity_row: DisplayRow,

    aileron_row: DisplayRow,
    elevator_row: DisplayRow,
    rudder_row: DisplayRow,
    throttle_row: DisplayRow,
    flaps_row: DisplayRow,
    speedbrake_row: DisplayRow,
}

impl TelemetryView {
    pub fn new() -> Self {
        TelemetryView {
            pitch_row: DisplayRow::new("PITCH"),
            roll_row: DisplayRow::new("ROLL"),
            yaw_row: DisplayRow::new("YAW"),

            altitude_row: DisplayRow::new("ALT"),
            latitude_row: DisplayRow::new("LAT"),
            longitude_row: DisplayRow::new("LON"),

            elevation_row: DisplayRow::new("ELV"),
            velocity_row: DisplayRow::new("SPEED"),

            aileron_row: DisplayRow::new("AIL"),
            elevator_row: DisplayRow::new("ELVR"),
            rudder_row: DisplayRow::new("RUD"),
            throttle_row: DisplayRow::new("THROT"),
            flaps_row: DisplayRow::new("FLAPS"),
            speedbrake_row: DisplayRow::new("BRAKE"),
        }
    }

    pub fn handle_client_message(&mut self, _: &BridgeSink, msg: &ClientBoundMessage) {
        match msg {
            ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Orientation {
                pitch,
                roll,
                yaw,
            }) => {
                self.pitch_row.update(*pitch);
                self.roll_row.update(*roll);
                self.yaw_row.update(*yaw);
            }
            ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Position {
                latitude,
                longitude,
                altitude,
            }) => {
                self.latitude_row.update(*latitude);
                self.longitude_row.update(*longitude);
                self.altitude_row.update(*altitude);
            }
            ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Terrain {
                elevation,
                velocity,
                ..
            }) => {
                self.elevation_row.update(*elevation);
                self.velocity_row
                    .update(velocity.0.abs() + velocity.1.abs() + velocity.2.abs());
            }
            ClientBoundMessage::UpdateTelemetry(TelemetryUpdate::Control {
                aileron,
                elevator,
                rudder,
                throttle,
                flaps,
                speedbrake,
            }) => {
                self.aileron_row.update(*aileron);
                self.elevator_row.update(*elevator);
                self.rudder_row.update(*rudder);
                self.throttle_row.update(*throttle);
                self.flaps_row.update(*flaps);
                self.speedbrake_row.update(*speedbrake);
            }
            _ => {}
        }
    }

    pub fn draw(&self, rect: Rect, frame: &mut Frame) {
        let rect = rect.inner(ratatui::layout::Margin::new(1, 1));
        let rows = [
            Row::new(self.pitch_row.cells()),
            Row::new(self.roll_row.cells()),
            Row::new(self.yaw_row.cells()),
            Row::new([Cell::new(""); 0]),
            Row::new(self.altitude_row.cells()),
            Row::new(self.latitude_row.cells()),
            Row::new(self.longitude_row.cells()),
            Row::new([Cell::new(""); 0]),
            Row::new(self.elevation_row.cells()),
            Row::new(self.velocity_row.cells()),
            Row::new([Cell::new(""); 0]),
            Row::new(self.aileron_row.cells()),
            Row::new(self.elevator_row.cells()),
            Row::new(self.rudder_row.cells()),
            Row::new(self.throttle_row.cells()),
            Row::new(self.flaps_row.cells()),
            Row::new(self.speedbrake_row.cells()),
        ];

        let table = Table::new(rows, [Constraint::Min(0), Constraint::Length(18)]);
        frame.render_widget(table, rect);
    }
}

struct DisplayRow {
    label: &'static str,
    current_value: Option<f64>,
    previous_value: Option<f64>,
    cached_string_value: String,
}

impl DisplayRow {
    pub fn new(label: &'static str) -> Self {
        DisplayRow {
            label,
            current_value: None,
            previous_value: None,
            cached_string_value: String::new(),
        }
    }

    pub fn update(&mut self, value: f64) {
        self.previous_value = self.current_value;
        self.current_value = Some(value);

        self.cached_string_value.clear();

        if let Some(current_value) = self.current_value {
            _ = write!(&mut self.cached_string_value, "{:.6}", current_value);
        }
    }

    pub fn cells<'a>(&'a self) -> impl IntoIterator<Item = Cell> + 'a {
        [
            Cell::from(Span::styled(self.label, Style::default().dark_gray())),
            Cell::from(
                Line::from(Span::styled(
                    &self.cached_string_value,
                    Style::default().bold(),
                ))
                .right_aligned(),
            )
            .bg(Color::Black),
        ]
    }
}
