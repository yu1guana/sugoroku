// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use crate::game_system::area::{Area, AreaEffect, NoEffect};
use crate::game_system::player_status::PlayerStatus;
use crate::game_system::world::World;
use anyhow::{Context, Result};
use serde_derive::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::str::FromStr;
use toml;

#[derive(Debug, Deserialize)]
struct PlayerListDescription {
    player: Vec<StatusDescription>,
}

#[derive(Debug, Deserialize)]
struct StatusDescription {
    name: String,
}

#[derive(Debug, Deserialize)]
struct WorldDescription {
    general: WorldSettingDescription,
    area: Vec<AreaDescription>,
}

#[derive(Debug, Deserialize)]
struct WorldSettingDescription {
    title: String,
    opening_msg: String,
    start_description: String,
    goal_description: String,
    dice_max: usize,
}

#[derive(Debug, Deserialize)]
struct AreaDescription {
    description: String,
    effect: Option<Vec<AreaEffectDescription>>,
}

#[derive(Debug, Deserialize)]
struct AreaEffectDescription {
    element: String,
}

pub fn read_player_list_from_file(
    file_path: &Path,
) -> Result<(Vec<String>, HashMap<String, PlayerStatus>)> {
    let file_contents = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;
    let player_description: PlayerListDescription = toml::from_str(&file_contents)
        .with_context(|| format!("failed to parse {}", file_path.display()))?;
    let mut player_status_table = HashMap::with_capacity(player_description.player.len());
    let mut player_order = Vec::with_capacity(player_description.player.len());
    for player in player_description.player {
        if player_status_table.contains_key(&player.name) {
            return Err(GameSystemError::DuplicatePlayer(player.name).into());
        } else {
            player_status_table.insert(player.name.to_owned(), PlayerStatus::default());
            player_order.push(player.name);
        }
    }
    Ok((player_order, player_status_table))
}

pub fn read_world_from_file(file_path: &Path) -> Result<World> {
    let file_contents = fs::read_to_string(file_path)
        .with_context(|| format!("failed to read {}", file_path.display()))?;
    let world_description: WorldDescription = toml::from_str(&file_contents)
        .with_context(|| format!("failed to parse {}", file_path.display()))?;
    let mut area_list = vec![Area::new(
        world_description.general.start_description,
        vec![Box::new(NoEffect::new())],
    )];
    for area_description in world_description.area.into_iter() {
        let area_effect_list: Vec<Box<dyn AreaEffect>> =
            if let Some(area_effect_description_list) = area_description.effect {
                area_effect_description_list
                    .into_iter()
                    .map(|area_effect_description| <_>::from_str(&area_effect_description.element))
                    .collect::<Result<_>>()?
            } else {
                vec![Box::new(NoEffect::new())]
            };
        area_list.push(Area::new(area_description.description, area_effect_list));
    }
    area_list.push(Area::new(
        world_description.general.goal_description,
        vec![Box::new(NoEffect::new())],
    ));
    Ok(World::new(
        world_description.general.title,
        world_description.general.opening_msg,
        world_description.general.dice_max,
        area_list,
    ))
}
