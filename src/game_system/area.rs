// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use crate::game_system::player_status::PlayerStatus;
use crate::preferences::{Language, Preferences};
use anyhow::Context;
use std::collections::HashMap;

/// 各マスを表す
#[derive(Debug)]
pub struct Area {
    description: String,
    effect_list: Vec<Box<dyn AreaEffect>>,
}

impl Area {
    pub fn new(description: String, effect_list: Vec<Box<dyn AreaEffect>>) -> Self {
        Self {
            description,
            effect_list,
        }
    }
    pub fn execute(
        &self,
        current_player: &str,
        player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError> {
        for effect in self.effect_list.iter() {
            effect.execute(current_player, player_order, player_status_table)?;
        }
        Ok(())
    }
    pub fn area_description(&self, preferences: &Preferences) -> String {
        let mut text = self.description.clone();
        text += "\n\n";
        match preferences.language() {
            Language::Japanese => text += "効果\n",
        }
        for effect in self.effect_list.iter() {
            text += "- ";
            text += &effect.effect_text(preferences);
            text += "\n";
        }
        text
    }
}

/// マスの持つ効果
pub trait AreaEffect: core::fmt::Debug {
    fn effect_text(&self, preferences: &Preferences) -> String;
    fn execute(
        &self,
        current_player: &str,
        player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError>;
}

/// 文字列からAreaEffectを作成
pub fn try_make_area_effect(
    area_type: &str,
    settings: &str,
) -> Result<Box<dyn AreaEffect>, anyhow::Error> {
    match area_type {
        "NoEffect" => Ok(Box::new(NoEffect::new())),
        "SkipSelf" => {
            let num_skip = settings
                .replace(char::is_whitespace, "")
                .parse()
                .with_context(|| {
                    "failed to parse settings of SkipSelf from String.\nFormat: <num_skip: u8>"
                })?;
            Ok(Box::new(SkipSelf::new(num_skip)))
        }
        "PushSelf" => {
            let num_advance = settings.replace(char::is_whitespace,"")
                .parse()
                .with_context(|| "failed to parse settings of PushSelf from String.\nFormat: <num_advance: usize>")?;
            Ok(Box::new(PushSelf::new(num_advance)))
        }
        "PullSelf" => {
            let num_disadvance = settings.replace(char::is_whitespace,"")
                .parse()
                .with_context(|| "failed to parse settings of PullSelf from String.\nFormat: <num_disadvance: usize>")?;
            Ok(Box::new(PullSelf::new(num_disadvance)))
        }
        _ => Err(GameSystemError::NotFoundAreaType(area_type.to_owned()).into()),
    }
}

/// 何も起こらない
#[derive(Debug)]
pub struct NoEffect {}
impl NoEffect {
    fn new() -> Self {
        Self {}
    }
}
impl AreaEffect for NoEffect {
    fn effect_text(&self, preferences: &Preferences) -> String {
        match preferences.language() {
            Language::Japanese => format!("なし"),
        }
    }
    fn execute(
        &self,
        _current_player: &str,
        _player_order: &[String],
        _player_status_list: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError> {
        Ok(())
    }
}

/// 次回以降プレイヤーをスキップする
///
/// ステージ作成時にはsetteingsに休む回数を記入する。
#[derive(Debug)]
pub struct SkipSelf {
    num_skip: u8,
}
impl SkipSelf {
    fn new(num_skip: u8) -> Self {
        Self { num_skip }
    }
}
impl AreaEffect for SkipSelf {
    fn effect_text(&self, preferences: &Preferences) -> String {
        match preferences.language() {
            Language::Japanese => format!("プレイヤーの休みを{}回追加。", self.num_skip),
        }
    }
    fn execute(
        &self,
        current_player: &str,
        _player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError> {
        player_status_table
            .get_mut(current_player)
            .ok_or_else(|| GameSystemError::NotFoundPlayer(current_player.to_owned()))?
            .add_num_skip(self.num_skip);
        Ok(())
    }
}

/// プレイヤーを進める
///
/// ステージ作成時にはsetteingsに進む数を記入する。
#[derive(Clone, Debug)]
pub struct PushSelf {
    num_advance: usize,
}
impl PushSelf {
    pub fn new(num_advance: usize) -> Self {
        Self { num_advance }
    }
}
impl AreaEffect for PushSelf {
    fn effect_text(&self, preferences: &Preferences) -> String {
        match preferences.language() {
            Language::Japanese => format!("プレイヤーは{} マス進む。", self.num_advance),
        }
    }
    fn execute(
        &self,
        current_player: &str,
        _player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError> {
        player_status_table
            .get_mut(current_player)
            .ok_or_else(|| GameSystemError::NotFoundPlayer(current_player.to_owned()))?
            .go_forward(self.num_advance);
        Ok(())
    }
}

/// プレイヤーを戻す
///
/// ステージ作成時にはsetteingsに戻す数を記入する。
#[derive(Clone, Debug)]
pub struct PullSelf {
    num_disadvance: usize,
}
impl PullSelf {
    pub fn new(num_disadvance: usize) -> Self {
        Self { num_disadvance }
    }
}
impl AreaEffect for PullSelf {
    fn effect_text(&self, preferences: &Preferences) -> String {
        match preferences.language() {
            Language::Japanese => format!("プレイヤーは{} マス戻る。", self.num_disadvance),
        }
    }
    fn execute(
        &self,
        current_player: &str,
        _player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<(), GameSystemError> {
        player_status_table
            .get_mut(current_player)
            .ok_or_else(|| GameSystemError::NotFoundPlayer(current_player.to_owned()))?
            .go_backward(self.num_disadvance);
        Ok(())
    }
}
