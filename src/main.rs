mod terminal;

use clap::Parser as _;
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers, MouseEventKind},
    },
    prelude::*,
    widgets::Paragraph,
};
use std::{fs, io, path::PathBuf, process::ExitCode};

#[derive(clap::Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    let mut terminal = terminal::init();

    let input = if let Some(path) = &args.file {
        fs::read_to_string(path)?
    } else {
        io::read_to_string(io::stdin())?
    };

    let mut state = State {
        input,
        vertical_scroll: 0,
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
    input: String,
    vertical_scroll: u16,
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

fn handle_event(state: &mut State, event: &Event) -> Option<ExitCode> {
    let mut exit_code = None;

    #[expect(clippy::match_same_arms)]
    match event {
        Event::Key(key_event) => match (key_event.modifiers, key_event.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => exit_code = Some(ExitCode::SUCCESS),
            (KeyModifiers::NONE, KeyCode::Char('q')) => exit_code = Some(ExitCode::SUCCESS),
            _ => {}
        },
        Event::Mouse(mouse_event) => match (mouse_event.modifiers, mouse_event.kind) {
            (KeyModifiers::NONE, MouseEventKind::ScrollUp) => {
                state.vertical_scroll = state.vertical_scroll.saturating_sub(1);
            }
            (KeyModifiers::NONE, MouseEventKind::ScrollDown) => state.vertical_scroll += 1,
            _ => {}
        },
        _ => {}
    }

    exit_code
}

fn render(state: &State, area: Rect, buffer: &mut Buffer) {
    Paragraph::new(&*state.input)
        .scroll((state.vertical_scroll, 0))
        .render(area, buffer);
}
