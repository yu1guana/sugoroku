// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use crate::game_system::player_status::{PlayerOrder, PlayerStatus};
use crate::game_system::world::World;
use crate::preferences::{Language, Preferences};
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

#[derive(Debug)]
struct GameData {
    world: World,
    current_player: String,
    player_order: Vec<String>,
    player_status_table: HashMap<String, PlayerStatus>,
    ui_status: UiStatus,
    ui_status_buffer: UiStatus,
    text_set: TextSet,
}

#[derive(Clone, Debug)]
enum UiStatus {
    QuitMenu,
    TitleMenu,
    DiceRoll,
    DiceResult,
    GameFinished,
}

#[derive(Clone, Debug, Default)]
struct TextSet {
    main_window: String,
    message: String,
    dice_string: String,
    guidance: String,
    player_list: String,
}

impl GameData {
    fn try_new(
        world: World,
        player_order: Vec<String>,
        player_status_table: HashMap<String, PlayerStatus>,
    ) -> Result<Self> {
        let current_player = player_order
            .first()
            .ok_or_else(|| GameSystemError::NoPlayer)?
            .to_owned();
        Ok(Self {
            world,
            current_player,
            player_order,
            player_status_table,
            ui_status: UiStatus::TitleMenu,
            ui_status_buffer: UiStatus::TitleMenu,
            text_set: Default::default(),
        })
    }
}

impl TextSet {
    fn set_guidance(&mut self, preferences: &Preferences) {
        self.guidance.clear();
        match preferences.language() {
            Language::Japanese => {
                self.guidance.push_str("ESC: 終了\n");
                self.guidance.push_str("Ctrl-l: 再描画\n");
                self.guidance.push_str("Ctrl-t: タイトル画面の表示");
            }
        }
    }
    fn set_player_list(
        &mut self,
        preferences: &Preferences,
        current_player: &str,
        player_order: &[String],
        player_status_table: &HashMap<String, PlayerStatus>,
    ) -> Result<()> {
        const GOAL_MARK: &'static str = "🏁 ";
        const DICE_MARK: &'static str = "🎲 ";
        self.player_list.clear();
        self.player_list.push_str(GOAL_MARK);
        self.player_list.push_str("   ");
        self.player_list.push_str("Name");
        self.player_list.push_str(match preferences.language() {
            Language::Japanese => "名前",
        });
        self.player_list.push_str("\n");
        for player in player_order {
            let order_of_arrival = player_status_table
                .get(player)
                .ok_or_else(|| GameSystemError::NotFoundPlayer(player.to_owned()))?
                .order_of_arrival();
            match order_of_arrival {
                Some(x) => self.player_list.push_str(&format!("{0:>2} ", x)),
                None => self.player_list.push_str(&format!("{0:>2} ", "")),
            }
            if player == current_player {
                self.player_list.push_str(DICE_MARK);
            } else {
                self.player_list.push_str("   ");
            }
            self.player_list.push_str(player);
            self.player_list.push_str("\n");
        }
        Ok(())
    }
    fn set_prompt_dice_roll(&mut self, preferences: &Preferences) {
        self.message.clear();
        match preferences.language() {
            Language::Japanese => {
                self.message.push_str("サイコロを振ってください。 >>> ");
            }
        }
        self.message.push_str(self.dice_string.as_str());
    }
    fn set_prompt_enter(&mut self, preferences: &Preferences) {
        self.message.clear();
        match preferences.language() {
            Language::Japanese => self.message.push_str("エンターキーを押してください。"),
        }
    }
    fn set_prompt_game_finish(&mut self, preferences: &Preferences) {
        self.message.clear();
        self.main_window.clear();
        match preferences.language() {
            Language::Japanese => self
                .message
                .push_str("全員ゴールしました。\nゲームを終了してください。"),
        }
    }
    fn set_dice_is_out_of_range(&mut self, preferences: &Preferences, dice: usize) {
        match preferences.language() {
            Language::Japanese => {
                self.main_window = format!("サイコロの値が範囲外です: {}", dice);
            }
        }
    }
}

pub fn run(
    preferences: Preferences,
    world: World,
    player_order: Vec<String>,
    player_status_table: HashMap<String, PlayerStatus>,
) -> Result<()> {
    let stdout = termion::screen::AlternateScreen::from(io::stdout().into_raw_mode()?);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut game_data = GameData::try_new(world, player_order, player_status_table)?;
    game_data.text_set.set_guidance(&preferences);
    game_data.text_set.set_prompt_dice_roll(&preferences);
    game_data.text_set.set_player_list(
        &preferences,
        &game_data.current_player,
        &game_data.player_order,
        &game_data.player_status_table,
    )?;
    game_data.text_set.main_window = game_data.world.start_description(&preferences);
    terminal.hide_cursor()?;
    terminal.draw(|frame| ui(frame, &preferences, &game_data))?;
    while let Some(Ok(key)) = io::stdin().keys().next() {
        match &game_data.ui_status {
            UiStatus::TitleMenu => {
                title_menu(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::DiceRoll => {
                dice_roll(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::DiceResult => {
                dice_result(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::QuitMenu => {
                if quit_menu(&preferences, &mut terminal, &mut game_data, key)? {
                    break;
                }
            }
            UiStatus::GameFinished => {
                game_finished(&preferences, &mut terminal, &mut game_data, key)?
            }
        }
    }
    Ok(())
}

fn title_menu<B: Backend>(
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

fn dice_roll<B: Backend>(
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

fn dice_result<B: Backend>(
    preferences: &Preferences,
    terminal: &mut Terminal<B>,
    game_data: &mut GameData,
    key: Key,
) -> Result<()> {
    match key {
        Key::Char('\n') => {
            game_data.ui_status = UiStatus::DiceRoll;
            game_data.ui_status_buffer = UiStatus::DiceRoll;
            game_data.text_set.dice_string.clear();
            game_data.text_set.main_window.clear();
            game_data.text_set.set_prompt_dice_roll(preferences);
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

fn game_finished<B: Backend>(
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

fn quit_menu<B: Backend>(
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

fn ui<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences, game_data: &GameData) {
    match game_data.ui_status {
        UiStatus::TitleMenu => ui_title(frame, preferences, &game_data),
        UiStatus::QuitMenu => {
            ui_quit(frame, preferences);
        }
        _ => ui_playing(frame, preferences, game_data),
    }
}

fn ui_title<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences, game_data: &GameData) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new(game_data.world.title())
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let mut opening_msg_text = String::new();
    opening_msg_text.push('\n');
    opening_msg_text.push_str(game_data.world.opening_msg());
    opening_msg_text.push('\n');
    opening_msg_text.push('\n');
    match game_data.ui_status_buffer {
        UiStatus::TitleMenu => {
            opening_msg_text.push_str(match preferences.language() {
                Language::Japanese => "開始するにはエンターキーを押してください。",
            });
        }
        _ => {
            opening_msg_text.push_str(match preferences.language() {
                Language::Japanese => "ゲームに戻るにはエンターキーを押してください。",
            });
        }
    }
    let opening_msg = Paragraph::new(opening_msg_text)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
}

fn ui_playing<B: Backend>(frame: &mut Frame<B>, _preferences: &Preferences, game_data: &GameData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(frame.size());
    frame.render_widget(
        Paragraph::new(game_data.text_set.guidance.as_str())
            .block(Block::default().title("Guidance").borders(Borders::ALL)),
        chunks[0],
    );
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(chunks[1]);
    frame.render_widget(
        Paragraph::new(game_data.text_set.player_list.as_str())
            .block(Block::default().title("Player list").borders(Borders::ALL)),
        bottom_chunks[0],
    );
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(bottom_chunks[1]);
    frame.render_widget(
        Paragraph::new(game_data.text_set.message.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL)),
        right_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(game_data.text_set.main_window.as_str())
            .block(Block::default().borders(Borders::ALL)),
        right_chunks[1],
    );
}

fn ui_quit<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new(match preferences.language() {
        Language::Japanese => "ゲームを終了しますか？",
    })
    .alignment(Alignment::Center)
    .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let opening_msg = Paragraph::new("Y / [n]")
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
}
