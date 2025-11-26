mod terminal;

use clap::Parser as _;
use ratatui::{
    crossterm::{
        self,
        event::{Event, KeyCode, KeyModifiers},
    },
    prelude::*,
};
use std::process::ExitCode;

#[derive(clap::Parser)]
struct Args {}

fn main() -> anyhow::Result<ExitCode> {
    let _args = Args::parse();

    let mut terminal = terminal::init();

    let mut state = State::default();

    loop {
        terminal.draw(|frame| {
            let area = frame.area();
            let buffer = frame.buffer_mut();
            render(&state, area, buffer);
        })?;

        let event = crossterm::event::read()?;
        if let Some(exit_code) = handle_event(&mut state, &event) {
            return Ok(exit_code);
        }
    }
}

#[derive(Default)]
struct State {}

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

fn render(_state: &State, area: Rect, buffer: &mut Buffer) {
    Line::raw("Hello, world!").render(area, buffer);
}
