// Copyright (c) 2022 Yuichi Ishida

#[derive(thiserror::Error, Debug)]
pub enum GeneralError {
    #[error("Player is not found: {0}")]
    NotFoundPlayer(String),
    #[error("Area type is not found: {0}")]
    NotFoundAreaType(String),
    #[error("duplicate player: {0}")]
    DuplicatePlayer(String),
    #[error("{0}")]
    OutOfRangeDice(usize),
    #[error("Position is out of range: {0} {1}")]
    OutOfRangePosition(String, usize),
    #[error("There is no player")]
    NoPlayer,
}
