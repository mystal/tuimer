use std::time::{Duration, Instant};

use notify_rust::{Notification, NotificationHandle, Timeout};
use ratatui::{
    prelude::*,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Rect},
    symbols::border,
    widgets::{
        Block, Paragraph,
        block::{Position, Title},
    },
};

#[derive(Default)]
enum TimerState {
    #[default]
    Off,
    Running {
        completion: Instant,
    },
    Paused {
        remaining: Duration,
    },
    Finished {
        completed: Instant,
        notify_handle: NotificationHandle,
    },
}

#[derive(Default)]
struct TimerNotifyConfig {
    // TODO: Play a sound. Whether to auto turn off sound. Notification.
}

struct TimerConfig {
    duration: Duration,
    label: String,
    notify_config: TimerNotifyConfig,
}

struct Timer {
    state: TimerState,
    config: TimerConfig,
}

impl Default for Timer {
    fn default() -> Self {
        let duration = Duration::from_secs(5);
        Self {
            state: TimerState::Running { completion: Instant::now() + duration },
            config: TimerConfig {
                label: "Placeholder".into(),
                duration,
                notify_config: TimerNotifyConfig::default(),
            }
        }
    }
}

#[derive(Default)]
struct App {
    timer: Timer,
    frame_count: u64,
    should_exit: bool,
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Title::from(" tuimer ".bold());
        let instructions = Title::from(Line::from(vec![
            " New Timer ".into(),
            "<N>".blue().bold(),
            " Quit ".into(),
            "<Q> ".blue().bold(),
        ]));
        let stats = Title::from(format!("{}", self.frame_count).bold());
        let block = Block::bordered()
            .title(title.alignment(Alignment::Center))
            .title(instructions
                .alignment(Alignment::Center)
                .position(Position::Bottom)
            )
            .title(stats
                .alignment(Alignment::Right)
                .position(Position::Bottom))
            .border_set(border::THICK);

        let timer_text = match self.timer.state {
            TimerState::Off => format!("Stopped ({:0.2}s)", self.timer.config.duration.as_secs_f64()),
            // TODO: Make the text blink when Paused.
            TimerState::Paused { remaining } => format!("{:0.2}s remaining [Paused]", remaining.as_secs_f64()),
            TimerState::Running { completion } => format!("{:0.2}s remaining", (completion - Instant::now()).as_secs_f64()),
            TimerState::Finished { completed, .. } => format!("Finished! ({:0.2}s ago)", (Instant::now() - completed).as_secs_f64()),
        };
        Paragraph::new(timer_text)
            .centered()
            .block(block)
            .render(area, buf)
    }
}

fn main() -> anyhow::Result<()> {
    let mut terminal = ratatui::init();

    let mut app = App::default();

    while !app.should_exit {
        // Render app.
        terminal.draw(|frame| {
            frame.render_widget(&app, frame.area());
            app.frame_count += 1;
        })?;

        // Poll for and handle any events.
        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key_event) => {
                    if key_event.kind == KeyEventKind::Press {
                        match key_event.code {
                            KeyCode::Char('n') => {
                                Notification::new()
                                    .summary("Test event")
                                    .show()?;
                            }
                            KeyCode::Char(' ') => {
                                match &app.timer.state {
                                    TimerState::Running { completion } => {
                                        app.timer.state = TimerState::Paused {
                                            remaining: completion.duration_since(Instant::now()),
                                        };
                                    }
                                    TimerState::Paused { remaining } => {
                                        app.timer.state = TimerState::Running {
                                            completion: Instant::now() + *remaining,
                                        };
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char('q') => app.should_exit = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // App logic.
        match &app.timer.state {
            TimerState::Running { completion } => {
                if *completion <= Instant::now() {
                    // Send notification!
                    let notify_handle = Notification::new()
                        .appname("tuimer")
                        .summary("Timer Finished")
                        .body(&format!("Timer {} has finished!", &app.timer.config.label))
                        .timeout(Timeout::Never)
                        .show()?;
                    app.timer.state = TimerState::Finished {
                        completed: *completion,
                        notify_handle,
                     };
                }
            }
            TimerState::Finished { notify_handle, .. } => {
                // TODO: Dismiss if notification is closed.
            }
            _ => {}
        }
    }

    ratatui::restore();

    Ok(())
}
