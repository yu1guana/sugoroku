// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use crate::game_system::player_status::PlayerStatus;
use crate::preferences::{Language, Preferences};
use anyhow::{anyhow, Context};
use std::collections::HashMap;
use std::str::FromStr;

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

impl FromStr for Box<dyn AreaEffect> {
    type Err = anyhow::Error;
    fn from_str(area_effect_str: &str) -> Result<Self, Self::Err> {
        macro_rules! parse_effect {
            ($effect_name_str:expr, $effect_parameters_str: expr, $($effect_names:ident),+) => {
                match $effect_name_str.as_str() {
                    $(stringify!($effect_names) => {
                        Ok(Box::new(
                                $effect_names::from_str($effect_parameters_str)
                                .with_context(||
                                    format!("faied to parse {} (the correct format is {})", stringify!($effect_names), $effect_names::input_format())
                                )?
                        ))
                    }),+
                    _ => Err(GameSystemError::NotFoundAreaType($effect_name_str.to_owned()).into()),
                }
            };
        }
        let area_effect_strings: Vec<_> = area_effect_str
            .replace(char::is_whitespace, "")
            .split(':')
            .map(|s| s.to_owned())
            .collect();
        if area_effect_strings.len() != 2 {
            return Err(anyhow!(
                "failed to parse an area effect (the correct format is `EffectName: [parameters...]`)."
            ));
        }
        let effect_name = area_effect_strings.get(0).unwrap();
        let effect_parameters = area_effect_strings.get(1).unwrap();
        parse_effect!(
            effect_name,
            effect_parameters,
            NoEffect,
            SkipSelf,
            PushSelf,
            PullSelf
        )
    }
}

macro_rules! err_msg_wrong_parameter {
    ($key:expr) => {
        format!("{} is a wrong parameter", $key)
    };
}

macro_rules! err_msg_parse_parameter {
    ($key:expr) => {
        format!("failed to parse a prameter `{}`", $key)
    };
}

fn try_get_key_value_list(
    effect_parameters: &str,
) -> Result<HashMap<String, String>, anyhow::Error> {
    let mut key_value_list = HashMap::new();
    for key_value_str in effect_parameters.split(',') {
        let key_value_strings: Vec<_> = key_value_str.split('=').map(|s| s.to_owned()).collect();
        if key_value_strings.len() != 2 {
            return Err(anyhow!(
                "failed to parse area effect parameters (the correct format is comma separated `key = value` list)."
            ));
        }
        let key = key_value_strings.get(0).unwrap().to_owned();
        let value = key_value_strings.get(1).unwrap().to_owned();
        if key_value_list.contains_key(&key) {
            return Err(anyhow!(format!(
                "failed to parse area effect parameters (`{}` is duplicated).",
                key
            )));
        }
        key_value_list.insert(key, value);
    }
    Ok(key_value_list)
}

/// 何も起こらない
#[derive(Clone, Debug)]
pub struct NoEffect {}
impl NoEffect {
    fn new() -> Self {
        Self {}
    }
    fn input_format() -> &'static str {
        "`NoEffect:`"
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
impl FromStr for NoEffect {
    type Err = anyhow::Error;
    fn from_str(effect_parameters: &str) -> Result<Self, Self::Err> {
        if !effect_parameters.is_empty() {
            Err(anyhow!("parameter must not exist.").into())
        } else {
            Ok(Self::new())
        }
    }
}

/// 次回以降プレイヤーをスキップする
///
/// ステージ作成時にはsetteingsに休む回数を記入する。
#[derive(Clone, Debug)]
pub struct SkipSelf {
    num_skip: u8,
}
impl SkipSelf {
    fn new(num_skip: u8) -> Self {
        Self { num_skip }
    }
    fn input_format() -> &'static str {
        "`SkipSelf: times = <u8>`"
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
impl FromStr for SkipSelf {
    type Err = anyhow::Error;
    fn from_str(effect_parameters: &str) -> Result<Self, Self::Err> {
        let mut num_skip = 0;
        let key_value_list = try_get_key_value_list(effect_parameters)?;
        for (key, value) in key_value_list {
            match key.as_str() {
                "times" => {
                    num_skip = value
                        .parse()
                        .with_context(|| err_msg_parse_parameter!(key))?;
                }
                _ => {
                    return Err(anyhow!(err_msg_wrong_parameter!(key)));
                }
            }
        }
        Ok(Self::new(num_skip))
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
    fn input_format() -> &'static str {
        "`PushSelf: num = <usize>`"
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
impl FromStr for PushSelf {
    type Err = anyhow::Error;
    fn from_str(effect_parameters: &str) -> Result<Self, Self::Err> {
        let mut num_push = 0;
        let key_value_list = try_get_key_value_list(effect_parameters)?;
        for (key, value) in key_value_list {
            match key.as_str() {
                "num" => {
                    num_push = value
                        .parse()
                        .with_context(|| err_msg_parse_parameter!(key))?;
                }
                _ => {
                    return Err(anyhow!(err_msg_wrong_parameter!(key)));
                }
            }
        }
        Ok(Self::new(num_push))
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
    fn input_format() -> &'static str {
        "`PullSelf: num = <usize>`"
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

impl FromStr for PullSelf {
    type Err = anyhow::Error;
    fn from_str(effect_parameters: &str) -> Result<Self, Self::Err> {
        let mut num_push = 0;
        let key_value_list = try_get_key_value_list(effect_parameters)?;
        for (key, value) in key_value_list {
            match key.as_str() {
                "num" => {
                    num_push = value
                        .parse()
                        .with_context(|| err_msg_parse_parameter!(key))?;
                }
                _ => {
                    return Err(anyhow!(err_msg_wrong_parameter!(key)));
                }
            }
        }
        Ok(Self::new(num_push))
    }
}
