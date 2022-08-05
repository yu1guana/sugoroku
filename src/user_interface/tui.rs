// Copyright (c) 2022 Yuichi Ishida

use crate::error::GeneralError;
use crate::player_status::{PlayerOrder, PlayerStatus};
use crate::preferences::Preferences;
use crate::world::World;
use anyhow::Result;
use std::collections::HashMap;
use std::io;
use termion;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::terminal::{Frame, Terminal};
use tui::widgets::{Block, Borders, Paragraph};

pub fn run(
    preferences: Preferences,
    world: World,
    player_order: Vec<String>,
    player_status_table: HashMap<String, PlayerStatus>,
) -> Result<()> {
    let stdout = termion::screen::AlternateScreen::from(io::stdout().into_raw_mode().unwrap());
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut world = world;
    let mut player_status_table = player_status_table;
    terminal.hide_cursor()?;
    display_title(&mut terminal, &preferences, &world)?;
    play(
        &mut terminal,
        &preferences,
        &mut world,
        &player_order,
        &mut player_status_table,
    )?;
    Ok(())
}

fn display_title<B: Backend>(
    terminal: &mut Terminal<B>,
    preferences: &Preferences,
    world: &World,
) -> Result<()> {
    terminal.draw(|frame| ui_title(frame, preferences, world).unwrap())?;
    for key in io::stdin().keys() {
        match key {
            Ok(Key::Char('\n')) => break,
            _ => {}
        }
    }
    Ok(())
}

fn ui_title<B: Backend>(
    frame: &mut Frame<B>,
    _preferences: &Preferences,
    world: &World,
) -> Result<()> {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new(world.title())
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let opening_msg =
        world.opening_msg().to_owned() + "\n" + "開始するにはエンターキーを押してください。";
    let opening_msg = Paragraph::new(opening_msg)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
    Ok(())
}

fn play<B: Backend>(
    terminal: &mut Terminal<B>,
    preferences: &Preferences,
    world: &mut World,
    player_order: &[String],
    player_status_table: &mut HashMap<String, PlayerStatus>,
) -> Result<()> {
    let mut current_player = player_order.first().unwrap().to_owned();
    const DICE_ROLL_MSG: &str = "サイコロを振ってください。 >>> ";
    let mut message_window_text = DICE_ROLL_MSG.to_string();
    let mut dice_string = String::new();
    let mut body_text = world.start_description(preferences);
    terminal.draw(|frame| {
        ui_playing(
            frame,
            &message_window_text,
            &body_text,
            world,
            &current_player,
            player_order,
            player_status_table,
        )
        .unwrap()
    })?;
    for key in io::stdin().keys() {
        match key {
            Ok(Key::Esc) => {
                if quit_menu(terminal)? {
                    return Ok(());
                } else {
                    terminal.draw(|frame| {
                        ui_playing(
                            frame,
                            &message_window_text,
                            &body_text,
                            world,
                            &current_player,
                            player_order,
                            player_status_table,
                        )
                        .unwrap()
                    })?;
                };
            }
            Ok(Key::Ctrl('l')) => {
                terminal.clear()?;
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text,
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
            Ok(Key::Char(c)) => {
                if c.is_ascii_digit() {
                    dice_string.push(c);
                } else if c == '\n' && !dice_string.is_empty() {
                    let dice = dice_string.parse().unwrap();
                    dice_string.clear();
                    body_text = match world.dice_roll(
                        &preferences,
                        dice,
                        &current_player,
                        player_order,
                        player_status_table,
                    ) {
                        Ok(body_text) => {
                            match player_order.next_player(&current_player, player_status_table)? {
                                Some(player) => current_player = player.to_owned(),
                                None => {
                                    break;
                                }
                            };
                            body_text
                        }
                        Err(e) => format!("{:?}", e),
                    };
                }
                message_window_text = DICE_ROLL_MSG.to_owned() + &dice_string;
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text.clone(),
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
            Ok(Key::Backspace) => {
                dice_string.pop();
                message_window_text = DICE_ROLL_MSG.to_owned() + &dice_string;
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text,
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
            _ => {
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text,
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
        }
    }
    message_window_text = "全員ゴールしました。ゲームを終了してください。".to_owned();
    for key in io::stdin().keys() {
        match key {
            Ok(Key::Esc) => {
                if quit_menu(terminal)? {
                    return Ok(());
                } else {
                    terminal.draw(|frame| {
                        ui_playing(
                            frame,
                            &message_window_text,
                            &body_text,
                            world,
                            &current_player,
                            player_order,
                            player_status_table,
                        )
                        .unwrap()
                    })?;
                };
            }
            Ok(Key::Ctrl('l')) => {
                terminal.clear()?;
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text,
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
            _ => {
                terminal.draw(|frame| {
                    ui_playing(
                        frame,
                        &message_window_text,
                        &body_text,
                        world,
                        &current_player,
                        player_order,
                        player_status_table,
                    )
                    .unwrap()
                })?;
            }
        }
    }
    Ok(())
}

fn quit_menu<B: Backend>(terminal: &mut Terminal<B>) -> Result<bool> {
    terminal.draw(|frame| ui_quit(frame))?;
    match io::stdin().keys().next().unwrap() {
        Ok(Key::Char('Y')) => return Ok(true),
        _ => return Ok(false),
    }
}

fn ui_quit<B: Backend>(frame: &mut Frame<B>) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new("ゲームを終了しますか？")
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let opening_msg = Paragraph::new("Y / [n]")
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
}

fn ui_playing<B: Backend>(
    frame: &mut Frame<B>,
    message_window_text: &str,
    body_window_text: &str,
    world: &World,
    current_player: &str,
    player_order: &[String],
    player_status_table: &HashMap<String, PlayerStatus>,
) -> Result<()> {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(frame.size());
    frame.render_widget(guidance_window()?, chunks[0]);
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(chunks[1]);
    frame.render_widget(
        player_list_window(world, current_player, player_order, player_status_table)?,
        bottom_chunks[0],
    );
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(bottom_chunks[1]);
    let message_window = Paragraph::new(message_window_text)
        .block(Block::default().title("Input").borders(Borders::ALL));
    frame.render_widget(message_window, right_chunks[0]);
    let body_window =
        Paragraph::new(body_window_text).block(Block::default().borders(Borders::ALL));
    frame.render_widget(body_window, right_chunks[1]);
    Ok(())
}

fn player_list_window<'a>(
    _world: &World,
    current_player: &str,
    player_order: &[String],
    player_status_table: &HashMap<String, PlayerStatus>,
) -> Result<Paragraph<'a>> {
    let block = Block::default().title("Player list").borders(Borders::ALL);
    let goal_mark = "🏁 ";
    let dice_mark = "🎲 ";
    let mut text = goal_mark.to_owned() + "   Name\n";
    for player in player_order {
        let order_of_arrival = player_status_table
            .get(player)
            .ok_or_else(|| GeneralError::NotFoundPlayer(player.to_owned()))?
            .order_of_arrival();
        match order_of_arrival {
            Some(x) => text += &format!("{0:>2} ", x),
            None => text += &format!("{0:>2} ", ""),
        }
        if player == current_player {
            text += dice_mark;
        } else {
            text += "   "
        }
        text += player;
        text += "\n";
    }
    Ok(Paragraph::new(text).block(block))
}

fn guidance_window<'a>() -> Result<Paragraph<'a>> {
    let block = Block::default().title("Guidance").borders(Borders::ALL);
    let mut text = "ESC: Quit\n".to_owned();
    text += "Ctrl-l: Rewrite window";
    Ok(Paragraph::new(text).block(block))
}
