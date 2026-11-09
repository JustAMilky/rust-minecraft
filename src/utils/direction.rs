use serde::Deserialize;

#[derive(Deserialize, Hash, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    North,
    South,
    East,
    West,
}