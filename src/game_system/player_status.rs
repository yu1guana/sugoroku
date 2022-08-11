// Copyright (c) 2022 Yuichi Ishida

use crate::error::GameSystemError;
use anyhow::Result;
use std::collections::HashMap;

/// プレイヤーの状態
#[derive(Debug)]
pub struct PlayerStatus {
    position: usize,
    num_skip: u8,
    order_of_arrival: Option<u8>,
}

impl Default for PlayerStatus {
    fn default() -> Self {
        Self {
            position: 0,
            num_skip: 0,
            order_of_arrival: None,
        }
    }
}

impl PlayerStatus {
    pub fn position(&self) -> usize {
        self.position
    }
    pub fn set_position(&mut self, position: usize) {
        self.position = position;
    }
    pub fn num_skip(&self) -> u8 {
        self.num_skip
    }
    pub fn add_num_skip(&mut self, x: u8) {
        self.num_skip = self.num_skip.saturating_add(x);
    }
    pub fn sub_num_skip(&mut self, x: u8) {
        self.num_skip = self.num_skip.saturating_sub(x);
    }
    pub fn order_of_arrival(&self) -> Option<u8> {
        self.order_of_arrival
    }
    pub fn set_order_of_arrival(&mut self, order_of_arrival: u8) {
        self.order_of_arrival = Some(order_of_arrival);
    }
    pub fn go_forward(&mut self, n: usize) {
        self.position = self.position.saturating_add(n);
    }
    pub fn go_backward(&mut self, n: usize) {
        self.position = self.position.saturating_sub(n);
    }
}

pub trait PlayerOrder {
    fn next_player(
        &self,
        current_player: &str,
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<Option<String>>;
}

impl PlayerOrder for [String] {
    fn next_player(
        &self,
        current_player: &str,
        player_status_table: &mut HashMap<String, PlayerStatus>,
    ) -> Result<Option<String>> {
        let mut player_cycle = self.iter().cycle();
        loop {
            if player_cycle.next().unwrap() == current_player {
                break;
            }
        }
        for player in player_cycle.take(self.len()) {
            match player_status_table
                .get(player)
                .ok_or_else(|| GameSystemError::NotFoundPlayer(player.to_owned()))?
                .order_of_arrival()
            {
                Some(_) => {}
                None => {
                    if player_status_table.get(player).unwrap().num_skip() != 0 {
                        player_status_table.get_mut(player).unwrap().sub_num_skip(1);
                    } else {
                        return Ok(Some(player.to_owned()));
                    };
                }
            }
        }
        Ok(None)
    }
}
