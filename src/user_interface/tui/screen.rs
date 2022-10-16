// Copyright (c) 2022 Yuichi Ishida

use crate::preferences::{Language, Preferences};
use crate::user_interface::tui::status::{GameData, UiStatus};
use tui::backend::Backend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::terminal::Frame;
use tui::widgets::{Block, Borders, Paragraph};

pub fn ui<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences, game_data: &GameData) {
    match game_data.ui_status {
        UiStatus::TitleMenu => ui_title(frame, preferences, game_data),
        UiStatus::QuitMenu => {
            ui_quit(frame, preferences);
        }
        _ => ui_playing(frame, preferences, game_data),
    }
}

fn ui_title<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences, game_data: &GameData) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new(game_data.world.title())
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let mut opening_msg_text = String::new();
    opening_msg_text.push('\n');
    opening_msg_text.push_str(game_data.world.opening_msg());
    opening_msg_text.push('\n');
    opening_msg_text.push('\n');
    match game_data.ui_status_buffer {
        UiStatus::TitleMenu => {
            opening_msg_text.push_str(match preferences.language() {
                Language::Japanese => "開始するにはエンターキーを押してください。",
            });
        }
        _ => {
            opening_msg_text.push_str(match preferences.language() {
                Language::Japanese => "ゲームに戻るにはエンターキーを押してください。",
            });
        }
    }
    let opening_msg = Paragraph::new(opening_msg_text)
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
}

fn ui_playing<B: Backend>(frame: &mut Frame<B>, _preferences: &Preferences, game_data: &GameData) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(frame.size());
    frame.render_widget(
        Paragraph::new(game_data.text_set.guidance.as_str())
            .block(Block::default().title("Guidance").borders(Borders::ALL)),
        chunks[0],
    );
    let bottom_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(chunks[1]);
    frame.render_widget(
        Paragraph::new(game_data.text_set.player_list.as_str())
            .block(Block::default().title("Player list").borders(Borders::ALL)),
        bottom_chunks[0],
    );
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .horizontal_margin(1)
        .constraints([Constraint::Percentage(10), Constraint::Percentage(90)].as_ref())
        .split(bottom_chunks[1]);
    frame.render_widget(
        Paragraph::new(game_data.text_set.message.as_str())
            .block(Block::default().title("Message").borders(Borders::ALL)),
        right_chunks[0],
    );
    frame.render_widget(
        Paragraph::new(game_data.text_set.main_window.as_str())
            .block(Block::default().borders(Borders::ALL)),
        right_chunks[1],
    );
}

fn ui_quit<B: Backend>(frame: &mut Frame<B>, preferences: &Preferences) {
    let chunks = Layout::default()
        .margin(1)
        .constraints([
            Constraint::Percentage(40),
            Constraint::Length(3),
            Constraint::Percentage(50),
        ])
        .split(frame.size());
    let title = Paragraph::new(match preferences.language() {
        Language::Japanese => "ゲームを終了しますか？",
    })
    .alignment(Alignment::Center)
    .block(Block::default());
    frame.render_widget(title, chunks[1]);
    let opening_msg = Paragraph::new("Y / [n]")
        .alignment(Alignment::Center)
        .block(Block::default());
    frame.render_widget(opening_msg, chunks[2]);
}
