use bevy::prelude::HandleUntyped;

#[derive(Default, Clone)]
pub struct TileSpriteHandles {
    pub handles: Vec<HandleUntyped>,
    pub atlas_loaded: bool,
}
