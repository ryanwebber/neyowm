use neyowm::{ClientBoundMessage, ConnectionStatus};
use ratatui::{layout::Rect, Frame};

use crate::app::BridgeSink;

pub struct ConnectionView {
    connected: bool,
}

impl ConnectionView {
    pub fn new() -> Self {
        ConnectionView { connected: false }
    }

    pub fn handle_client_message(&mut self, _: &BridgeSink, msg: &ClientBoundMessage) {
        match msg {
            ClientBoundMessage::UpdateConnectionStatus(status) => {
                self.connected = *status == ConnectionStatus::Connected;
            }
            _ => {}
        }
    }

    pub fn draw(&self, rect: Rect, frame: &mut Frame) {
        let (status, style) = if self.connected {
            (
                "● CONNECTED",
                ratatui::style::Style::default().fg(ratatui::style::Color::Green),
            )
        } else {
            (
                "○ DISCONNECTED",
                ratatui::style::Style::default().fg(ratatui::style::Color::Red),
            )
        };

        frame.render_widget(ratatui::widgets::Paragraph::new(status).style(style), rect);
    }
}
