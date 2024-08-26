use crossterm::event::{KeyCode, KeyEvent};
use neyowm::ServerBoundMessage;
use ratatui::{
    layout::{self, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style, Stylize},
    widgets::Paragraph,
    Frame,
};
use tui_textarea::TextArea;

use crate::app::{BridgeSink, Effects};

pub struct CommandView<'a> {
    prompt_field: TextArea<'a>,
}

impl<'a> CommandView<'a> {
    pub fn new() -> Self {
        let mut prompt_field = TextArea::default();

        // Prevent the text area widget from managing the cursor
        // because it overwrites the default cursor style the terminal has configured
        prompt_field.set_cursor_style(Style::default().hidden());

        CommandView { prompt_field }
    }

    pub fn handle_user_event(&mut self, bridge: &BridgeSink, event: crossterm::event::Event) {
        if matches!(
            event,
            crossterm::event::Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })
        ) {
            // TODO: Actually handle commands here
            let command = self.prompt_field.lines()[0].as_str();
            if command == "exit" {
                bridge.send(ServerBoundMessage::Shutdown);
            }

            // Is there really no better way to manipulate the buffer?
            // https://github.com/rhysd/tui-textarea/issues/57
            self.prompt_field = {
                let mut prompt_field = TextArea::default();
                prompt_field.set_cursor_style(Style::default().hidden());
                prompt_field
            };

            return;
        }

        self.prompt_field.input(event);
    }

    pub fn draw(&self, rect: Rect, frame: &mut Frame, effects: &mut Effects, in_focus: bool) {
        let rect = rect.inner(Margin::new(1, 1));
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .split(rect);

        // Prompt
        {
            let layout = layout::Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(2), Constraint::Min(0)])
                .split(layout[0]);

            frame.render_widget(Paragraph::new("> ").dark_gray(), layout[0]);
            frame.render_widget(
                Paragraph::new(self.prompt_field.lines()[0].clone())
                    .bold()
                    .fg(if in_focus {
                        Color::White
                    } else {
                        Color::DarkGray
                    }),
                layout[1],
            );

            if in_focus {
                let (cursor_y, cursor_x) = self.prompt_field.cursor();
                effects.set_cursor_position(
                    (cursor_x as u16) + layout[1].x,
                    (cursor_y as u16) + layout[1].y,
                );
            }
        }
    }
}
