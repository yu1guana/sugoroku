// Copyright (c) 2022 Yuichi Ishida

use crate::game_system::toml_interface::{read_player_list_from_file, read_world_from_file};
use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;

impl Cli {
    pub fn run() -> Result<()> {
        match Cli::parse().action {
            Action::Game {
                world_file,
                player_list_file,
            } => {
                let (player_order, player_status_table) =
                    read_player_list_from_file(&player_list_file)?;
                let world = read_world_from_file(&world_file)?;
                crate::user_interface::tui::run(
                    Default::default(),
                    world,
                    player_order,
                    player_status_table,
                )?;
                Ok(())
            }
            Action::WorldToTex { world_file } => {
                unimplemented!();
            }
        }
    }
}

#[derive(Parser)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    author = env!("CARGO_PKG_AUTHORS"),
    version = env!("CARGO_PKG_VERSION"),
    about = "Sugoloku"
)]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Game {
        #[clap(value_hint(ValueHint::FilePath))]
        world_file: PathBuf,
        #[clap(value_hint(ValueHint::FilePath))]
        player_list_file: PathBuf,
    },
    WorldToTex {
        #[clap(value_hint(ValueHint::FilePath))]
        world_file: PathBuf,
    },
}
