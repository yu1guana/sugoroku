// Copyright (c) 2022 Yuichi Ishida

use crate::game_system::toml_interface::read_world_from_file;
use crate::preferences::Preferences;
use anyhow::Result;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

pub fn run(world_file_path: PathBuf) -> Result<()> {
    let preferences: Preferences = Default::default();
    let world = read_world_from_file(&world_file_path)?;
    let tex_file_name = world_file_path
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
        + ".tex";
    let tex_file_path = world_file_path.parent().unwrap().join(tex_file_name);
    let mut buf_writer = BufWriter::new(File::create(tex_file_path)?);
    writeln!(
        buf_writer,
        "{}",
        r#"\documentclass[11pt,dvipdfmx]{jsarticle}"#
    )?;
    writeln!(buf_writer, "{}", r#""#)?;
    writeln!(buf_writer, "{}", r#"\usepackage{tcolorbox}"#)?;
    writeln!(
        buf_writer,
        "{}",
        r#"\newtcolorbox{areabox}[2][]{colbacktitle=black,coltitle=white,title={#2}}"#
    )?;
    writeln!(buf_writer, "{}", r#""#)?;
    writeln!(buf_writer, "{}", r#"\begin{document}"#)?;
    writeln!(
        buf_writer,
        "{}",
        r#"\title{"#.to_owned() + world.title() + "}"
    )?;
    writeln!(buf_writer, "{}", r#"\author{}"#)?;
    writeln!(buf_writer, "{}", r#"\date{}"#)?;
    writeln!(buf_writer, "{}", r#"\maketitle"#)?;
    writeln!(buf_writer, "{}", r#""#)?;
    for (i_area, area) in world.area_list().iter().enumerate() {
        writeln!(
            buf_writer,
            "{}",
            r#"\begin{areabox}{"#.to_owned() + &format!("{}", i_area) + "}"
        )?;
        for line in area.area_description(&preferences).lines() {
            writeln!(buf_writer, "{}\\\\", line)?;
        }
        writeln!(buf_writer, "{}", r#"\end{areabox}"#)?;
        writeln!(buf_writer, "{}", r#""#)?;
    }
    writeln!(buf_writer, "{}", r#"\end{document}"#)?;
    Ok(())
}
