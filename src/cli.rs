// Copyright (c) 2023 Yuichi Ishida
//
// Released under the MIT license.
// see https://opensource.org/licenses/mit-license.php

use anyhow::Result;
use clap::{Parser, Subcommand, ValueHint};
use std::path::PathBuf;

impl Cli {
    pub fn run() -> Result<()> {
        match Cli::parse().action {
            Action::Game {
                player_list_file,
                world_file,
            } => {
                crate::user_interface::tui::run(Default::default(), player_list_file, world_file)?;
                Ok(())
            }
            Action::WorldToTex { world_file } => {
                crate::world_to_tex::run(world_file)?;
                Ok(())
            }
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, after_help = concat!("Repository: ", env!("CARGO_PKG_REPOSITORY")))]
pub struct Cli {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Game {
        #[clap(value_hint(ValueHint::FilePath))]
        player_list_file: PathBuf,
        #[clap(value_hint(ValueHint::FilePath))]
        world_file: PathBuf,
    },
    WorldToTex {
        #[clap(value_hint(ValueHint::FilePath))]
        world_file: PathBuf,
    },
}
