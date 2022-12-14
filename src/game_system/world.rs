// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use crate::error::GameSystemError;
use crate::game_system::area::Area;
use crate::game_system::player_status::PlayerStatus;
use crate::preferences::Preferences;
use rand::rngs::ThreadRng;
use std::collections::HashMap;

#[derive(Debug)]
pub struct World {
    title: String,
    opening_msg: String,
    dice_max: usize,
    area_list: Vec<Area>,
    num_goal_player: u8,
    rng: ThreadRng,
}

impl World {
    pub fn new(title: String, opening_msg: String, dice_max: usize, area_list: Vec<Area>) -> Self {
        Self {
            title,
            opening_msg,
            dice_max,
            area_list,
            num_goal_player: 0,
            rng: rand::thread_rng(),
        }
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn opening_msg(&self) -> &str {
        &self.opening_msg
    }
    pub fn dice_max(&self) -> usize {
        self.dice_max
    }
    pub fn area_list(&self) -> &Vec<Area> {
        &self.area_list
    }
    pub fn start_description(&self, preferences: &Preferences) -> String {
        self.area_list
            .first()
            .unwrap()
            .area_description(preferences)
    }
    pub fn dice_roll(
        &mut self,
        preferences: &Preferences,
        dice: usize,
        current_player: &str,
        player_order: &[String],
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<String, GameSystemError> {
        if dice < 1 || self.dice_max < dice {
            return Err(GameSystemError::OutOfRangeDice(dice));
        }
        player_status_table
            .get_mut(current_player)
            .ok_or_else(|| GameSystemError::NotFoundPlayer(current_player.to_owned()))?
            .go_forward(dice);
        self.check_goal_player(player_status_table);
        let current_player_position = player_status_table
            .get_mut(current_player)
            .ok_or_else(|| GameSystemError::NotFoundPlayer(current_player.to_owned()))?
            .position();
        self.area_list
            .get(current_player_position)
            .ok_or_else(|| {
                GameSystemError::OutOfRangePosition(
                    current_player.to_owned(),
                    current_player_position,
                )
            })?
            .execute(
                current_player,
                player_order,
                player_status_table,
                &mut self.rng,
            )?;
        self.check_goal_player(player_status_table);
        Ok(self
            .area_list
            .get(current_player_position)
            .ok_or_else(|| {
                GameSystemError::OutOfRangePosition(
                    current_player.to_owned(),
                    current_player_position,
                )
            })?
            .area_description(preferences))
    }
    fn check_goal_player(&mut self, player_status_table: &mut HashMap<String, PlayerStatus>) {
        let mut num_goal_player = 0;
        for player_status in player_status_table.values_mut() {
            if player_status.order_of_arrival() == None
                && player_status.position() >= self.area_list.len() - 1
            {
                player_status.set_order_of_arrival(self.num_goal_player + 1);
                player_status.set_position(self.area_list.len() - 1);
                num_goal_player += 1;
            }
        }
        self.num_goal_player += num_goal_player;
    }
}
