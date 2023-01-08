// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

mod screen;
mod status;

use crate::game_system::toml_interface::{read_player_list_from_file, read_world_from_file};
use crate::preferences::Preferences;
use crate::user_interface::tui::screen::ui;
use crate::user_interface::tui::status::GameData;
use anyhow::Result;
use std::io;
use std::path::PathBuf;
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::terminal::Terminal;

pub fn run(
    preferences: Preferences,
    player_list_file_path: PathBuf,
    world_file_path: PathBuf,
) -> Result<()> {
    let (player_order, player_status_table) = read_player_list_from_file(&player_list_file_path)?;
    let world = read_world_from_file(&world_file_path)?;
    let stdout = termion::screen::AlternateScreen::from(io::stdout().into_raw_mode()?);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut game_data = GameData::try_new(world, player_order, player_status_table)?;
    game_data.init(&preferences)?;
    terminal.hide_cursor()?;
    terminal.draw(|frame| ui(frame, &preferences, &game_data))?;
    while let Some(Ok(key)) = io::stdin().keys().next() {
        if game_data.transition(&preferences, key)? {
            break;
        }
        terminal.draw(|frame| ui(frame, &preferences, &game_data))?;
    }
    Ok(())
}
