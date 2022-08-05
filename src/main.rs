// Copyright (c) 2022 Yuichi Ishida

mod area;
mod error;
mod player_status;
mod preferences;
mod toml_interface;
mod user_interface;
mod world;

use crate::toml_interface::{read_player_list_from_file, read_world_from_file};
use anyhow::Result;

fn main() -> Result<()> {
    let (player_order, player_status_table) =
        read_player_list_from_file("example_player_list.toml")?;
    let world = read_world_from_file("example_world.toml")?;
    crate::user_interface::tui::run(Default::default(), world, player_order, player_status_table)?;
    Ok(())
}
