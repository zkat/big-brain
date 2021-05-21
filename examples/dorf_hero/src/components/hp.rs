pub struct Hp {
    pub current: u32,
    pub max: u32,
}

impl Hp {
    pub fn new(max: u32) -> Self {
        Hp { current: max, max }
    }
}
