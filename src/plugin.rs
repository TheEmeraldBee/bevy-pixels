use bevy::prelude::*;

use crate::{
    chunk::{chunk_deleter, chunk_texture_update},
    multi_tile::multi_tile_delete,
    tilemap::tilemap_event_system,
};

pub struct PixelPlugin;

impl Plugin for PixelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (
                chunk_deleter,
                multi_tile_delete,
                tilemap_event_system,
                chunk_texture_update,
            )
                .chain(),
        );
    }
}
