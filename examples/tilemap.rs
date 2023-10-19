use bevy::{prelude::*, render::camera::ScalingMode, utils::HashMap, window::PrimaryWindow};
use bevy_pixels::{
    multi_tile::{multi_tile_delete, MultiTile},
    plugin::PixelPlugin,
    tile::Tile,
    tilemap::{world_unit_to_tile, Tilemap, TilemapBundle},
};

#[derive(Default, Resource)]
pub struct TileResource {
    image_handles: HashMap<String, Handle<Image>>,
}

pub fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest()))
        .add_plugins(PixelPlugin)
        .add_systems(Startup, setup_system)
        .add_systems(PostStartup, set_tons_of_tiles)
        .add_systems(Update, placement_system)
        .insert_resource(TileResource::default())
        .add_systems(PreUpdate, multi_tile_delete)
        .run()
}

pub fn setup_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut tile_resource: ResMut<TileResource>,
) {
    let mut camera_bundle = Camera2dBundle::default();
    camera_bundle.projection.viewport_origin = Vec2::new(0.5, 0.5);
    camera_bundle.projection.scaling_mode = ScalingMode::WindowSize(25.6);
    commands.spawn(camera_bundle);

    commands.spawn(TilemapBundle::default());

    let image_resoure = asset_server.load("lightslate.png");
    tile_resource
        .image_handles
        .insert("Lightslate".to_string(), image_resoure);
}

pub fn set_tons_of_tiles(mut commands: Commands, mut tilemaps: Query<&mut Tilemap>) {
    let mut tilemap = tilemaps.get_single_mut().unwrap();

    for x in -64..64 {
        for y in -64..64 {
            tilemap.set_tile(
                &mut commands,
                IVec2::new(x, y),
                Tile::from_color(Color::rgba(1.0, 1.0, 1.0, 1.0)),
                (),
            );
        }
    }
}

fn placement_system(
    mut commands: Commands,
    mut tilemaps: Query<&mut Tilemap>,
    input: Res<Input<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,

    images: Res<Assets<Image>>,
    tile_resource: Res<TileResource>,
) {
    if let Some(mouse_pos) = windows.single().cursor_position() {
        let (camera, trans) = cameras.single();
        let mouse_location = camera.viewport_to_world_2d(trans, mouse_pos).unwrap();

        // Now convert to tile coords.
        let tile_coord = world_unit_to_tile(mouse_location);

        if input.pressed(MouseButton::Left) {
            let image = images
                .get(tile_resource.image_handles.get("Lightslate").unwrap())
                .unwrap();
            // Set the tile at that location
            tilemaps.single_mut().set_tile(
                &mut commands,
                tile_coord,
                Tile::from_image(image, (16..24, 16..24)),
                (),
            );
        }

        if input.pressed(MouseButton::Right) {
            // Remove the tile at that location
            let image = images
                .get(tile_resource.image_handles.get("Lightslate").unwrap())
                .unwrap();

            let tile = MultiTile::from_image(image);
            tile.place(tile_coord, &mut tilemaps.single_mut(), &mut commands);
        }
    }
}
