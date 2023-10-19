use bevy::prelude::*;

use crate::{
    chunk::{chunk_deleter, chunk_texture_update},
    tilemap::tilemap_event_system,
};

pub struct PixelPlugin;

impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (chunk_deleter, tilemap_event_system, chunk_texture_update).chain(),
        );
    }
}
