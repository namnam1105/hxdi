/*
hexi - a TUI hex editor
Copyright (C) 2026 namnam1105

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.
*/

mod actions;
mod app;
mod events;
mod render;
mod types;

use crossterm::event;
use crossterm::event::DisableMouseCapture;
use crossterm::event::EnableMouseCapture;
use crossterm::execute;
use crossterm::terminal::*;
use ratatui::Terminal;
use ratatui::backend::{Backend, CrosstermBackend};

use std::io;
use std::io::stdout;

use actions::Action;
use app::App;

pub fn run(
    data: Vec<u8>,
    file_name: Option<String>,
    read_only: bool,
    color: bool,
    show_header: bool,
    show_offsets: bool,
    show_hex: bool,
    show_ascii: bool,
) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut app = App::new(
        data,
        file_name,
        read_only,
        color,
        show_header,
        show_offsets,
        show_hex,
        show_ascii,
    );
    let result = run_loop(&mut terminal, &mut app);
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;
    result
}

fn run_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()>
where
    std::io::Error: From<<B as Backend>::Error>,
{
    loop {
        // Expire status message
        if let Some(until) = app.status_msg_until {
            if std::time::Instant::now() >= until {
                app.status_msg = None;
                app.status_msg_until = None;
            }
        }

        terminal.draw(|f| render::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(16))? {
            match events::handle_event(app, event::read()?) {
                Action::Quit => break,
                Action::SaveQuit => {
                    save_file(app)?;
                    break;
                }
                Action::SaveFile => save_file(app)?,
                Action::None => {}
            }
        }
    }
    Ok(())
}

fn save_file(app: &App) -> io::Result<()> {
    if let Some(path) = &app.file_name {
        std::fs::write(path, &app.data)?;
    }
    Ok(())
}
