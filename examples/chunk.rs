use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_pixel_map::{
    chunk::{Chunk, ChunkBundle},
    plugin::PixelPlugin,
    tile::Tile,
};

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(PostStartup, set_tons_of_tiles)
        .run()
}

pub fn setup_system(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.viewport_origin = Vec2::new(0.5, 0.5);
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(25.6);
    commands.spawn(camera_bundle);

    commands.spawn(ChunkBundle::new(IVec2::new(0, 0), Chunk::new(&mut images)));
}

pub fn set_tons_of_tiles(mut commands: Commands, mut chunks: Query<(Entity, &mut Chunk)>) {
    let mut tile = Tile::from_color(Color::rgba(1.0, 1.0, 1.0, 1.0));
    tile.set_pixel(IVec2::new(1, 5), Color::rgba(0.0, 1.0, 0.0, 1.0));
    for (entity, mut chunk) in &mut chunks {
        chunk.set_tile(entity, IVec2::new(3, 2), tile.clone(), (), &mut commands);
    }
}
