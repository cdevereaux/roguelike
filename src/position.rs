use bevy::prelude::*;
use std::ops::Add;

#[derive(Component, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy)]
pub struct PositionDelta {
    pub x: isize,
    pub y: isize,
}

impl Add<PositionDelta> for Position {
    type Output = Position;

    fn add(self, rhs: PositionDelta) -> Position {
        Self {
            x: self.x.saturating_add_signed(rhs.x),
            y: self.y.saturating_add_signed(rhs.y),
        }
    }
}

impl PositionDelta {
    pub fn new(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}
