use std::convert::From;

#[derive(Clone, Copy)]
pub enum Move {
    Top,
    Bottom,
    Left,
    Right,
    Front,
    Back,
    None,
}

impl From<i32> for Move {
    fn from(v: i32) -> Self {
        match v {
            0 => Move::Top,
            1 => Move::Bottom,
            2 => Move::Left,
            3 => Move::Right,
            4 => Move::Front,
            5 => Move::Back,
            _ => Move::None,
        }
    }
}
