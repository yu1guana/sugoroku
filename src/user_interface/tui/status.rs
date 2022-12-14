// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use crate::error::GameSystemError;
use crate::game_system::player_status::PlayerOrder;
use crate::game_system::player_status::PlayerStatus;
use crate::game_system::world::World;
use crate::preferences::{Language, Preferences};
use anyhow::Result;
use std::collections::HashMap;
use std::fmt::Write as _;
use termion;
use termion::event::Key;

#[derive(Debug)]
pub struct GameData {
    pub world: World,
    pub current_player: String,
    pub player_order: Vec<String>,
    pub player_status_table: HashMap<String, PlayerStatus>,
    pub ui_status: UiStatus,
    pub ui_status_buffer: UiStatus,
    pub text_set: TextSet,
}

#[derive(Clone, Debug)]
pub enum UiStatus {
    QuitMenu,
    TitleMenu,
    DiceRoll,
    Skip,
    DiceResult,
    GameFinished,
}

#[derive(Clone, Debug, Default)]
pub struct TextSet {
    pub main_window: String,
    pub message: String,
    pub dice_string: String,
    pub guidance: String,
    pub player_list: String,
}

impl GameData {
    pub fn try_new(
        world: World,
        player_order: Vec<String>,
        player_status_table: HashMap<String, PlayerStatus>,
    ) -> Result<Self> {
        let current_player = player_order
            .first()
            .ok_or(GameSystemError::NoPlayer)?
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
    pub fn init(&mut self, preferences: &Preferences) -> Result<()> {
        self.text_set.set_guidance(preferences);
        self.text_set
            .set_prompt_dice_roll(preferences, self.world.dice_max());
        self.text_set.set_player_list(
            preferences,
            &self.current_player,
            &self.player_order,
            &self.player_status_table,
        )?;
        self.text_set.main_window = self.world.start_description(preferences);
        Ok(())
    }
    pub fn transition(&mut self, preferences: &Preferences, key: Key) -> Result<bool> {
        let mut flag_loop_break = false;
        match &self.ui_status {
            UiStatus::TitleMenu => {
                self.title_menu(preferences, key)?;
            }
            UiStatus::DiceRoll => {
                self.dice_roll(preferences, key)?;
            }
            UiStatus::Skip => {
                self.skip(preferences, key)?;
            }
            UiStatus::DiceResult => {
                self.dice_result(preferences, key)?;
            }
            UiStatus::QuitMenu => {
                if self.quit_menu(preferences, key)? {
                    flag_loop_break = true;
                }
            }
            UiStatus::GameFinished => self.game_finished(preferences, key)?,
        }
        Ok(flag_loop_break)
    }

    fn title_menu(&mut self, _preferences: &Preferences, key: Key) -> Result<()> {
        match key {
            Key::Char('\n') => {
                self.ui_status = UiStatus::DiceRoll;
                self.ui_status_buffer = UiStatus::DiceRoll;
            }
            Key::Esc => {
                self.ui_status = UiStatus::QuitMenu;
            }
            Key::Ctrl('l') => {}
            _ => return Ok(()),
        }
        Ok(())
    }

    fn dice_roll(&mut self, preferences: &Preferences, key: Key) -> Result<()> {
        match key {
            Key::Char(c) => {
                match c {
                    '0' => {
                        if !self.text_set.dice_string.is_empty() {
                            self.text_set.dice_string.push(c);
                            self.text_set
                                .set_prompt_dice_roll(preferences, self.world.dice_max());
                        }
                    }
                    '1'..='9' => {
                        self.text_set.dice_string.push(c);
                        self.text_set
                            .set_prompt_dice_roll(preferences, self.world.dice_max());
                    }
                    '\n' => {
                        if self.text_set.dice_string.is_empty() {
                            return Ok(());
                        }
                        self.text_set.set_prompt_enter(preferences);
                        match self.world.dice_roll(
                            preferences,
                            self.text_set.dice_string.parse()?,
                            &self.current_player,
                            &self.player_order,
                            &mut self.player_status_table,
                        ) {
                            Ok(main_window_text) => {
                                self.text_set.main_window = main_window_text;
                                self.change_player()?;
                            }
                            Err(GameSystemError::OutOfRangeDice(dice)) => {
                                self.ui_status = UiStatus::DiceResult;
                                self.ui_status_buffer = UiStatus::DiceResult;
                                self.text_set.set_dice_is_out_of_range(preferences, dice);
                            }
                            Err(e) => return Err(e.into()),
                        }
                    }
                    _ => {}
                };
            }
            Key::Backspace => {
                self.text_set.dice_string.pop();
                self.text_set
                    .set_prompt_dice_roll(preferences, self.world.dice_max());
            }
            Key::Esc => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::QuitMenu;
            }
            Key::Ctrl('t') => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::TitleMenu;
            }
            Key::Ctrl('l') => {}
            _ => return Ok(()),
        }
        Ok(())
    }

    fn skip(&mut self, preferences: &Preferences, key: Key) -> Result<()> {
        match key {
            Key::Char('\n') => {
                self.player_status_table
                    .get_mut(&self.current_player)
                    .ok_or_else(|| GameSystemError::NotFoundPlayer(self.current_player.to_owned()))?
                    .sub_num_skip(1);
                self.text_set.set_prompt_enter(preferences);
                self.text_set.main_window.clear();
                self.change_player()?;
            }
            Key::Esc => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::QuitMenu;
            }
            Key::Ctrl('t') => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::TitleMenu;
            }
            Key::Ctrl('l') => {}
            _ => return Ok(()),
        }
        Ok(())
    }

    fn dice_result(&mut self, preferences: &Preferences, key: Key) -> Result<()> {
        match key {
            Key::Char('\n') => {
                let num_skip_of_current_player = self
                    .player_status_table
                    .get(&self.current_player)
                    .ok_or_else(|| GameSystemError::NotFoundPlayer(self.current_player.to_owned()))?
                    .num_skip();
                if num_skip_of_current_player == 0 {
                    self.ui_status = UiStatus::DiceRoll;
                    self.ui_status_buffer = UiStatus::DiceRoll;
                    self.text_set.dice_string.clear();
                    self.text_set.main_window.clear();
                    self.text_set
                        .set_prompt_dice_roll(preferences, self.world.dice_max());
                } else {
                    self.ui_status = UiStatus::Skip;
                    self.ui_status_buffer = UiStatus::Skip;
                    self.text_set.set_prompt_enter(preferences);
                    self.text_set
                        .set_skip_player(preferences, num_skip_of_current_player);
                };
                self.text_set.set_player_list(
                    preferences,
                    &self.current_player,
                    &self.player_order,
                    &self.player_status_table,
                )?;
            }
            Key::Esc => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::QuitMenu;
            }
            Key::Ctrl('t') => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::TitleMenu;
            }
            Key::Ctrl('l') => {}
            _ => return Ok(()),
        }
        Ok(())
    }

    fn game_finished(&mut self, preferences: &Preferences, key: Key) -> Result<()> {
        match key {
            Key::Char('\n') => {
                self.text_set.set_prompt_game_finish(preferences);
            }
            Key::Esc => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::QuitMenu;
            }
            Key::Ctrl('t') => {
                self.ui_status_buffer = self.ui_status.clone();
                self.ui_status = UiStatus::TitleMenu;
            }
            Key::Ctrl('l') => {}
            _ => return Ok(()),
        }
        Ok(())
    }

    fn quit_menu(&mut self, _preferences: &Preferences, key: Key) -> Result<bool> {
        match key {
            Key::Char('Y') => Ok(true),
            Key::Ctrl('l') => Ok(false),
            _ => {
                self.ui_status = self.ui_status_buffer.clone();
                Ok(false)
            }
        }
    }

    fn change_player(&mut self) -> Result<()> {
        match self
            .player_order
            .next_player(&self.current_player, &mut self.player_status_table)?
        {
            Some(player) => {
                self.ui_status = UiStatus::DiceResult;
                self.ui_status_buffer = UiStatus::DiceResult;
                self.current_player = player;
            }
            None => {
                self.ui_status = UiStatus::GameFinished;
                self.ui_status_buffer = UiStatus::GameFinished;
            }
        }
        Ok(())
    }
}

impl TextSet {
    fn set_guidance(&mut self, preferences: &Preferences) {
        self.guidance.clear();
        match preferences.language() {
            Language::Japanese => {
                self.guidance.push_str("ESC: ??????\n");
                self.guidance.push_str("Ctrl-l: ?????????\n");
                self.guidance.push_str("Ctrl-t: ???????????????????????????");
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
        const GOAL_MARK: &str = "???? ";
        const DICE_MARK: &str = "???? ";
        self.player_list.clear();
        self.player_list.push_str(GOAL_MARK);
        self.player_list.push_str("   ");
        self.player_list.push_str("Name");
        self.player_list.push_str(match preferences.language() {
            Language::Japanese => "??????",
        });
        self.player_list.push('\n');
        for player in player_order {
            let order_of_arrival = player_status_table
                .get(player)
                .ok_or_else(|| GameSystemError::NotFoundPlayer(player.to_owned()))?
                .order_of_arrival();
            match order_of_arrival {
                // Some(x) => self.player_list.push_str(&format!("{0:>2} ", x)),
                // None => self.player_list.push_str(&format!("{0:>2} ", "")),
                Some(x) => write!(self.player_list, "{0:>2} ", x).unwrap(),
                None => write!(self.player_list, "{0:>2} ", "").unwrap(),
            }
            if player == current_player {
                self.player_list.push_str(DICE_MARK);
            } else {
                self.player_list.push_str("   ");
            }
            self.player_list.push_str(player);
            self.player_list.push('\n');
        }
        Ok(())
    }
    fn set_prompt_dice_roll(&mut self, preferences: &Preferences, dice_max: usize) {
        self.message.clear();
        match preferences.language() {
            Language::Japanese => {
                write!(
                    self.message,
                    "????????????????????????????????????????????????: {}???>> ",
                    dice_max
                )
                .unwrap();
                // self.message.push_str(&format!(
                //     "????????????????????????????????????????????????: {}???>> ",
                //     dice_max
                // ));
            }
        }
        self.message.push_str(self.dice_string.as_str());
    }
    fn set_prompt_enter(&mut self, preferences: &Preferences) {
        self.message.clear();
        match preferences.language() {
            Language::Japanese => self.message.push_str("?????????????????????????????????????????????"),
        }
    }
    fn set_prompt_game_finish(&mut self, preferences: &Preferences) {
        self.message.clear();
        self.main_window.clear();
        match preferences.language() {
            Language::Japanese => self
                .message
                .push_str("??????????????????????????????\n???????????????????????????????????????"),
        }
    }
    fn set_dice_is_out_of_range(&mut self, preferences: &Preferences, dice: usize) {
        match preferences.language() {
            Language::Japanese => {
                self.main_window = format!("????????????????????????????????????: {}", dice);
            }
        }
    }
    fn set_skip_player(&mut self, preferences: &Preferences, num_skip: u8) {
        match preferences.language() {
            Language::Japanese => {
                self.main_window = format!("????????????????????????????????????????????????: {}", num_skip)
            }
        }
    }
}
