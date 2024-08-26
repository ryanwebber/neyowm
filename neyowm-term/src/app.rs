use std::{
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use crossbeam_channel::select;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use neyowm::{Bridge, ClientBoundMessage, ServerBoundMessage};
use ratatui::{
    layout::{Margin, Position},
    prelude::{Backend, CrosstermBackend},
    style::{Color, Stylize},
    widgets::{Block, BorderType, Borders},
    Terminal,
};

use crate::ui::{AutopilotView, CommandView, ConnectionView, FlightVectorView, TelemetryView};

fn is_exit_event(event: &Event) -> bool {
    matches!(event, Event::Key(KeyEvent { code: KeyCode::Char('c'), modifiers, .. }) if modifiers.contains(KeyModifiers::CONTROL))
}

pub struct App(PhantomData<()>);

impl App {
    pub fn new() -> Self {
        Self(PhantomData)
    }

    pub fn run(self, bridge: Bridge) -> std::io::Result<()> {
        let mut term = {
            let mut stdout = std::io::stdout();

            crossterm::terminal::enable_raw_mode()?;
            crossterm::execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
            let backend = CrosstermBackend::new(stdout);
            Terminal::new(backend)?
        };

        let cancellation_token = Arc::new(Mutex::new(false));
        let (rx1, tx1) = bridge.into_inner();
        let (tx2, rx2) = crossbeam_channel::unbounded();
        let event_loop_join_handle = {
            let cancellation_token = cancellation_token.clone();
            std::thread::spawn(move || {
                loop {
                    if crossterm::event::poll(std::time::Duration::from_millis(50))? {
                        let e = crossterm::event::read()?;
                        let is_exit_event = is_exit_event(&e);
                        if let Err(_) = tx2.send(e) {
                            break;
                        }

                        // Bail out early, saving a few milliseconds of shutdown time
                        if is_exit_event {
                            break;
                        }
                    }

                    if let Ok(cancel) = cancellation_token.lock() {
                        if *cancel {
                            println!("Event loop cancelled");
                            break;
                        }
                    }
                }

                std::io::Result::Ok(())
            })
        };

        let bridge = BridgeSink { tx: tx1.clone() };
        let mut view_state = ViewState {
            view_in_focus: None,
            autopilot_view: AutopilotView::new(),
            command_view: CommandView::new(),
            connection_view: ConnectionView::new(),
            flight_vector_view: FlightVectorView::new(),
            telemetry_view: TelemetryView::new(),

            _phantom: std::marker::PhantomData,
        };

        // First draw, let's get the UI on screen without having to wait for an event
        _ = view_state.draw(&mut term);

        loop {
            let should_draw = select! {
                recv(rx1) -> msg => {
                    if let Ok(msg) = msg {
                        match msg {
                            ClientBoundMessage::Shutdown => {
                                println!("Shutdown message received");
                                break;
                            },
                            _ => {
                                view_state.handle_client_message(&bridge, msg)
                            }
                        }
                    } else {
                        false
                    }
                }
                recv(rx2) -> event => {
                    if let Ok(event) = event {
                        if is_exit_event(&event) {
                            tx1.send(ServerBoundMessage::Shutdown).unwrap();
                        } else {
                            view_state.handle_user_event(&bridge, event);
                        }
                    }

                    true
                }
            };

            if should_draw {
                _ = view_state.draw(&mut term);
            }
        }

        if let Ok(mut cancellation_token) = cancellation_token.lock() {
            *cancellation_token = true;
        }

        let _ = event_loop_join_handle.join();

        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(
            term.backend_mut(),
            crossterm::terminal::LeaveAlternateScreen,
        )?;

        term.show_cursor()?;

        Ok(())
    }
}

struct ViewState<'a> {
    view_in_focus: Option<FocusedView>,
    autopilot_view: AutopilotView,
    command_view: CommandView<'a>,
    connection_view: ConnectionView,
    flight_vector_view: FlightVectorView,
    telemetry_view: TelemetryView,

    _phantom: std::marker::PhantomData<&'a ()>,
}

impl<'a> ViewState<'a> {
    fn draw(
        &self,
        term: &mut ratatui::Terminal<impl ratatui::backend::Backend>,
    ) -> std::io::Result<()> {
        let mut effects = Effects::none();
        term.draw(|frame| {
            let layout = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .margin(0)
                .constraints(
                    [
                        ratatui::layout::Constraint::Length(1),
                        ratatui::layout::Constraint::Length(3),
                        ratatui::layout::Constraint::Min(0),
                        ratatui::layout::Constraint::Length(11),
                    ]
                    .as_ref(),
                )
                .split(frame.area());

            let layout = &layout[1..];
            const DEFAULT_MARGINS: Margin = Margin::new(1, 1);

            // Render the connection status
            self.connection_view
                .draw(layout[0].inner(DEFAULT_MARGINS), frame);

            // Render the main view
            {
                let layout = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Horizontal)
                    .margin(0)
                    .constraints(
                        [
                            ratatui::layout::Constraint::Min(28),
                            ratatui::layout::Constraint::Length(
                                ((layout[1].height - 2) * 2) as u16,
                            ),
                            ratatui::layout::Constraint::Min(44),
                        ]
                        .as_ref(),
                    )
                    .split(layout[1]);

                // Render the telemetry view
                {
                    let area = layout[0].inner(Margin::new(1, 0));
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" TELEMETRY ")
                        .white();

                    frame.render_widget(block, area);
                    self.telemetry_view.draw(area.inner(DEFAULT_MARGINS), frame);
                }

                // Render the canvas view
                {
                    let area = layout[1].inner(Margin::new(1, 0)).inner(Margin::new(5, 5));
                    self.flight_vector_view.draw(area, frame);
                }

                // Render the autopilot view
                {
                    let area = layout[2].inner(Margin::new(1, 0));
                    let block = Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(" AUTOPILOT [a] ")
                        .border_style({
                            if self.view_in_focus == Some(FocusedView::AutoPilotView) {
                                ratatui::style::Style::default().fg(Color::Yellow).bold()
                            } else {
                                ratatui::style::Style::default().fg(Color::DarkGray)
                            }
                        });

                    frame.render_widget(block, area);
                    self.autopilot_view.draw(
                        area.inner(DEFAULT_MARGINS),
                        frame,
                        &mut effects,
                        self.view_in_focus == Some(FocusedView::AutoPilotView),
                    );
                }
            }

            // Render the command view
            {
                let area = layout[2].inner(DEFAULT_MARGINS);
                let block = Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" COMMAND [/] ")
                    .border_style({
                        if self.view_in_focus == Some(FocusedView::CommandView) {
                            ratatui::style::Style::default().fg(Color::Yellow).bold()
                        } else {
                            ratatui::style::Style::default().fg(Color::DarkGray)
                        }
                    });

                frame.render_widget(block, area);
                self.command_view.draw(
                    area.inner(DEFAULT_MARGINS),
                    frame,
                    &mut effects,
                    self.view_in_focus == Some(FocusedView::CommandView),
                );
            }
        })?;

        effects.apply(term);

        Ok(())
    }

    fn handle_user_event(&mut self, bridge: &BridgeSink, event: crossterm::event::Event) {
        let propogate = match &event {
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                self.view_in_focus = None;
                false
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                ..
            }) if event_utils::is_nav_event(&event) => {
                self.view_in_focus = Some(FocusedView::CommandView);
                false
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('a'),
                ..
            }) if event_utils::is_nav_event(&event) => {
                self.view_in_focus = Some(FocusedView::AutoPilotView);
                false
            }
            _ => true,
        };

        if propogate {
            match self.view_in_focus {
                Some(FocusedView::CommandView) => {
                    self.command_view.handle_user_event(bridge, event);
                }
                Some(FocusedView::AutoPilotView) => {
                    self.autopilot_view.handle_user_event(bridge, event);
                }
                None => {
                    // Noop
                }
            }
        }
    }

    fn handle_client_message(&mut self, bridge: &BridgeSink, msg: ClientBoundMessage) -> bool {
        match &msg {
            ClientBoundMessage::Shutdown => return false,
            _ => {}
        }

        self.autopilot_view.handle_client_message(bridge, &msg);
        self.connection_view.handle_client_message(bridge, &msg);
        self.flight_vector_view.handle_client_message(bridge, &msg);
        self.telemetry_view.handle_client_message(bridge, &msg);

        true
    }
}

pub struct BridgeSink {
    tx: crossbeam_channel::Sender<ServerBoundMessage>,
}

impl BridgeSink {
    pub fn send(&self, msg: ServerBoundMessage) {
        let _ = self.tx.send(msg);
    }

    pub fn broadcast(&self, msg: ClientBoundMessage) {
        self.send(ServerBoundMessage::Broadcast(msg))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusedView {
    AutoPilotView,
    CommandView,
}

pub struct Effects {
    cursor_position: Option<(u16, u16)>,
}

impl Effects {
    pub fn none() -> Self {
        Self {
            cursor_position: None,
        }
    }

    pub fn set_cursor_position(&mut self, x: u16, y: u16) {
        self.cursor_position = Some((x, y));
    }

    pub fn apply(&self, term: &mut Terminal<impl Backend>) {
        match self.cursor_position {
            Some((x, y)) => {
                _ = term.show_cursor();
                _ = term.set_cursor_position(Position { x, y });
            }
            None => {
                _ = term.hide_cursor();
            }
        }
    }
}

pub mod event_utils {
    use crossterm::event::{Event, KeyEvent, KeyModifiers};

    pub fn is_nav_event(e: &Event) -> bool {
        match e {
            Event::Key(KeyEvent { modifiers, .. }) if modifiers.contains(KeyModifiers::ALT) => true,
            _ => false,
        }
    }
}
