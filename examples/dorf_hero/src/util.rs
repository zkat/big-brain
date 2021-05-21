use crate::components::Position;

pub fn euclidean_distance(pos1: &Position, pos2: &Position) -> f32 {
    let a = (pos1.x - pos2.x) as f32;
    let b = (pos1.y - pos2.y) as f32;
    a.hypot(b)
}
