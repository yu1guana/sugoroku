// Copyright (c) 2022 Yuichi Ishida

mod screen;
mod transition;

use crate::error::GameSystemError;
use crate::game_system::player_status::PlayerStatus;
use crate::game_system::toml_interface::{read_player_list_from_file, read_world_from_file};
use crate::game_system::world::World;
use crate::preferences::{Language, Preferences};
use crate::user_interface::tui::screen::ui;
use anyhow::Result;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use termion;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::{Backend, TermionBackend};
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
                transition::title_menu(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::DiceRoll => {
                transition::dice_roll(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::Skip => {
                transition::skip(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::DiceResult => {
                transition::dice_result(&preferences, &mut terminal, &mut game_data, key)?;
            }
            UiStatus::QuitMenu => {
                if transition::quit_menu(&preferences, &mut terminal, &mut game_data, key)? {
                    break;
                }
            }
            UiStatus::GameFinished => {
                transition::game_finished(&preferences, &mut terminal, &mut game_data, key)?
            }
        }
    }
    Ok(())
}

#[derive(Debug)]
pub struct GameData {
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
    Skip,
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
    fn set_skip_player(&mut self, preferences: &Preferences, num_skip: u8) {
        match preferences.language() {
            Language::Japanese => {
                self.main_window = format!("プレイヤーはお休みです。カウント: {}", num_skip)
            }
        }
    }
}
