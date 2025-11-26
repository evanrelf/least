mod terminal;

use clap::Parser as _;
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers, MouseEventKind},
    },
    prelude::*,
};
use std::process::ExitCode;

#[derive(clap::Parser)]
struct Args {}

fn main() -> anyhow::Result<ExitCode> {
    let _args = Args::parse();

    let mut terminal = terminal::init();

    let mut state = State {
        message: String::from("Hello, world!"),
    };

    'frame: loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            render(&state, area, buffer);
        })?;

        let event = 'event: loop {
            let event = crossterm::event::read()?;

            // Immediately re-render when the terminal is resized.
            if event.is_resize() {
                continue 'frame;
            }

            // Don't re-render on spammy events that we ignore (e.g. mouse movement).
            if !should_skip_event(&event) {
                break 'event event;
            }
        };

        if let Some(exit_code) = handle_event(&mut state, &event) {
            return Ok(exit_code);
        }
    }
}

struct State {
    message: String,
}

fn should_skip_event(event: &Event) -> bool {
    match event {
        Event::Mouse(mouse_event) => matches!(
            mouse_event.kind,
            MouseEventKind::Moved | MouseEventKind::ScrollLeft | MouseEventKind::ScrollRight
        ),
        _ => false,
    }
}

fn handle_event(_state: &mut State, event: &Event) -> Option<ExitCode> {
    let mut exit_code = None;

    #[expect(clippy::single_match)]
    #[expect(clippy::match_same_arms)]
    match event {
        Event::Key(key_event) => match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => exit_code = Some(ExitCode::SUCCESS),
            (KeyModifiers::NONE, KeyCode::Char('q')) => exit_code = Some(ExitCode::SUCCESS),
            _ => {}
        },
        _ => {}
    }

    exit_code
}

fn render(state: &State, area: Rect, buffer: &mut Buffer) {
    Line::raw(&state.message).render(area, buffer);
}
