// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use crate::game_system::player_status::PlayerOrder;
use crate::preferences::Preferences;
use crate::user_interface::tui::screen::ui;
use crate::user_interface::tui::{GameData, UiStatus};
use anyhow::Result;
use termion;
use termion::event::Key;
use tui::backend::Backend;
use tui::terminal::Terminal;

pub fn title_menu<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char('\n') => {
            game_data.ui_status = UiStatus::DiceRoll;
            game_data.ui_status_buffer = UiStatus::DiceRoll;
        }
        Key::Esc => {
            game_data.ui_status = UiStatus::QuitMenu;
        }
        Key::Ctrl('l') => terminal.clear()?,
        _ => return Ok(()),
    }
    terminal.draw(|frame| ui(frame, preferences, &game_data))?;
    Ok(())
}

pub fn dice_roll<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char(c) => {
            match c {
                '0'..='9' => {
                    game_data.text_set.dice_string.push(c);
                    game_data.text_set.set_prompt_dice_roll(preferences);
                }
                '\n' => {
                    if game_data.text_set.dice_string.is_empty() {
                        return Ok(());
                    }
                    game_data.text_set.set_prompt_enter(preferences);
                    match game_data.world.dice_roll(
                        preferences,
                        game_data.text_set.dice_string.parse()?,
                        &game_data.current_player,
                        &game_data.player_order,
                        &mut game_data.player_status_table,
                    ) {
                        Ok(main_window_text) => {
                            game_data.text_set.main_window = main_window_text;
                            change_player(game_data)?;
                        }
                        Err(GameSystemError::OutOfRangeDice(dice)) => {
                            game_data.ui_status = UiStatus::DiceResult;
                            game_data.ui_status_buffer = UiStatus::DiceResult;
                            game_data
                                .text_set
                                .set_dice_is_out_of_range(preferences, dice);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
                _ => {}
            };
        }
        Key::Backspace => {
            game_data.text_set.dice_string.pop();
            game_data.text_set.set_prompt_dice_roll(preferences);
        }
        Key::Esc => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::QuitMenu;
        }
        Key::Ctrl('t') => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::TitleMenu;
        }
        Key::Ctrl('l') => terminal.clear()?,
        _ => return Ok(()),
    }
    terminal.draw(|frame| ui(frame, preferences, &game_data))?;
    Ok(())
}

pub fn skip<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char('\n') => {
            game_data
                .player_status_table
                .get_mut(&game_data.current_player)
                .ok_or_else(|| {
                    GameSystemError::NotFoundPlayer(game_data.current_player.to_owned())
                })?
                .sub_num_skip(1);
            game_data.text_set.set_prompt_enter(preferences);
            game_data.text_set.main_window.clear();
            change_player(game_data)?;
        }
        Key::Esc => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::QuitMenu;
        }
        Key::Ctrl('t') => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::TitleMenu;
        }
        Key::Ctrl('l') => terminal.clear()?,
        _ => return Ok(()),
    }
    terminal.draw(|frame| ui(frame, preferences, &game_data))?;
    Ok(())
}

pub fn dice_result<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char('\n') => {
            let num_skip_of_current_player = game_data
                .player_status_table
                .get(&game_data.current_player)
                .ok_or_else(|| {
                    GameSystemError::NotFoundPlayer(game_data.current_player.to_owned())
                })?
                .num_skip();
            if num_skip_of_current_player == 0 {
                game_data.ui_status = UiStatus::DiceRoll;
                game_data.ui_status_buffer = UiStatus::DiceRoll;
                game_data.text_set.dice_string.clear();
                game_data.text_set.main_window.clear();
                game_data.text_set.set_prompt_dice_roll(preferences);
            } else {
                game_data.ui_status = UiStatus::Skip;
                game_data.ui_status_buffer = UiStatus::Skip;
                game_data.text_set.set_prompt_enter(preferences);
                game_data
                    .text_set
                    .set_skip_player(preferences, num_skip_of_current_player);
            };
            game_data.text_set.set_player_list(
                preferences,
                &game_data.current_player,
                &game_data.player_order,
                &game_data.player_status_table,
            )?;
        }
        Key::Esc => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::QuitMenu;
        }
        Key::Ctrl('t') => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::TitleMenu;
        }
        Key::Ctrl('l') => terminal.clear()?,
        _ => return Ok(()),
    }
    terminal.draw(|frame| ui(frame, preferences, &game_data))?;
    Ok(())
}

pub fn game_finished<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char('\n') => {
            game_data.text_set.set_prompt_game_finish(preferences);
        }
        Key::Esc => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::QuitMenu;
        }
        Key::Ctrl('t') => {
            game_data.ui_status_buffer = game_data.ui_status.clone();
            game_data.ui_status = UiStatus::TitleMenu;
        }
        Key::Ctrl('l') => terminal.clear()?,
        _ => return Ok(()),
    }
    terminal.draw(|frame| ui(frame, preferences, &game_data))?;
    Ok(())
}

pub fn quit_menu<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<bool> {
    match key {
        Key::Char('Y') => return Ok(true),
        Key::Ctrl('l') => {
            terminal.clear()?;
            Ok(false)
        }
        _ => {
            game_data.ui_status = game_data.ui_status_buffer.clone();
            terminal.draw(|frame| ui(frame, preferences, &game_data))?;
            Ok(false)
        }
    }
}

fn change_player(game_data: &mut GameData) -> Result<()> {
    match game_data.player_order.next_player(
        &game_data.current_player,
        &mut game_data.player_status_table,
    )? {
        Some(player) => {
            game_data.ui_status = UiStatus::DiceResult;
            game_data.ui_status_buffer = UiStatus::DiceResult;
            game_data.current_player = player.to_owned();
        }
        None => {
            game_data.ui_status = UiStatus::GameFinished;
            game_data.ui_status_buffer = UiStatus::GameFinished;
        }
    }
    Ok(())
}
