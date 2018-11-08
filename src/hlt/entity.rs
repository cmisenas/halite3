use hlt::position::Position;
use hlt::PlayerId;

pub trait Entity {
    fn owner(&self) -> PlayerId;
    fn position(&self) -> Position;
}
