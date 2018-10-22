use hlt::direction::Direction;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn directional_offset(&self, d: Direction) -> Position {
        let (dx, dy) = match d {
            Direction::North => (0, -1),
            Direction::South => (0, 1),
            Direction::East => (1, 0),
            Direction::West => (-1, 0),
            Direction::Still => (0, 0),
        };

        Position { x: self.x + dx, y: self.y + dy }
    }

    // TODO: Fix so that if other position is not in surrounding cardinal direction, it doesn't just return Direction::Still
    pub fn get_direction_to_position(&self, p: &Position) -> Direction {
        match (self.x - p.x, self.y - p.y) {
            (0, -1) => Direction::South,
            (0, 1)  => Direction::North,
            (-1, 0) => Direction::East,
            (1, 0)  => Direction::West,
            _       => Direction::Still,
        }
    }
    pub fn equal(&self, p: &Position) -> bool {
      self.x == p.x && self.y == p.y
    }

    pub fn get_surrounding_cardinals(&self) -> Vec<Position> {
        vec! {
            self.directional_offset(Direction::North), self.directional_offset(Direction::South),
            self.directional_offset(Direction::East), self.directional_offset(Direction::West)
        }
    }
}
