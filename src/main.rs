mod terminal;

use ansi_to_tui::IntoText as _;
use clap::Parser as _;
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers, MouseEventKind},
    },
    prelude::*,
};
use std::{
    fs,
    io::{self, Read as _, Write as _},
    path::PathBuf,
    process::ExitCode,
};

#[derive(clap::Parser)]
struct Args {
    file: Option<PathBuf>,
}

fn main() -> anyhow::Result<ExitCode> {
    let args = Args::parse();

    // TODO: Don't read all input into memory at once naively.
    let input = if let Some(path) = &args.file {
        fs::read(path)?
    } else {
        let mut bytes = Vec::new();
        io::stdin().read_to_end(&mut bytes)?;
        bytes
    };

    let (_, terminal_lines) = crossterm::terminal::size()?;

    // TODO: Handle `\r\n` line endings.
    let input_lines = bytecount::count(&input, b'\n');

    if input_lines <= usize::from(terminal_lines) {
        // Input fits on screen, just print it without entering TUI.
        let mut stdout = io::stdout().lock();
        stdout.write_all(&input)?;
        stdout.flush()?;

        return Ok(ExitCode::SUCCESS);
    }

    let mut terminal = terminal::init();

    let text = input.into_text()?;

    let mut state = State {
        text,
        terminal_lines,
        input_lines,
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
            if matches!(event, Event::Resize(_, _)) {
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
    text: Text<'static>,
    terminal_lines: u16,
    input_lines: usize,
    vertical_scroll: usize,
}

impl State {
    fn scroll_up(&mut self, distance: usize) {
        self.vertical_scroll = self.vertical_scroll.saturating_sub(distance);
    }

    fn scroll_down(&mut self, distance: usize) {
        self.vertical_scroll += distance;
    }

    fn scroll_half_page_up(&mut self) {
        self.scroll_up(usize::from(self.terminal_lines) / 2);
    }

    fn scroll_half_page_down(&mut self) {
        self.scroll_down(usize::from(self.terminal_lines) / 2);
    }

    fn scroll_full_page_up(&mut self) {
        self.scroll_up(usize::from(self.terminal_lines));
    }

    fn scroll_full_page_down(&mut self) {
        self.scroll_down(usize::from(self.terminal_lines));
    }

    fn scroll_to_top(&mut self) {
        self.vertical_scroll = 0;
    }

    fn scroll_to_bottom(&mut self) {
        self.vertical_scroll = self
            .input_lines
            .saturating_sub(usize::from(self.terminal_lines));
    }
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
            (KeyModifiers::NONE, KeyCode::Char('k')) => state.scroll_up(1),
            (KeyModifiers::NONE, KeyCode::Char('j')) => state.scroll_down(1),
            (KeyModifiers::NONE, KeyCode::Char('u')) => state.scroll_half_page_up(),
            (KeyModifiers::NONE, KeyCode::Char('d')) => state.scroll_half_page_down(),
            (KeyModifiers::NONE, KeyCode::Char('b')) => state.scroll_full_page_up(),
            (KeyModifiers::NONE, KeyCode::Char('f')) => state.scroll_full_page_down(),
            (KeyModifiers::NONE, KeyCode::Char('g')) => state.scroll_to_top(),
            (KeyModifiers::NONE, KeyCode::Char('G'))
            | (KeyModifiers::SHIFT, KeyCode::Char('g' | 'G')) => state.scroll_to_bottom(),
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => exit_code = Some(ExitCode::SUCCESS),
            (KeyModifiers::NONE, KeyCode::Char('q')) => exit_code = Some(ExitCode::SUCCESS),
            _ => {}
        },
        Event::Mouse(mouse_event) => match (mouse_event.modifiers, mouse_event.kind) {
            (KeyModifiers::NONE, MouseEventKind::ScrollUp) => state.scroll_up(1),
            (KeyModifiers::NONE, MouseEventKind::ScrollDown) => state.scroll_down(1),
            _ => {}
        },
        _ => {}
    }

    exit_code
}

fn render(state: &State, area: Rect, buffer: &mut Buffer) {
    let text: Text = state
        .text
        .iter()
        .skip(state.vertical_scroll)
        .take(usize::from(state.terminal_lines))
        .cloned()
        .collect();
    text.render(area, buffer);
}
