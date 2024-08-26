use crossterm::event::{Event, KeyCode};
use neyowm::{AutopilotMode, ClientBoundMessage};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Offset, Rect},
    style::{Color, Style, Stylize},
    text::Span,
    widgets::{Block, Borders, Paragraph, Tabs},
    Frame,
};

use crate::app::{event_utils, BridgeSink, Effects};

pub struct AutopilotView {
    mode: AutopilotModeKind,
    active_mode: AutopilotModeKind,
    autopilot_hold_form: Form<AutopilotHoldState>,
}

impl AutopilotView {
    pub fn new() -> Self {
        let autopilot_hold_state = AutopilotHoldState {
            roll: 0.0,
            pitch: 2.0,
            altitude: 10_000.0,
        };

        AutopilotView {
            mode: AutopilotModeKind::Off,
            active_mode: AutopilotModeKind::Off,
            autopilot_hold_form: Form::new(
                autopilot_hold_state.clone(),
                vec![
                    Field::new(
                        "ROLL",
                        format!("{:.01}", autopilot_hold_state.roll),
                        |value, state| {
                            if let Ok(value) = value.parse::<f64>() {
                                state.roll = value;
                                true
                            } else {
                                false
                            }
                        },
                    ),
                    Field::new(
                        "PITCH",
                        format!("{:.01}", autopilot_hold_state.pitch),
                        |value, state| {
                            if let Ok(value) = value.parse::<f64>() {
                                state.pitch = value;
                                true
                            } else {
                                false
                            }
                        },
                    ),
                    Field::new(
                        "ALT",
                        format!("{}", autopilot_hold_state.altitude),
                        |value, state| {
                            if let Ok(value) = value.parse::<f64>() {
                                state.altitude = value;
                                true
                            } else {
                                false
                            }
                        },
                    ),
                ],
            ),
        }
    }

    pub fn handle_client_message(&mut self, _: &BridgeSink, _: &ClientBoundMessage) {
        // Nothing to listen for yet
    }

    pub fn handle_user_event(&mut self, bridge: &BridgeSink, event: Event) {
        match &event {
            Event::Key(key) => {
                match key.code {
                    KeyCode::Char('1') if event_utils::is_nav_event(&event) => {
                        self.mode = AutopilotModeKind::Off;
                        self.active_mode = AutopilotModeKind::Off;
                        bridge.broadcast(ClientBoundMessage::SetAutopilotMode(AutopilotMode::Off));
                    }
                    KeyCode::Char('2') if event_utils::is_nav_event(&event) => {
                        self.mode = AutopilotModeKind::Hold;
                    }
                    KeyCode::Enter if event_utils::is_nav_event(&event) => {
                        let active_form_valid = match self.mode {
                            AutopilotModeKind::Off => true,
                            AutopilotModeKind::Hold => self.autopilot_hold_form.is_valid(),
                        };

                        if active_form_valid && self.active_mode != self.mode {
                            self.active_mode = self.mode;
                            bridge.broadcast(ClientBoundMessage::SetAutopilotMode(
                                match self.mode {
                                    AutopilotModeKind::Off => AutopilotMode::Off,
                                    AutopilotModeKind::Hold => AutopilotMode::Hold {
                                        roll: self.autopilot_hold_form.state.roll,
                                        pitch: self.autopilot_hold_form.state.pitch,
                                    },
                                },
                            ));
                        } else {
                            self.active_mode = AutopilotModeKind::Off;
                            bridge.broadcast(ClientBoundMessage::SetAutopilotMode(
                                AutopilotMode::Off,
                            ));
                        }
                    }
                    _ => {
                        match self.mode {
                            AutopilotModeKind::Hold => {
                                self.autopilot_hold_form.handle_user_event(event);
                            }
                            _ => {}
                        };
                    }
                };
            }
            _ => {}
        }
    }

    pub fn draw(&self, rect: Rect, frame: &mut Frame, effects: &mut Effects, is_focused: bool) {
        let rect = rect.inner(Margin::new(1, 1));

        let tabs = Tabs::new(vec![" OFF [1] ", " HOLD [2] "])
            .highlight_style({
                let style = Style::default().fg(Color::White);
                match self.mode {
                    AutopilotModeKind::Off => style.bg(Color::Red),
                    AutopilotModeKind::Hold => {
                        if self.active_mode == self.mode {
                            style.bg(Color::Green)
                        } else {
                            style.bg(Color::Yellow)
                        }
                    }
                }
            })
            .dark_gray()
            .select(match self.mode {
                AutopilotModeKind::Off => 0,
                AutopilotModeKind::Hold => 1,
            })
            .divider("");

        frame.render_widget(tabs, rect);

        let rect = rect
            .clone()
            .inner(Margin::new(1, 0))
            .intersection(rect.offset(Offset { x: 0, y: 1 }));

        let block = Block::default().borders(Borders::TOP).dark_gray();
        frame.render_widget(block, rect);

        let rect = rect
            .clone()
            .intersection(rect.offset(Offset { x: 0, y: 2 }));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0), Constraint::Length(1)])
            .split(rect);

        match &self.mode {
            AutopilotModeKind::Off => {
                // No form to render
            }
            AutopilotModeKind::Hold => {
                let (cursor_x, cursor_y) = self.autopilot_hold_form.draw(layout[0], frame);
                if is_focused {
                    effects.set_cursor_position(cursor_x, cursor_y);
                }
            }
        }

        if self.mode != AutopilotModeKind::Off {
            frame.render_widget(
                Paragraph::new(
                    Span::raw(" ACTIVE [<ENTER>] ")
                        .fg(if self.active_mode == self.mode {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .on_black()
                        .bold(),
                )
                .centered(),
                layout[1],
            );
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AutopilotModeKind {
    Off,
    Hold,
}

#[derive(Clone, Copy, PartialEq, Debug)]
struct AutopilotHoldState {
    roll: f64,
    pitch: f64,
    altitude: f64,
}

struct Form<S> {
    state: S,
    fields: Vec<Field<S>>,
    active_field_index: usize,
}

impl<S> Form<S> {
    fn new(state: S, fields: Vec<Field<S>>) -> Self {
        Form {
            state,
            fields,
            active_field_index: 0,
        }
    }

    fn is_valid(&self) -> bool {
        self.fields.iter().all(|field| field.is_valid)
    }

    fn handle_user_event(&mut self, event: Event) {
        match &event {
            Event::Key(key) => match key.code {
                KeyCode::Up | KeyCode::BackTab => {
                    if self.active_field_index > 0 {
                        self.active_field_index -= 1;
                    } else {
                        self.active_field_index = self.fields.len() - 1;
                    }
                }
                KeyCode::Down | KeyCode::Tab => {
                    if self.active_field_index < self.fields.len() - 1 {
                        self.active_field_index += 1;
                    } else {
                        self.active_field_index = 0;
                    }
                }
                _ => {
                    self.fields[self.active_field_index].handle_user_event(event, &mut self.state);
                }
            },
            _ => {}
        }
    }

    fn draw(&self, mut rect: Rect, frame: &mut Frame) -> (u16, u16) {
        let label_width = self
            .fields
            .iter()
            .map(|field| field.label.len())
            .max()
            .unwrap_or(0) as u16;

        let form_area = rect;
        let value_width = rect.width - label_width - 4;

        for field in self.fields.iter() {
            let layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(label_width),
                    Constraint::Min(0),
                    Constraint::Length(value_width),
                ])
                .split(Rect {
                    x: rect.x,
                    y: rect.y,
                    width: rect.width,
                    height: 1,
                });

            frame.render_widget(
                Paragraph::new(field.label.clone()).white().bold(),
                layout[0],
            );

            frame.render_widget(
                Paragraph::new(field.value.clone())
                    .bg(if field.is_valid {
                        Color::Black
                    } else {
                        Color::Red
                    })
                    .white(),
                layout[2],
            );

            rect = rect
                .clone()
                .intersection(rect.offset(Offset { x: 0, y: 2 }));
        }

        (
            form_area.x + form_area.width - value_width
                + self.fields[self.active_field_index].cursor_position as u16,
            form_area.y + self.active_field_index as u16 * 2,
        )
    }
}

struct Field<S> {
    label: String,
    value: String,
    is_valid: bool,
    on_edit: fn(&mut String, &mut S) -> bool,
    cursor_position: usize,
}

impl<S> Field<S> {
    fn new(
        label: impl Into<String>,
        value: impl Into<String>,
        on_edit: fn(&mut String, &mut S) -> bool,
    ) -> Self {
        let label = label.into();
        let value = value.into();
        let value_len = value.len();

        Field {
            label,
            value,
            on_edit,
            is_valid: true,
            cursor_position: value_len,
        }
    }

    fn handle_user_event(&mut self, event: Event, state: &mut S) {
        match &event {
            Event::Key(key) => match key.code {
                KeyCode::Char(c) => {
                    self.value.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                    self.is_valid = (self.on_edit)(&mut self.value, state);
                }
                KeyCode::Backspace => {
                    if self.cursor_position > 0 {
                        self.value.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                        self.is_valid = (self.on_edit)(&mut self.value, state);
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_position < self.value.len() {
                        self.value.remove(self.cursor_position);
                        self.is_valid = (self.on_edit)(&mut self.value, state);
                    }
                }
                KeyCode::Left => {
                    if self.cursor_position > 0 {
                        self.cursor_position -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_position < self.value.len() {
                        self.cursor_position += 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}
