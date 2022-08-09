// Copyright (c) 2022 Yuichi Ishida

#[derive(Clone, Copy, Debug, Default)]
pub struct Preferences {
    language: Language,
}

impl Preferences {
    pub fn language(&self) -> Language {
        self.language
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Language {
    Japanese,
}

impl Default for Language {
    fn default() -> Self {
        Self::Japanese
    }
}

// #[derive(Clone, Copy, Debug, Default)]
// struct TuiPreferences {
//     player_list_window_width: u16,
//     player_list_window_height: u16,
//     guidance_window_width: u16,
//     guidance_window_height: u16,
//     body_window_width: u16,
//     body_window_height: u16,
//     input_window_width: u16,
//     input_window_height: u16,
// }
