// Copyright (c) 2022 Yuichi Ishida

use anyhow::Result;
use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use sugoroku::activate::Cli;

#[derive(Parser)]
#[clap(
    name = "Make completion script",
    author = env!("CARGO_PKG_AUTHORS"),
    version = "",
    about = "Make shellscript to complete arguments of Sugoroku."
    )]
struct AppArg {
    #[clap(arg_enum)]
    shell: Shell,
}

fn main() -> Result<()> {
    let arg = AppArg::parse();
    let mut app = Cli::command();
    let name = app.get_name().to_owned();
    let script_file_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("completion_script")
        .join(arg.shell.to_string() + concat!("_complition_of_", env!("CARGO_PKG_NAME"), ".sh"));

    let mut writer = BufWriter::new(File::create(&script_file_path)?);
    generate(arg.shell, &mut app, name, &mut writer);
    println!("Successfully done.");
    println!(
        "A completion script is created (the file path is `{}`).",
        script_file_path.display()
    );
    println!("Please read the sciprt using `source` command.");
    Ok(())
}
